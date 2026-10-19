//! Reading one guide page: its executed blocks, the directives that qualify them, and the step each one
//! sits in (`docs/GUIDE_FORMAT.md` §§2, 4.1, 6, 10). A page that breaks the grammar or a checked page rule
//! is refused here, naming the line, before anything is written or run.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};

use super::files;

/// A directive that qualifies the executed block after it (§6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Directive {
    /// The block waits for this feature (§8.1).
    Requires(String),
    /// The block needs a live model or a display and is never run in CI (§6).
    OperatorOnly,
    /// After the block, the scratch folder `folder` equals the committed folder `committed` (§6.1).
    Equals { folder: String, committed: String },
    /// Add `--packs DIR` to each command of the block (§6).
    Packs(String),
}

/// What the runner does with an executed block (§§3, 4, 6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    /// Write `text` to `path` inside the scratch folder (§3).
    File { path: String, text: String },
    /// Run the commands of `lines` and match their output (§4). The lines are split into commands and
    /// expected output when the block runs.
    Console { exit: i32, lines: Vec<String> },
    /// Run `text` with the shell of `family`, in the scratch folder, on the operating systems that use that
    /// shell (§4.6). Its output is shown on failure and not matched; it must exit 0.
    Shell { family: Family, text: String },
}

/// The shell a `sh` or `powershell` block is written for (§4.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// `sh`: run on macOS and Linux.
    Posix,
    /// `powershell`: run on Windows.
    PowerShell,
}

impl Family {
    /// Whether this test process runs on an operating system that uses this shell.
    pub fn is_this_os(self) -> bool {
        match self {
            Self::Posix => !cfg!(windows),
            Self::PowerShell => cfg!(windows),
        }
    }

    /// The block's language word, as the page writes it.
    pub fn word(self) -> &'static str {
        match self {
            Self::Posix => "sh",
            Self::PowerShell => "powershell",
        }
    }
}

/// One executed block, in page order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The 1-based line of the block's opening fence.
    pub line: usize,
    /// The text of the nearest level-3 heading above the block, if there is one.
    pub step: Option<String>,
    /// The directives that qualify this block, each kind at most once.
    pub directives: Vec<Directive>,
    /// What the block does.
    pub body: Body,
}

/// A parsed page: its executed blocks in order, and its `continues` directive (§5, §6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    /// The page whose scratch folder this page reuses, if it declares `continues`.
    pub continues: Option<String>,
    /// The executed blocks. A block that is shown only is not here (§2 rule 3).
    pub blocks: Vec<Block>,
}

/// Parses a page. An error names the line and what the grammar refused.
pub fn parse(source: &str) -> Result<Page, String> {
    let text = source.replace("\r\n", "\n");
    let mut reader = Reader {
        text: &text,
        page: Page {
            continues: None,
            blocks: Vec::new(),
        },
        step: None,
        tally: None,
        pending: Vec::new(),
        seen_block: false,
    };
    reader.run()?;
    reader.close_step()?;
    if let Some((line, _)) = reader.pending.first() {
        return Err(format!(
            "line {line}: a directive has no executed block after it to qualify"
        ));
    }
    Ok(reader.page)
}

/// A directive as its comment spells it, before it is attached to a block.
enum Parsed {
    Continues(String),
    Attach(Directive),
}

/// What one `### Step …` has held so far, for the page rules of §10 the runner checks.
struct Tally {
    /// The line of the step's heading.
    line: usize,
    /// The heading's text.
    title: String,
    /// The `file=` blocks in the step (§10 rule 1: at most one).
    files: usize,
    /// Whether the step has a command block, or a block it hands to the operator (§10 rule 3).
    checked: bool,
}

struct Reader<'a> {
    text: &'a str,
    page: Page,
    step: Option<String>,
    /// The step being read, when the nearest heading above is a `### Step …` heading.
    tally: Option<Tally>,
    /// Directives waiting for the next executed block, with the line each was written on.
    pending: Vec<(usize, Directive)>,
    /// Whether any block (executed or shown) has been read: `continues` must precede it.
    seen_block: bool,
}

