//! A scratch World Pack's `world.yaml`, edited by a test as text: `configure:` widened by keys.
//!
//! A reference world may already configure packs (`ARC-269`: `social-cafe` configures `calendar`), so a
//! test that appends a second `configure:` writes a duplicate key, which the loader refuses. This adds
//! the keys to the manifest's own top-level list when it has one, and writes the list when it has none,
//! whatever the reference world configures today.

/// `manifest` with every key of `keys` listed under its top-level `configure:`, once each, after the
/// keys already there. Line endings are read as `\n` or `\r\n` and written as `\n`.
///
/// ```
/// let with = mineworld_test_support::manifest::configure_also("systems:\n  - a\n", &["b"]);
/// assert_eq!(with, "systems:\n  - a\n\nconfigure:\n  - b\n");
/// let widened = mineworld_test_support::manifest::configure_also(&with, &["c", "b"]);
/// assert_eq!(widened, "systems:\n  - a\n\nconfigure:\n  - b\n  - c\n");
/// ```
pub fn configure_also(manifest: &str, keys: &[&str]) -> String {
    let text = manifest.replace("\r\n", "\n");
    let lines: Vec<&str> = text.lines().collect();
    let Some(start) = lines
        .iter()
        .position(|line| line.trim_end() == "configure:")
    else {
        let mut out = text;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("\nconfigure:\n");
        for key in unique(keys, &[]) {
            out.push_str(&format!("  - {key}\n"));
        }
        return out;
    };
    // The list is the run of indented `- ` items (and comments) right after the key.
    let mut end = start + 1;
    let mut listed = Vec::new();
    while let Some(line) = lines.get(end) {
        let trimmed = line.trim_start();
        if line.starts_with(' ') && (trimmed.starts_with("- ") || trimmed.starts_with('#')) {
            if let Some(item) = trimmed.strip_prefix("- ") {
                listed.push(item.trim().to_owned());
            }
            end += 1;
        } else {
            break;
        }
    }
    let listed: Vec<&str> = listed.iter().map(String::as_str).collect();
    let mut out: Vec<String> = lines[..end].iter().map(|line| (*line).to_owned()).collect();
    out.extend(
        unique(keys, &listed)
            .into_iter()
            .map(|key| format!("  - {key}")),
    );
    out.extend(lines[end..].iter().map(|line| (*line).to_owned()));
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

/// The keys of `keys` not in `listed`, first occurrence only, in order.
fn unique(keys: &[&str], listed: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for key in keys {
        if !listed.contains(key) && !out.iter().any(|seen| seen == key) {
            out.push((*key).to_owned());
        }
    }
    out
}

/// `manifest` with the top-level lines of `extra` added: a `configure:` list in `extra` widens the
/// manifest's own through [`configure_also`], and every other line is appended as written.
///
/// ```
/// let base = "systems:\n  - a\n\nconfigure:\n  - a\n";
/// let with = mineworld_test_support::manifest::extended(base, "configure:\n  - b\nother: x\n");
/// assert_eq!(with, "systems:\n  - a\n\nconfigure:\n  - a\n  - b\nother: x\n");
/// ```
pub fn extended(manifest: &str, extra: &str) -> String {
    let extra = extra.replace("\r\n", "\n");
    let mut keys = Vec::new();
    let mut rest = String::new();
    let mut in_configure = false;
    for line in extra.lines() {
        if line.trim_end() == "configure:" {
            in_configure = true;
            continue;
        }
        if in_configure {
            if let Some(item) = line
                .trim_start()
                .strip_prefix("- ")
                .filter(|_| line.starts_with(' '))
            {
                keys.push(item.trim().to_owned());
                continue;
            }
            in_configure = false;
        }
        rest.push_str(line);
        rest.push('\n');
    }
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    let mut out = if keys.is_empty() {
        manifest.replace("\r\n", "\n")
    } else {
        configure_also(manifest, &keys)
    };
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&rest);
    out
}
