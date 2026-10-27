//! Re-recording a page's expected output (`docs/GUIDE_FORMAT.md` §11.2). With `MINEWORLD_WRITE_GUIDE=1` the
//! real guide's run writes what each console command printed back into the page, in place of the lines that no
//! longer match, and copies the new block into the page's mirror where the mirror's block was identical. Prose,
//! directives, file blocks, shell blocks and commands are never written.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

/// The environment variable that turns write mode on, when it is `1`.
pub const WRITE_VAR: &str = "MINEWORLD_WRITE_GUIDE";

/// Whether a run checks the pages, or re-records the expected output of the console blocks that differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// A difference fails its block (§4.4).
    Check,
    /// A difference is written into the page, and its block counts as passed (§11.2).
    Write,
}

impl Mode {
    /// `Write` when `MINEWORLD_WRITE_GUIDE` is `1`, otherwise `Check`.
    pub fn from_env() -> Self {
        match std::env::var(WRITE_VAR) {
            Ok(value) if value == "1" => Self::Write,
            _ => Self::Check,
        }
    }
}

/// The expected output write mode records for one command (§11.2). The expected lines are aligned with the lines
/// the command printed, so that as many as possible still match: a `[..]` line that matches its printed line is
/// kept as written, a `...` is kept and stands for the printed lines between the lines that still match, an
/// expected line that no longer matches is replaced by the printed line in its place, and a printed line nothing
/// matches is written literally. snapbox's own overwrite stops keeping wildcards at the first line that differs,
/// which would turn a `[..]` timing below a changed line into a literal one; the alignment keeps it. Should the
/// alignment ever not match the output, snapbox's own record is used.
pub fn recorded(expected: &str, actual: &str) -> String {
    let actual = snapbox::filter::normalize_paths(&snapbox::filter::normalize_lines(actual));
    let wanted: Vec<&str> = expected.lines().collect();
    let printed: Vec<&str> = actual.lines().collect();
    let lines = align(&wanted, &printed);
    let text: String = lines.iter().map(|line| format!("{line}\n")).collect();
    if super::command::matches(&text, &actual).is_ok() {
        text
    } else {
        super::command::snapbox_recorded(expected, &actual)
    }
}

/// One step of the alignment, in the order a tie is broken.
#[derive(Clone, Copy)]
enum Step {
    /// The expected line matches the printed line: keep the expected line.
    Keep,
    /// A `...` stands for the printed line.
    Elide,
    /// The expected line no longer matches: the printed line takes its place.
    Replace,
    /// A printed line nothing stands for: written literally.
    Insert,
    /// An expected line with no printed line left for it: dropped (a `...` is kept, matching none).
    Drop,
}

/// The recorded lines of `wanted` aligned with `printed`: the alignment with the most kept lines (each worth 3),
/// then the most replaced ones (each 1), so a changed line is written where its old one stood.
fn align<'a>(wanted: &[&'a str], printed: &[&'a str]) -> Vec<&'a str> {
    let (n, m) = (wanted.len(), printed.len());
    let line_matches = |i: usize, j: usize| {
        super::command::matches(&format!("{}\n", wanted[i]), &format!("{}\n", printed[j])).is_ok()
    };
    let mut best = vec![vec![0_u32; m + 1]; n + 1];
    let mut step = vec![vec![Step::Drop; m + 1]; n + 1];
    for i in (0..=n).rev() {
        for j in (0..=m).rev() {
            let mut options: Vec<(u32, Step)> = Vec::new();
            if i < n && wanted[i] == "..." {
                if j < m {
                    options.push((best[i][j + 1], Step::Elide));
                }
                options.push((best[i + 1][j], Step::Drop));
            } else {
                if i < n && j < m {
                    if line_matches(i, j) {
                        options.push((3 + best[i + 1][j + 1], Step::Keep));
                    }
                    options.push((1 + best[i + 1][j + 1], Step::Replace));
                }
                if j < m {
                    options.push((best[i][j + 1], Step::Insert));
                }
                if i < n {
                    options.push((best[i + 1][j], Step::Drop));
                }
            }
            if let Some(&(score, chosen)) = options
                .iter()
                .reduce(|kept, next| if next.0 > kept.0 { next } else { kept })
            {
                best[i][j] = score;
                step[i][j] = chosen;
            }
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n || j < m {
        match step[i][j] {
            Step::Keep => {
                out.push(wanted[i]);
                i += 1;
                j += 1;
            }
            Step::Elide => {
                if out.last() != Some(&"...") {
                    out.push("...");
                }
                j += 1;
            }
            Step::Replace => {
                out.push(printed[j]);
                i += 1;
                j += 1;
            }
            Step::Insert => {
                out.push(printed[j]);
                j += 1;
            }
            Step::Drop => {
                if wanted[i] == "..." && out.last() != Some(&"...") {
                    out.push("...");
                }
                i += 1;
            }
        }
    }
    out
}

/// A console block's new content, named by the line of its opening fence, as the page reader numbers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rewrite {
    pub line: usize,
    pub lines: Vec<String>,
}

/// One fenced block of a page, as the source spells it.
struct Fence {
    /// The 1-based line of the opening fence.
    line: usize,
    /// The info string and the content, as the Markdown reader sees them (the mirror rule's terms, §7).
    info: String,
    content: String,
    /// The bytes of the content in the source, between the opening and the closing fence.
    span: Range<usize>,
    /// The whitespace before the opening fence, which every content line carries too.
    indent: String,
}

/// Every fenced block of `text` (with `\n` line endings), in order.
fn fences(text: &str) -> Result<Vec<Fence>, String> {
    let mut found = Vec::new();
    let mut events = Parser::new(text).into_offset_iter();
    while let Some((event, range)) = events.next() {
        let Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) = event else {
            continue;
        };
        let mut content = String::new();
        for (event, _) in events.by_ref() {
            match event {
                Event::Text(text) => content.push_str(&text),
                Event::End(TagEnd::CodeBlock) => break,
                _ => {}
            }
        }
        let line_start = text[..range.start].rfind('\n').map_or(0, |at| at + 1);
        let line = text[..range.start].matches('\n').count() + 1;
        let before = &text[line_start..range.start];
        if !before.trim().is_empty() {
            return Err(format!(
                "line {line}: the fence shares its line with other text, so its block cannot be rewritten"
            ));
        }
        let open_end = text[range.start..range.end]
            .find('\n')
            .map_or(range.end, |at| range.start + at + 1);
        let block = text[..range.end].trim_end_matches('\n');
        let last_line = block.rfind('\n').map_or(0, |at| at + 1);
        let close = if last_line >= open_end && block[last_line..].trim_start().starts_with("```") {
            last_line
        } else {
            range.end
        };
        found.push(Fence {
            line,
            info: info.into_string(),
            content,
            span: open_end..close.max(open_end),
            indent: before.to_owned(),
        });
    }
    Ok(found)
}

/// `lines` as a block's content, each with the fence's indentation (none on a blank line).
fn content(indent: &str, lines: &[String]) -> String {
    lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                "\n".to_owned()
            } else {
                format!("{indent}{line}\n")
            }
        })
        .collect()
}