impl Reader<'_> {
    fn run(&mut self) -> Result<(), String> {
        let mut events = Parser::new(self.text).into_offset_iter();
        let mut heading: Option<String> = None;
        while let Some((event, range)) = events.next() {
            let line = self.line(range.start);
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    // Any heading of level 3 or above ends the step being read (§10).
                    if level <= HeadingLevel::H3 {
                        self.close_step()?;
                    }
                    if level == HeadingLevel::H3 {
                        heading = Some(String::new());
                    }
                }
                Event::End(TagEnd::Heading(HeadingLevel::H3)) => {
                    self.step = heading.take().map(|text| text.trim().to_owned());
                    self.tally = self
                        .step
                        .as_ref()
                        .filter(|title| title.starts_with("Step "))
                        .map(|title| Tally {
                            line,
                            title: title.clone(),
                            files: 0,
                            checked: false,
                        });
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some(heading) = heading.as_mut() {
                        heading.push_str(&text);
                    }
                }
                Event::Html(html) | Event::InlineHtml(html) => self.comments(&html, line)?,
                Event::Start(Tag::CodeBlock(kind)) => {
                    let info = match kind {
                        CodeBlockKind::Fenced(info) => info.into_string(),
                        CodeBlockKind::Indented => {
                            return Err(self.at(
                                line,
                                "an indented code block; a guide page uses a fence of backticks",
                            ));
                        }
                    };
                    if self.text[range.start..].trim_start().starts_with('~') {
                        return Err(
                            self.at(line, "a tilde fence (~~~); a guide page uses backticks")
                        );
                    }
                    let mut content = String::new();
                    for (event, _) in events.by_ref() {
                        match event {
                            Event::Text(text) => content.push_str(&text),
                            Event::End(TagEnd::CodeBlock) => break,
                            _ => {}
                        }
                    }
                    self.code_block(line, &info, &content)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Reads each `<!-- guide: … -->` comment on its own line of an HTML event. Other HTML is ignored.
    fn comments(&mut self, html: &str, line: usize) -> Result<(), String> {
        for (offset, text) in html.lines().enumerate() {
            let text = text.trim();
            let comment = text
                .strip_prefix("<!--")
                .and_then(|inner| inner.strip_suffix("-->"))
                .map(str::trim);
            if let Some(directive) = comment.and_then(|inner| inner.strip_prefix("guide:")) {
                self.directive(directive.trim(), line + offset)?;
            }
        }
        Ok(())
    }

    fn directive(&mut self, text: &str, line: usize) -> Result<(), String> {
        let mut words = text.split_whitespace();
        let name = words.next().unwrap_or_default();
        let arguments: Vec<&str> = words.collect();
        let parsed = match (name, arguments.as_slice()) {
            ("continues", [page]) => Parsed::Continues((*page).to_owned()),
            ("requires", [feature]) => Parsed::Attach(Directive::Requires((*feature).to_owned())),
            ("operator-only", []) => Parsed::Attach(Directive::OperatorOnly),
            ("equals", [folder, committed]) => {
                for path in [folder, committed] {
                    files::confine(path).map_err(|reason| self.at(line, reason))?;
                }
                Parsed::Attach(Directive::Equals {
                    folder: (*folder).to_owned(),
                    committed: (*committed).to_owned(),
                })
            }
            ("packs", [dir]) => {
                files::confine(dir).map_err(|reason| self.at(line, reason))?;
                Parsed::Attach(Directive::Packs((*dir).to_owned()))
            }
            _ => {
                return Err(self.at(
                    line,
                    format!("guide directive {text:?} is not one the runner knows, or has the wrong arguments"),
                ));
            }
        };
        match parsed {
            Parsed::Continues(page) => {
                if self.seen_block || self.page.continues.is_some() {
                    return Err(
                        self.at(line, "continues must be given once, before the first block")
                    );
                }
                self.page.continues = Some(page);
            }
            Parsed::Attach(directive) => {
                if self.pending.iter().any(|(_, held)| {
                    std::mem::discriminant(held) == std::mem::discriminant(&directive)
                }) {
                    return Err(self.at(line, "two directives of one kind qualify the same block"));
                }
                self.pending.push((line, directive));
            }
        }
        Ok(())
    }

    fn code_block(&mut self, line: usize, info: &str, content: &str) -> Result<(), String> {
        self.seen_block = true;
        let words: Vec<&str> = info.split_whitespace().collect();
        let (language, options) = match words.first() {
            Some(first) if !first.contains('=') => (*first, &words[1..]),
            _ => ("", words.as_slice()),
        };
        let mut file = None;
        let mut exit = None;
        for word in options {
            if let Some(path) = word.strip_prefix("file=") {
                if file.replace(path).is_some() {
                    return Err(self.at(line, "file= is given twice in one info string"));
                }
            } else if let Some(code) = word.strip_prefix("exit=") {
                if exit.replace(code).is_some() {
                    return Err(self.at(line, "exit= is given twice in one info string"));
                }
            } else {
                return Err(self.at(
                    line,
                    format!("info-string token {word:?} is not one the guide understands"),
                ));
            }
        }
        let console = language == "console";
        if file.is_some() && console {
            return Err(self.at(
                line,
                "a block with file= is a file block, not a console block",
            ));
        }
        if exit.is_some() && !console {
            return Err(self.at(line, "exit= is valid on console blocks only"));
        }
        let body = if let Some(path) = file {
            files::confine(path).map_err(|reason| self.at(line, reason))?;
            Body::File {
                path: path.to_owned(),
                text: content.to_owned(),
            }
        } else if let Some(family) = match language {
            "sh" => Some(Family::Posix),
            "powershell" => Some(Family::PowerShell),
            _ => None,
        } {
            if content.trim().is_empty() {
                return Err(self.at(line, format!("a {language} block has no command")));
            }
            Body::Shell {
                family,
                text: content.to_owned(),
            }
        } else if console {
            let lines: Vec<String> = content.lines().map(str::to_owned).collect();
            let exit = match exit {
                None => 0,
                Some(code) => code
                    .parse::<i32>()
                    .ok()
                    .filter(|code| (0..=255).contains(code))
                    .ok_or_else(|| self.at(line, format!("exit={code} is not an exit code")))?,
            };
            check_console(&lines).map_err(|reason| self.at(line, reason))?;
            Body::Console { exit, lines }
        } else {
            // Shown only: a block the runner never executes or reads (§2 rule 3).
            return Ok(());
        };
        let directives = self.take_pending(line, &body)?;
        if let Some(tally) = self.tally.as_mut() {
            match body {
                Body::File { .. } => tally.files += 1,
                Body::Console { .. } => tally.checked = true,
                Body::Shell { .. } => {}
            }
            if directives.contains(&Directive::OperatorOnly) {
                tally.checked = true;
            }
            if tally.files > 1 {
                let title = tally.title.clone();
                return Err(self.at(
                    line,
                    format!(
                        "step {title:?} writes a second file; one action per step, so a second file is a step of its own (§10 rule 1)"
                    ),
                ));
            }
        }
        self.page.blocks.push(Block {
            line,
            step: self.step.clone(),
            directives,
            body,
        });
        Ok(())
    }

    /// The pending directives, once each is checked against the kind of block it qualifies (§6).
    fn take_pending(&mut self, line: usize, body: &Body) -> Result<Vec<Directive>, String> {
        for (held, directive) in &self.pending {
            let allowed = match (directive, body) {
                (Directive::Packs(_), Body::Console { .. }) => true,
                (Directive::Packs(_), _) => false,
                (Directive::Equals { .. }, _) => true,
                (Directive::Requires(_) | Directive::OperatorOnly, _) => true,
            };
            if !allowed {
                return Err(self.at(
                    *held,
                    format!("directive {directive:?} does not qualify the block at line {line}"),
                ));
            }
        }
        Ok(std::mem::take(&mut self.pending)
            .into_iter()
            .map(|(_, directive)| directive)
            .collect())
    }

    /// Ends the step being read: it must have shown a command and its output, or handed its block to the
    /// operator (§10 rule 3).
    fn close_step(&mut self) -> Result<(), String> {
        match self.tally.take() {
            Some(tally) if !tally.checked => Err(self.at(
                tally.line,
                format!(
                    "step {:?} has no console block and no operator-only block; every step ends with a command and its output (§10 rule 3)",
                    tally.title
                ),
            )),
            _ => Ok(()),
        }
    }

    fn line(&self, offset: usize) -> usize {
        self.text[..offset].matches('\n').count() + 1
    }

    fn at(&self, line: usize, reason: impl std::fmt::Display) -> String {
        format!("line {line}: {reason}")
    }
}

/// A console block is commands and the output they print: blank lines and output may come only after a
/// command line, and the block must have at least one command (§4.1).
fn check_console(lines: &[String]) -> Result<(), String> {
    let Some(first) = lines.iter().position(|line| line.starts_with("$ ")) else {
        return Err("a console block has no command line (a line starting with \"$ \")".to_owned());
    };
    match lines[..first].iter().find(|line| !line.trim().is_empty()) {
        Some(line) => Err(format!(
            "output {line:?} comes before the first command line"
        )),
        None => Ok(()),
    }
}
