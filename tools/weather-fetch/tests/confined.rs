//! `INV-TW-7` (step-19 criterion 4): the HTTP client is confined to this tool. Read from `Cargo.lock`,
//! which lists every package's dependencies whatever their kind (normal, dev, build, optional), so a
//! runtime crate — or a test of one — that reached `ureq` directly or through this tool would show here.
//! The `cargo tree` commands of §18.4 C3 (d) give the same answer for normal edges; this test keeps it
//! in CI.

use std::collections::BTreeMap;

/// Each package in the lock file and the names it depends on.
fn lock() -> BTreeMap<String, Vec<String>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let text = std::fs::read_to_string(&path).expect("Cargo.lock is readable");
    let mut packages = BTreeMap::new();
    for block in text.split("[[package]]").skip(1) {
        let mut name = String::new();
        let mut dependencies = Vec::new();
        let mut in_dependencies = false;
        for line in block.lines().map(str::trim) {
            if let Some(value) = line.strip_prefix("name = ") {
                name = value.trim_matches('"').to_owned();
            } else if line.starts_with("dependencies = [") {
                in_dependencies = true;
            } else if in_dependencies && line.starts_with(']') {
                in_dependencies = false;
            } else if in_dependencies {
                // `"name"` or `"name version"` or `"name version (source)"`.
                let entry = line.trim_end_matches(',').trim_matches('"');
                dependencies.push(entry.split(' ').next().unwrap_or(entry).to_owned());
            }
        }
        packages
            .entry(name)
            .or_insert_with(Vec::new)
            .extend(dependencies);
    }
    packages
}

fn dependents(packages: &BTreeMap<String, Vec<String>>, of: &str) -> Vec<String> {
    packages
        .iter()
        .filter(|(_, dependencies)| dependencies.iter().any(|name| name == of))
        .map(|(name, _)| name.clone())
        .collect()
}

#[test]
fn only_the_weather_tool_depends_on_an_http_client_and_nothing_depends_on_the_tool() {
    let packages = lock();
    assert!(
        packages.contains_key("ureq"),
        "the lock lists ureq (the tool's optional dependency)"
    );
    assert_eq!(dependents(&packages, "ureq"), ["mineworld-weather-fetch"]);
    assert_eq!(
        dependents(&packages, "mineworld-weather-fetch"),
        Vec::<String>::new()
    );
}
