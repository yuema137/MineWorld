//! Meals and sleep move needs, driven through the real `eat` and `sleep` dispatch (SD-Nb-6; AC-Nb-2 in
//! a world; §4.7's edge cases).

mod support;

use mineworld_consumption::Drink;
use mineworld_needs::Mode;
use support::{Kind, Town, accepted, hour, record};

/// One person's AC-Nb-2 script: read at 00:00 and 05:00, eat one `food` at 05:00, sleep at 06:00 for
/// nine hours, read at 14:00 (still asleep). The literal expected values, computed by hand in the
/// comments below, and the words at 14:00.
fn script(town: &mut Town, who: &str, food: &str) -> [(u64, u64); 4] {
    let start = town.levels(who);
    town.pass_to(hour(5));
    let before = town.levels(who);
    assert!(accepted(&town.eat(who, food)), "{who} eats {food}");
    let fed = town.levels(who);
    town.pass_to(hour(6));
    assert!(accepted(&town.sleep(who, 540)), "{who} sleeps");
    town.pass_to(hour(14));
    [start, before, fed, town.levels(who)]
}

#[test]
fn flat_and_cafe_levels_equal_the_hand_computed_values() {
    let mut town = Town::new(Kind::FlatAndCafe);
    // ana: 300 + 120 × 5 = 900, 700 + 60 × 5 = 1 060 → 1 000; soup 600 → 300; 06:00 → 420, asleep;
    // 14:00: 420 + 30 × 8 = 660; 1 000 recovering 8 h at 125 per hour at 300 = 35 (Q-Nc3-5).
    assert_eq!(
        script(&mut town, "ana", "soup"),
        [(300, 700), (900, 1_000), (300, 1_000), (660, 35)]
    );
    let record = town.disclosed("ana", "ana").expect("disclosed");
    assert_eq!(record["words"]["hunger"], "hungry");
    assert_eq!(record["words"]["tiredness"], "ok");
    assert_eq!(record["asleep"], true);
    assert_eq!(town.needs("ana").expect("needs").mode(), Mode::Asleep);
}

#[test]
fn night_shift_levels_use_the_actor_scoped_rates_and_thresholds() {
    let mut town = Town::new(Kind::NightShift);
    // dee (night: 90 / 30 / 60 / 200): 300 + 90 × 5 = 750, 1 000; stew 500 → 250; 06:00 → 340;
    // 14:00: 340 + 30 × 8 = 580, 1 000 recovering 8 h at 200 per hour at 300 → 4. Thresholds [300, 500, 800]: hungry, ok.
    assert_eq!(
        script(&mut town, "dee", "stew"),
        [(300, 700), (750, 1_000), (250, 1_000), (580, 4)]
    );
    let record = town.disclosed("dee", "dee").expect("disclosed");
    assert_eq!(record["words"]["hunger"], "hungry");
    assert_eq!(record["words"]["tiredness"], "ok");
    assert_eq!(record["rates"]["rest_per_hour"], 200);

    let mut town = Town::new(Kind::NightShift);
    // eli (defaults): 900, 1 000; bread 300 → 600; 06:00 → 720; 14:00: 720 + 240 = 960, 35. Famished
    // under [300, 500, 800].
    assert_eq!(
        script(&mut town, "eli", "bread"),
        [(300, 700), (900, 1_000), (600, 1_000), (960, 35)]
    );
    let record = town.disclosed("eli", "eli").expect("disclosed");
    assert_eq!(record["words"]["hunger"], "famished");
}

#[test]
fn waking_turns_the_mode_back_and_tiredness_rises_again() {
    let mut town = Town::new(Kind::FlatAndCafe);
    town.pass_to(hour(1));
    assert!(accepted(&town.sleep("ana", 600)));
    town.pass_to(hour(3));
    assert!(accepted(&town.wake("ana")));
    // 01:00 → (420, 760); asleep two hours, recovering in proportion (Q-Nc3-5) → (480, 330); awake one
    // hour → (600, 390).
    assert_eq!(town.levels("ana"), (480, 330));
    town.pass_to(hour(4));
    assert_eq!(town.levels("ana"), (600, 390));
    assert_eq!(town.needs("ana").expect("needs").mode(), Mode::Awake);
}

#[test]
fn a_fact_that_changes_nothing_writes_nothing() {
    let mut town = Town::new(Kind::FlatAndCafe);
    town.pass_to(hour(2));
    let ana = town.needs("ana");
    // Coffee has no nourishment: drinking it stores nothing.
    let coffee = town.item("coffee");
    let (drunk, _) = town.submit(town.id("ana"), record(&Drink::new(coffee)));
    assert!(accepted(&drunk));
    assert_eq!(
        town.needs("ana"),
        ana,
        "a drink that nourishes 0 writes nothing"
    );
    // cy has no needs: eating, sleeping in the cafe or not, changes nothing of this pack's.
    assert!(accepted(&town.eat("cy", "soup")));
    assert!(town.needs("cy").is_none());
    assert!(town.disclosed("cy", "cy").is_none());
}

#[test]
fn eating_asleep_lowers_hunger_and_keeps_the_sleeper_asleep() {
    let mut town = Town::new(Kind::FlatAndCafe);
    town.pass_to(hour(1));
    assert!(accepted(&town.sleep("ben", 600)));
    town.pass_to(hour(2));
    // ben: 900 + 120 = 1 000 (clamped) at 01:00; asleep an hour: 1 000 + 30 → 1 000; apple 200 → 800.
    assert!(accepted(&town.eat("ben", "apple")));
    assert_eq!(town.levels("ben").0, 800);
    assert_eq!(town.needs("ben").expect("needs").mode(), Mode::Asleep);
}
