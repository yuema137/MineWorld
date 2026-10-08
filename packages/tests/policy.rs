//! The licence policy: the default, how an expression is judged, and the typed value a world will
//! override (`DECISIONS.md` `ARC-55`; step-16 §15.0 FQ-b2).

use mineworld_packages::{LicencePolicy, License};

fn judge(policy: &LicencePolicy, expression: &str) -> Result<(), String> {
    policy
        .judge("a-pack", &License::new(expression).expect("spdx"))
        .map_err(|refusal| refusal.to_string())
}

#[test]
fn the_default_allows_the_permissive_set_and_judges_whole_expressions() {
    let policy = LicencePolicy::default();
    for allowed in [
        "MIT",
        "Apache-2.0",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "ISC",
        "Zlib",
        "CC0-1.0",
        "Unlicense",
        "MIT OR Apache-2.0",
        "MIT OR GPL-3.0-only",
        "MIT AND CC0-1.0",
    ] {
        judge(&policy, allowed).unwrap_or_else(|refusal| panic!("{allowed}: {refusal}"));
    }
    for (refused, failing) in [
        ("GPL-3.0-only", "GPL-3.0-only"),
        ("MIT AND GPL-3.0-only", "GPL-3.0-only"),
        ("CC-BY-4.0", "CC-BY-4.0"),
        ("CC-BY-SA-4.0", "CC-BY-SA-4.0"),
        (
            "Apache-2.0 WITH LLVM-exception",
            "Apache-2.0 WITH LLVM-exception",
        ),
        ("Apache-2.0+", "Apache-2.0+"),
    ] {
        let refusal = judge(&policy, refused).expect_err(refused);
        assert!(
            refusal.contains("a-pack") && refusal.contains(failing) && refusal.contains("CC0-1.0"),
            "{refused}: {refusal}"
        );
    }
}

/// The hook FQ-b2 asks for: the policy is a value a world's `configure/packages.yaml` will decode into
/// (S17's seam), narrowing or extending the default. Decoded here from the YAML it will be written in.
#[test]
fn a_world_can_state_its_own_policy_as_a_typed_value() {
    let narrowed: LicencePolicy = serde_saphyr::from_str("allowed: [MIT]\n").expect("decodes");
    judge(&narrowed, "MIT").expect("still allowed");
    assert!(judge(&narrowed, "Apache-2.0").is_err(), "narrowed out");

    let extended: LicencePolicy =
        serde_saphyr::from_str("allowed: [MIT, CC-BY-4.0]\n").expect("decodes");
    judge(&extended, "CC-BY-4.0").expect("extended in");

    for (text, needle) in [
        ("allowed: [Not-A-Licence]\n", "Not-A-Licence"),
        ("allowed: [MIT]\nalso: x\n", "also"),
    ] {
        let refusal = serde_saphyr::from_str::<LicencePolicy>(text)
            .expect_err(text)
            .to_string();
        assert!(refusal.contains(needle), "{refusal}");
    }
}
