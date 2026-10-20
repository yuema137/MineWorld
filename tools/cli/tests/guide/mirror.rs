//! The checks that tie a guide page to its mirror and to its links (`docs/GUIDE_FORMAT.md` §§7, 8.2, 8.3).
//! Both the English page and its `*.zh-CN.md` mirror are read the same way, by [`outline`].

use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

/// What a page contains, in order, for comparison with its mirror.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outline {
    /// The text of every level-3 heading.
    pub level3: Vec<String>,
    /// The anchor of every heading (§8.2).
    pub slugs: Vec<String>,
    /// The info string and content of every fenced block, executed or shown.
    pub fences: Vec<(String, String)>,
    /// The text after `guide:` of every `<!-- guide: … -->` comment.
    pub directives: Vec<String>,
    /// The destination of every link.
    pub links: Vec<String>,
}

/// Reads a page's outline. Line endings are normalised first, as the page reader does.
pub fn outline(source: &str) -> Outline {
    let text = source.replace("\r\n", "\n");
    let mut outline = Outline::default();
    let mut heading: Option<(HeadingLevel, String)> = None;
    let mut events = Parser::new(&text);
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => heading = Some((level, String::new())),
            Event::End(TagEnd::Heading(_)) => {
                if let Some((level, title)) = heading.take() {
                    let title = title.trim().to_owned();
                    if level == HeadingLevel::H3 {
                        outline.level3.push(title.clone());
                    }
                    outline.slugs.push(slug(&title));
                }
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, title)) = heading.as_mut() {
                    title.push_str(&text);
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind {
                    CodeBlockKind::Fenced(info) => info.into_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let mut content = String::new();
                for event in events.by_ref() {
                    match event {
                        Event::Text(text) => content.push_str(&text),
                        Event::End(TagEnd::CodeBlock) => break,
                        _ => {}
                    }
                }
                outline.fences.push((info, content));
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                for line in html.lines() {
                    let comment = line
                        .trim()
                        .strip_prefix("<!--")
                        .and_then(|inner| inner.strip_suffix("-->"));
                    if let Some(directive) =
                        comment.and_then(|inner| inner.trim().strip_prefix("guide:"))
                    {
                        outline.directives.push(directive.trim().to_owned());
                    }
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => outline.links.push(dest_url.into_string()),
            _ => {}
        }
    }
    outline
}

/// The anchor a heading gets, as §8.2 spells it: lowercase; letters, digits and hyphens kept; each space a
/// hyphen; everything else dropped.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    for character in title.chars() {
        if character == ' ' {
            out.push('-');
        } else if character.is_alphanumeric() || character == '-' {
            out.extend(character.to_lowercase());
        }
    }
    out
}

/// §7: the mirror has the English page's steps, code blocks and directives, in the same order, and each code
/// block byte for byte. The first difference is returned.
pub fn mirror_problem(english: &Outline, mirror: &Outline) -> Option<String> {
    let steps = |outline: &Outline| -> Vec<String> {
        outline
            .level3
            .iter()
            .filter_map(|heading| {
                let rest = heading.strip_prefix("Step ")?;
                rest.split_whitespace().next().map(str::to_owned)
            })
            .collect()
    };
    let (english_steps, mirror_steps) = (steps(english), steps(mirror));
    if english_steps != mirror_steps {
        return Some(format!(
            "the steps differ: English {english_steps:?}, mirror {mirror_steps:?}"
        ));
    }
    if english.fences.len() != mirror.fences.len() {
        return Some(format!(
            "English has {} code block(s), the mirror {}",
            english.fences.len(),
            mirror.fences.len()
        ));
    }
    for (index, (ours, theirs)) in english.fences.iter().zip(&mirror.fences).enumerate() {
        if ours != theirs {
            return Some(format!(
                "code block {} differs from its English original (English starts with info {:?}); \
                 a mirror copies code blocks byte for byte",
                index + 1,
                ours.0
            ));
        }
    }
    if english.directives != mirror.directives {
        return Some(format!(
            "the directives differ: English {:?}, mirror {:?}",
            english.directives, mirror.directives
        ));
    }
    None
}

/// §8.2 and §8.3: every relative link names an existing file, and every anchor names a heading of that file.
/// `reference` is the generated reference folder, as an absolute normalised path. While it does not exist
/// (`reference_present` is false) a link into it is not checked. Returns one problem per broken link.
pub fn link_problems(
    page: &Path,
    outline: &Outline,
    reference: &Path,
    reference_present: bool,
) -> Vec<String> {
    let folder = page.parent().unwrap_or(Path::new("."));
    let mut problems = Vec::new();
    for link in &outline.links {
        if link.starts_with("http://")
            || link.starts_with("https://")
            || link.starts_with("mailto:")
        {
            continue;
        }
        let (path, anchor) = match link.split_once('#') {
            Some((path, anchor)) => (path, Some(anchor)),
            None => (link.as_str(), None),
        };
        if path.is_empty() {
            if let Some(anchor) = anchor
                && !outline.slugs.iter().any(|slug| slug == anchor)
            {
                problems.push(format!("{link}: this page has no heading with that anchor"));
            }
            continue;
        }
        let target = normalise(&folder.join(path));
        if target.starts_with(reference) && !reference_present {
            continue;
        }
        if !target.is_file() {
            problems.push(format!("{link}: no file {}", target.display()));
            continue;
        }
        if let Some(anchor) = anchor {
            match std::fs::read_to_string(&target) {
                Ok(source) if outline_of_heading(&source, anchor) => {}
                Ok(_) => problems.push(format!("{link}: {} has no heading with that anchor", path)),
                Err(error) => problems.push(format!("{link}: {error}")),
            }
        }
    }
    problems
}

fn outline_of_heading(source: &str, anchor: &str) -> bool {
    outline(source).slugs.iter().any(|slug| slug == anchor)
}

/// `path` with its `.` and `..` components resolved, without touching the file system.
pub fn normalise(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}
