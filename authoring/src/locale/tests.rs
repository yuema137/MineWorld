use super::*;

#[test]
fn the_grammar_takes_language_script_and_region_and_refuses_the_rest() {
    for good in [
        "en",
        "fr",
        "zh_Hans",
        "pt_BR",
        "es_419",
        "sr_Latn_RS",
        "haw",
    ] {
        assert_eq!(Locale::canonical(good).expect(good).as_str(), good);
    }
    for bad in [
        "",
        "e",
        "EN",
        "english",
        "zh_hans",
        "zh_HANS",
        "pt_br",
        "zh-Hans",
        "zh_Hans_",
        "en_US_x",
        "en__US",
        "_en",
        "zh_Hans_Latn",
    ] {
        assert!(Locale::canonical(bad).is_err(), "{bad}");
    }
}

#[test]
fn a_typed_hyphen_is_normalised_and_a_bad_one_named_as_typed() {
    assert_eq!(Locale::parse("zh-Hans").expect("typed").as_str(), "zh_Hans");
    assert_eq!(Locale::parse("pt-BR").expect("typed").as_str(), "pt_BR");
    let refused = Locale::parse("zh-hans").expect_err("lower-case script");
    assert!(
        refused.to_string().starts_with("'zh-hans' is not a locale"),
        "{refused}"
    );
}

#[test]
fn a_reader_selects_its_locale_or_the_one_of_its_language_or_english() {
    let offered: Vec<Locale> = ["fr", "zh_Hans"]
        .into_iter()
        .map(|text| Locale::canonical(text).expect("valid"))
        .collect();
    let pick = |wanted: &str| {
        Locale::parse(wanted)
            .expect("valid")
            .select(&offered)
            .map(Locale::as_str)
    };
    assert_eq!(pick("zh_Hans"), Some("zh_Hans"));
    assert_eq!(
        pick("zh_CN"),
        Some("zh_Hans"),
        "the one offered of its language"
    );
    assert_eq!(pick("fr_CA"), Some("fr"));
    assert_eq!(pick("de"), None, "not offered: English");
    assert_eq!(pick("en"), None, "English is the declarations");
    let two: Vec<Locale> = ["zh_Hans", "zh_Hant"]
        .into_iter()
        .map(|text| Locale::canonical(text).expect("valid"))
        .collect();
    assert_eq!(
        Locale::parse("zh_CN").expect("valid").select(&two),
        None,
        "two of the language: no guess"
    );
}
