//! Contract tests for the spatial vocabulary and the one evaluator that reads it.
//!
//! These are the tests that decide whether `ENGINEERING_RULES.md` §§5–9 landed: one `Location`
//! type describes both an embodied position and a purely semantic one, the evaluator is the only
//! place a spatial rule is decided, and no client has to implement any of it.
//!
//! Every expected value is written by hand. Distances are chosen so that the arithmetic can be
//! checked on paper: 3-4-5 triangles, and the extremes of the representable range.

use mineworld_contracts::{
    ContractError, EntityId, EntityType, LocalPosition, Location, Millidegrees, Millimetres,
    Orientation, PlaceId, PlaceRequirement, Rejection, SpatialRequirement,
};

fn cafe() -> PlaceId {
    PlaceId::new(EntityId::from_raw(7), EntityType::Place).unwrap()
}

fn park() -> PlaceId {
    PlaceId::new(EntityId::from_raw(8), EntityType::Place).unwrap()
}

/// A location with a position, in millimetres from the place's origin.
fn at(place: PlaceId, x: i32, y: i32) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

/// The adversarial criterion of §3 of the design, stated as a test: one type must describe both
/// "standing 1.2 m from Alice, facing her, inside the café" and "in the café, position
/// irrelevant". If those needed two types, a 2D world and a 3D world would need two spatial
/// models, and every system would have to be written twice.
#[test]
fn one_location_type_describes_both_an_embodied_and_a_purely_semantic_position() {
    let embodied = Location::in_place(cafe())
        .with_local(LocalPosition::new(
            Millimetres::new(1_200),
            Millimetres::ZERO,
            Millimetres::new(1_650),
        ))
        .with_facing(
            Orientation::new(Millidegrees::new(90_000), Some(Millidegrees::new(-5_000)))
                .expect("5 degrees below the horizon is a legal pitch"),
        );
    let semantic = Location::in_place(cafe());

    assert_eq!(embodied.place(), semantic.place());
    assert_eq!(
        embodied.local().map(LocalPosition::z),
        Some(Millimetres::new(1_650))
    );
    assert_eq!(
        embodied.facing().and_then(Orientation::pitch),
        Some(Millidegrees::new(-5_000))
    );
    assert_eq!(semantic.local(), None);
    assert_eq!(semantic.facing(), None);

    // Both survive the transport unchanged, and the semantic one does not acquire a position on
    // the way: a world that models no geometry must not be given one by its own contract layer.
    let text = serde_json::to_string(&semantic).unwrap();
    assert_eq!(
        text,
        r#"{"place":{"entity":7,"entity_type":"place"},"local":null,"facing":null}"#
    );
    assert_eq!(serde_json::from_str::<Location>(&text).unwrap(), semantic);
    let text = serde_json::to_string(&embodied).unwrap();
    assert_eq!(
        text,
        r#"{"place":{"entity":7,"entity_type":"place"},"local":{"x":1200,"y":0,"z":1650},"facing":{"yaw":90000,"pitch":-5000}}"#
    );
    assert_eq!(serde_json::from_str::<Location>(&text).unwrap(), embodied);
}

