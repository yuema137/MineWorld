//! A small GDScript lexer shared by the client scans (`client_rules.rs`, I-S14-1; `client_text.rs`, S20
//! SET-a). Moved out of `client_rules.rs` unchanged (step-20 SD-SET-a-13): it separates code, string
//! literals and comments, so a scan reads what a script does and not what its comments say.

/// One GDScript file, lexed: its code with comments removed and strings blanked, line by line, and its
/// string literals with the line each starts on.
#[derive(Debug, Default, PartialEq)]
pub struct Lexed {
    pub code: Vec<String>,
    pub literals: Vec<(usize, String)>,
}

/// Separate code, string literals and comments. `#` outside a string starts a comment to the end of
/// the line; `"…"`, `'…'` and `"""…"""` are strings, with backslash escapes; a `&` or `^` prefix
/// (StringName, NodePath) is code and the string after it a string.
pub fn lex(text: &str) -> Lexed {
    let mut out = Lexed {
        code: vec![String::new()],
        literals: Vec::new(),
    };
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut line = 1;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            out.code.push(String::new());
            line += 1;
            i += 1;
        } else if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if c == '"' || c == '\'' {
            let triple = i + 2 < chars.len() && chars[i + 1] == c && chars[i + 2] == c;
            let start_line = line;
            let mut literal = String::new();
            i += if triple { 3 } else { 1 };
            loop {
                if i >= chars.len() {
                    break;
                }
                let d = chars[i];
                if d == '\\' && i + 1 < chars.len() {
                    literal.push(chars[i + 1]);
                    if chars[i + 1] == '\n' {
                        out.code.push(String::new());
                        line += 1;
                    }
                    i += 2;
                    continue;
                }
                if d == c
                    && (!triple || (i + 2 < chars.len() && chars[i + 1] == c && chars[i + 2] == c))
                {
                    i += if triple { 3 } else { 1 };
                    break;
                }
                if d == '\n' {
                    out.code.push(String::new());
                    line += 1;
                }
                literal.push(d);
                i += 1;
            }
            out.code.last_mut().expect("a line").push_str("\"\"");
            out.literals.push((start_line, literal));
        } else {
            out.code.last_mut().expect("a line").push(c);
            i += 1;
        }
    }
    out
}