/// `text` with each fence's content replaced, the replacements given by fence index.
fn replaced(text: &str, fences: &[Fence], by_index: &[(usize, Vec<String>)]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    let mut ordered: Vec<&(usize, Vec<String>)> = by_index.iter().collect();
    ordered.sort_by_key(|(index, _)| *index);
    for (index, lines) in ordered {
        let fence = &fences[*index];
        out.push_str(&text[at..fence.span.start]);
        out.push_str(&content(&fence.indent, lines));
        at = fence.span.end;
    }
    out.push_str(&text[at..]);
    out
}

/// A page's source with its line endings kept: rewritten as `\n`, written back as the file had them.
fn with_endings(original: &str, text: String) -> String {
    if original.contains("\r\n") {
        text.replace('\n', "\r\n")
    } else {
        text
    }
}

/// What rewriting one page produced: its new source, and its mirror's when a mirror block was copied.
pub struct Rewritten {
    pub page: String,
    pub mirror: Option<String>,
    /// The rewritten blocks the mirror did not hold byte for byte, by line of the English block: they are left
    /// for a person, and the mirror guard names them.
    pub mirror_skipped: Vec<usize>,
}

/// Applies `rewrites` to the English page `source`, and to `mirror` where its block at the same position was
/// identical to the English block before the rewrite (§7: a mirror copies code blocks byte for byte).
pub fn apply(
    source: &str,
    mirror: Option<&str>,
    rewrites: &[Rewrite],
) -> Result<Rewritten, String> {
    let text = source.replace("\r\n", "\n");
    let english = fences(&text)?;
    let mut by_index = Vec::new();
    for rewrite in rewrites {
        let index = english
            .iter()
            .position(|fence| fence.line == rewrite.line)
            .ok_or_else(|| format!("line {}: no fenced block opens there", rewrite.line))?;
        by_index.push((index, rewrite.lines.clone()));
    }
    let page = with_endings(source, replaced(&text, &english, &by_index));
    let Some(mirror_source) = mirror else {
        return Ok(Rewritten {
            page,
            mirror: None,
            mirror_skipped: Vec::new(),
        });
    };
    let mirror_text = mirror_source.replace("\r\n", "\n");
    let translated = fences(&mirror_text)?;
    let (mut copied, mut skipped) = (Vec::new(), Vec::new());
    for (index, lines) in &by_index {
        let ours = &english[*index];
        match translated.get(*index) {
            Some(theirs) if theirs.info == ours.info && theirs.content == ours.content => {
                copied.push((*index, lines.clone()));
            }
            _ => skipped.push(ours.line),
        }
    }
    let mirror = (!copied.is_empty())
        .then(|| with_endings(mirror_source, replaced(&mirror_text, &translated, &copied)));
    Ok(Rewritten {
        page,
        mirror,
        mirror_skipped: skipped,
    })
}