/// A heading is canonicalized, because a full turn is not a different direction and two
/// representations of one direction in the event log would make two identical worlds compare
/// unequal. A pitch is *rejected* instead, because past straight up there is no steeper direction:
/// clamping would turn an impossible value into a plausible one and hide the mistake.
#[test]
fn a_heading_is_canonicalized_and_an_impossible_pitch_is_refused() {
    let quarter_turn = Millidegrees::new(90_000);
    for equivalent in [90_000, 450_000, -270_000, 360_090_000] {
        assert_eq!(
            Orientation::facing(Millidegrees::new(equivalent)).yaw(),
            quarter_turn,
            "{equivalent}mdeg names the same direction as 90000mdeg"
        );
    }
    assert_eq!(
        Orientation::facing(Millidegrees::new(360_000)).yaw(),
        Millidegrees::ZERO
    );
    assert_eq!(
        Orientation::facing(Millidegrees::new(-1)).yaw(),
        Millidegrees::new(359_999)
    );

    assert_eq!(
        Orientation::new(Millidegrees::new(0), Some(Millidegrees::new(90_000)))
            .expect("straight up is the steepest legal pitch")
            .pitch(),
        Some(Millidegrees::new(90_000))
    );
    assert_eq!(
        Orientation::new(Millidegrees::new(0), Some(Millidegrees::new(90_001))),
        Err(ContractError::PitchOutOfRange {
            millidegrees: 90_001,
            limit: 90_000,
        })
    );
    assert_eq!(
        Orientation::new(Millidegrees::new(0), Some(Millidegrees::new(-90_001))),
        Err(ContractError::PitchOutOfRange {
            millidegrees: -90_001,
            limit: 90_000,
        })
    );

    // A recorded orientation is read back through the same check, so a hand-edited save cannot
    // reintroduce either fault.
    assert_eq!(
        serde_json::from_str::<Orientation>(r#"{"yaw":450000,"pitch":null}"#)
            .unwrap()
            .yaw(),
        quarter_turn
    );
    let error = serde_json::from_str::<Orientation>(r#"{"yaw":0,"pitch":120000}"#)
        .expect_err("an impossible pitch must not be readable");
    assert_eq!(
        error.to_string(),
        ContractError::PitchOutOfRange {
            millidegrees: 120_000,
            limit: 90_000,
        }
        .to_string()
    );
}

/// The whole documented case matrix of the evaluator, which is the only place in MineWorld where a
/// spatial rule is decided. Each row is a rejection a client must be able to show, or a pass.
#[test]
fn the_evaluator_answers_every_documented_spatial_case() {
    let none = SpatialRequirement::NONE;
    let same_place = SpatialRequirement::same_place();
    let within_two_metres = SpatialRequirement::same_place()
        .within(Millimetres::new(2_000))
        .expect("two metres is a legal range");
    let at_the_cafe = SpatialRequirement::at_place(cafe());

    let actor = at(cafe(), 0, 0);
    let near = at(cafe(), 1_200, 1_600); // 2.0 m away exactly: a 3-4-5 triangle scaled by 400.
    let far = at(cafe(), 1_200, 1_601); // one millimetre further.
    let elsewhere = at(park(), 0, 0);

    // Requires nothing of space: the case ENGINEERING_RULES §7 names for a message or a remote
    // application, and it must pass with no target at all.
    assert_eq!(none.evaluate(&actor, None, false), Ok(()));
    assert_eq!(none.evaluate(&actor, Some(&elsewhere), false), Ok(()));

    // Same place, both ways.
    assert_eq!(same_place.evaluate(&actor, Some(&far), true), Ok(()));
    assert_eq!(
        same_place.evaluate(&actor, Some(&elsewhere), true),
        Err(Rejection::TooFarAway)
    );

    // In range and out of range, decided by exact integer arithmetic.
    assert_eq!(
        within_two_metres.evaluate(&actor, Some(&near), true),
        Ok(())
    );
    assert_eq!(
        within_two_metres.evaluate(&actor, Some(&far), true),
        Err(Rejection::TooFarAway)
    );

    // A specific place constrains the actor, wherever the target may be.
    assert_eq!(
        at_the_cafe.evaluate(&actor, Some(&elsewhere), false),
        Ok(())
    );
    assert_eq!(
        at_the_cafe.evaluate(&elsewhere, Some(&actor), false),
        Err(Rejection::TooFarAway)
    );

    // A requirement about a target cannot be met by a request that named none, and no distance was
    // ever established — so this is a failed precondition, not a distance.
    assert_eq!(
        same_place.evaluate(&actor, None, true),
        Err(Rejection::PreconditionFailed)
    );
    assert_eq!(
        within_two_metres.evaluate(&actor, None, true),
        Err(Rejection::PreconditionFailed)
    );

    // An unavailable target is answered before distance, because walking closer cannot make a
    // target available and a client that showed the distance would send a player on a pointless
    // journey.
    let requires_available = SpatialRequirement::same_place().requiring_target_available();
    assert_eq!(
        requires_available.evaluate(&actor, Some(&far), true),
        Ok(())
    );
    assert_eq!(
        requires_available.evaluate(&actor, Some(&elsewhere), false),
        Err(Rejection::TargetUnavailable)
    );

    // Line of access is declared and not evaluated (DD-7): it must not silently reject, and it
    // must not silently claim to have checked anything either — the declaration is what a geometry
    // provider will read.
    let needs_line_of_access = SpatialRequirement::same_place().requiring_line_of_access();
    assert!(needs_line_of_access.requires_line_of_access());
    assert_eq!(
        needs_line_of_access.evaluate(&actor, Some(&far), true),
        Ok(())
    );
}

/// `DD-4` promises that a world leaving continuous position out "loses nothing". The consequence
/// for a range requirement is the interesting one: a 2D or headless world must still be able to use
/// an action a 3D-capable system declared, without a millimetre range either blocking everything or
/// silently reaching across the map.
#[test]
fn a_range_requirement_degenerates_to_the_same_place_when_no_position_is_modelled() {
    let within_two_metres = SpatialRequirement::same_place()
        .within(Millimetres::new(2_000))
        .expect("two metres is a legal range");

    let semantic_actor = Location::in_place(cafe());
    let semantic_target = Location::in_place(cafe());
    let semantic_elsewhere = Location::in_place(park());

    assert_eq!(
        within_two_metres.evaluate(&semantic_actor, Some(&semantic_target), true),
        Ok(())
    );
    assert_eq!(
        within_two_metres.evaluate(&semantic_actor, Some(&semantic_elsewhere), true),
        Err(Rejection::TooFarAway)
    );

    // Half a world: only one side has a position. Still the same answer, because a distance needs
    // both ends.
    assert_eq!(
        within_two_metres.evaluate(&semantic_actor, Some(&at(cafe(), 900_000, 0)), true),
        Ok(())
    );
    assert_eq!(
        within_two_metres.evaluate(&at(cafe(), 900_000, 0), Some(&semantic_elsewhere), true),
        Err(Rejection::TooFarAway)
    );

    // A place requirement never needed a position in the first place.
    assert_eq!(
        SpatialRequirement::same_place().evaluate(&semantic_actor, Some(&semantic_target), true),
        Ok(())
    );

    // The case that isolates the degeneracy, with no place clause to fall back on: a bare range,
    // which is how a system declares "near the target, wherever that is". Without the degeneracy
    // this would reach across the whole world in any world that models no position, and a
    // requirement that names three metres would silently mean none.
    let bare_range = SpatialRequirement::NONE
        .within(Millimetres::new(2_000))
        .expect("two metres is a legal range");
    assert_eq!(
        bare_range.evaluate(&semantic_actor, Some(&semantic_target), true),
        Ok(())
    );
    assert_eq!(
        bare_range.evaluate(&semantic_actor, Some(&semantic_elsewhere), true),
        Err(Rejection::TooFarAway)
    );
    // And with positions on both sides it is a distance again, not a place comparison.
    assert_eq!(
        bare_range.evaluate(&at(cafe(), 0, 0), Some(&at(cafe(), 1_200, 1_600)), true),
        Ok(())
    );
    assert_eq!(
        bare_range.evaluate(&at(cafe(), 0, 0), Some(&at(cafe(), 1_200, 1_601)), true),
        Err(Rejection::TooFarAway)
    );
}

/// The evaluator must be total at the edges of the representable range: two positions at opposite
/// ends of it are about 4 295 km apart, which does not fit in an `i32` of millimetres, and the
/// squares of those differences do not fit in an `i64` either. An overflow here would either panic
/// in a release build's wrapping arithmetic or answer that the two are adjacent.
#[test]
fn distance_arithmetic_survives_the_extremes_of_the_representable_range() {
    let one_kilometre = SpatialRequirement::same_place()
        .within(Millimetres::new(1_000_000))
        .expect("a kilometre is a legal range");

    let lowest = Location::in_place(cafe()).with_local(LocalPosition::new(
        Millimetres::new(i32::MIN),
        Millimetres::new(i32::MIN),
        Millimetres::new(i32::MIN),
    ));
    let highest = Location::in_place(cafe()).with_local(LocalPosition::new(
        Millimetres::new(i32::MAX),
        Millimetres::new(i32::MAX),
        Millimetres::new(i32::MAX),
    ));

    assert_eq!(
        one_kilometre.evaluate(&lowest, Some(&highest), true),
        Err(Rejection::TooFarAway)
    );
    assert_eq!(
        one_kilometre.evaluate(&highest, Some(&lowest), true),
        Err(Rejection::TooFarAway)
    );
    // And a point is always within range of itself, at either extreme.
    assert_eq!(one_kilometre.evaluate(&lowest, Some(&lowest), true), Ok(()));

    let nothing = SpatialRequirement::same_place()
        .within(Millimetres::ZERO)
        .expect("a range of zero is legal: it means exactly here");
    assert_eq!(nothing.evaluate(&highest, Some(&highest), true), Ok(()));
    assert_eq!(
        nothing.evaluate(&highest, Some(&at(cafe(), 0, 0)), true),
        Err(Rejection::TooFarAway)
    );
}

/// A requirement is a declaration a system ships, and a negative range is a mistake that must be
/// caught where it is written: the evaluator compares squared distances, so `-3 m` would otherwise
/// behave exactly like `3 m` and the declaration would quietly mean something else.
#[test]
fn a_negative_interaction_range_is_refused_where_it_is_declared() {
    assert_eq!(
        SpatialRequirement::same_place().within(Millimetres::new(-3_000)),
        Err(ContractError::NegativeInteractionRange {
            millimetres: -3_000
        })
    );

    let text = r#"{"place":"same_place_as_actor","within_range":-3000,"requires_line_of_access":false,"requires_target_available":false}"#;
    let error = serde_json::from_str::<SpatialRequirement>(text)
        .expect_err("a negative range must not be readable either");
    assert_eq!(
        error.to_string(),
        ContractError::NegativeInteractionRange {
            millimetres: -3_000
        }
        .to_string()
    );
}

/// The stored shape of a requirement, asserted exactly: it travels to a client inside an
/// affordance, so a client that is not written in Rust reads exactly this.
#[test]
fn a_requirement_is_stored_as_its_documented_shape() {
    let requirement = SpatialRequirement::at_place(cafe())
        .within(Millimetres::new(3_000))
        .expect("three metres is a legal range")
        .requiring_line_of_access()
        .requiring_target_available();

    let text = r#"{"place":{"specific":{"entity":7,"entity_type":"place"}},"within_range":3000,"requires_line_of_access":true,"requires_target_available":true}"#;
    assert_eq!(serde_json::to_string(&requirement).unwrap(), text);
    assert_eq!(
        serde_json::from_str::<SpatialRequirement>(text).unwrap(),
        requirement
    );

    assert_eq!(
        serde_json::to_string(&SpatialRequirement::NONE).unwrap(),
        r#"{"place":"any","within_range":null,"requires_line_of_access":false,"requires_target_available":false}"#
    );
    assert_eq!(SpatialRequirement::NONE.place(), PlaceRequirement::Any);
    assert_eq!(SpatialRequirement::NONE.within_range(), None);
}

/// A structural guarantee about the whole crate, not only about this module: `AC-12` requires that
/// a fixed seed with fixed inputs reproduces a run exactly, and floating-point arithmetic is not
/// reproducible across platforms or optimization levels. Positions and angles reach the event log,
/// so one `f32` anywhere in this vocabulary would make two runs of one world diverge in a way no
/// behavioural test would catch.
///
/// The test reads the crate's own sources, because that is the only way to assert the *absence* of
/// a type. Two deliberate limits on what it reads:
///
/// - it scans `src/` and not `tests/`, because this file has to name the forbidden types in order
///   to look for them, and would fail on itself;
/// - it scans code and not comments, because documenting the ban — as `spatial.rs` does, and as
///   this very test does — is not breaking it. The crate uses no block comments, so discarding
///   everything after `//` is sufficient; a block comment naming one of these types would produce
///   a false failure, which is loud and correctable rather than silent.
#[test]
fn no_floating_point_appears_anywhere_in_the_contract_crate() {
    let sources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut scanned = Vec::new();
    let mut offences = Vec::new();

    let mut pending = vec![sources.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the crate's src/ must be readable") {
            let path = entry.expect("a directory entry must be readable").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a source file must be readable");
            for (number, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or_default();
                if code.contains("f32") || code.contains("f64") {
                    offences.push(format!(
                        "{}:{}: {}",
                        path.display(),
                        number + 1,
                        line.trim()
                    ));
                }
            }
            scanned.push(path);
        }
    }

    assert!(
        !scanned.is_empty(),
        "the scan found no source files at {}, so it proves nothing",
        sources.display()
    );
    assert!(
        offences.is_empty(),
        "the contract layer must contain no floating point, but found:\n{}",
        offences.join("\n")
    );
}
