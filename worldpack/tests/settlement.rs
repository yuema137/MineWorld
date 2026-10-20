//! A town keeps its station-and-landmark contract, and a broken one is refused by name (`ARC-105`,
//! `MODULE_SPEC.md` §4.3).
//!
//! Each test writes a real pack: a minimal town (a street, the hall, a ticket bay and a platform, one
//! seat in the street), then breaks exactly one thing. The assertions read the refusal's code, its file
//! and its key, never its prose.

use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_test_support::Scratch;
use mineworld_worldpack::{PackError, SettlementCode, SettlementKind, WorldPack};

struct Fixture {
    root: Scratch,
}

/// The minimal town's places: key, tags, and the passages its file states.
const STREET: &str = "tags: [street, public, outdoor]\n";
const HALL: &str = "tags: [station, public, landmark]\npassages:\n  - to: street\n";
const TICKETS: &str = "tags: [station, ticket-office, public]\npassages:\n  - to: station\n";
const PLATFORM: &str =
    "tags: [station, stop, platform, public, outdoor]\npassages:\n  - to: station\n";

impl Fixture {
    fn new(id: &str) -> Self {
        let root = mineworld_test_support::scratch!(id);
        std::fs::create_dir_all(root.join("people")).expect("a writable temporary directory");
        std::fs::create_dir_all(root.join("places")).expect("a writable temporary directory");
        Self { root }
    }

    /// A town with every rule kept: systems presence and movement, the station group joined to the street,
    /// and one seat, `ada`, starting in the street.
    fn town(id: &str) -> Self {
        let fixture = Self::new(id);
        fixture.manifest(
            "systems: [presence, movement]",
            "settlement:\n  kind: town\n  station: station\n  landmark: station\n",
        );
        fixture.write("places/street.yaml", STREET);
        fixture.write("places/station.yaml", HALL);
        fixture.write("places/station-tickets.yaml", TICKETS);
        fixture.write("places/station-platform.yaml", PLATFORM);
        fixture.write(
            "people/ada.yaml",
            "tags: [resident]\nlocation:\n  place: street\n",
        );
        fixture
    }

    /// Writes `world.yaml` with the town's lists, `systems` and `settlement` as given.
    fn manifest(&self, systems: &str, settlement: &str) {
        let id = self.id();
        self.write(
            "world.yaml",
            &format!(
                "world:\n  id: {id}\n  name: A Test Town\n{systems}\nplaces: [station, station-platform, \
                 station-tickets, street]\npopulation: [ada]\nseats: [ada]\n{settlement}"
            ),
        );
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        std::fs::write(&path, contents).unwrap_or_else(|error| {
            panic!("the fixture must be writable: {} — {error}", path.display())
        });
    }

    fn id(&self) -> String {
        self.root
            .file_name()
            .expect("a named directory")
            .to_string_lossy()
            .into_owned()
    }

    fn read(&self) -> Result<WorldPack, PackError> {
        WorldPack::read(&self.root)
    }

    /// The settlement refusal, asserted to carry `code`, to name `file` and `key`.
    fn refused(&self, code: SettlementCode, file: &str, key: Option<&str>) {
        let refusal = match self.read() {
            Ok(_) => panic!("this town must be refused with {code}, and it read"),
            Err(error) => error,
        };
        let PackError::Settlement {
            code: got,
            path,
            key: named,
            ..
        } = &refusal
        else {
            panic!("expected {code}, got: {refusal}");
        };
        assert_eq!(*got, code, "got: {refusal}");
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some(file),
            "the refusal names the file: {refusal}"
        );
        assert_eq!(named.as_ref().map(EntityKey::as_str), key, "got: {refusal}");
        assert!(refusal.to_string().starts_with(code.as_str()));
    }

    /// The decoding refusal: a `Malformed` world.yaml whose detail carries the code and a line.
    fn malformed(&self, expect: &str) {
        let refusal = match self.read() {
            Ok(_) => panic!("this settlement must be malformed, and it read"),
            Err(error) => error,
        };
        let PackError::Malformed { kind, detail, .. } = &refusal else {
            panic!("expected a malformed world.yaml, got: {refusal}");
        };
        assert_eq!(*kind, "world");
        assert!(
            detail.contains("SETTLEMENT_MALFORMED"),
            "the code travels: {detail}"
        );
        assert!(detail.contains(expect), "expected `{expect}` in: {detail}");
        assert!(
            detail.chars().any(|c| c.is_ascii_digit()),
            "the parser's position travels: {detail}"
        );
    }
}

#[test]
fn a_town_with_a_station_group_reads() {
    let pack = Fixture::town("town-reads")
        .read()
        .expect("a sound town reads");
    let settlement = pack.require_settlement().expect("it says it is a town");
    assert_eq!(
        settlement.kind(),
        &SettlementKind::Town {
            station: EntityKey::new("station").unwrap(),
            landmark: EntityKey::new("station").unwrap(),
        }
    );
}

#[test]
fn kind_none_reads() {
    let fixture = Fixture::town("town-kind-none");
    fixture.manifest(
        "systems: [presence, movement]",
        "settlement:\n  kind: none\n",
    );
    let pack = fixture.read().expect("a world that is not a town reads");
    assert_eq!(
        pack.settlement().map(|s| s.kind()),
        Some(&SettlementKind::None)
    );
}

#[test]
fn a_world_without_a_settlement_still_loads() {
    let fixture = Fixture::town("town-no-settlement");
    fixture.manifest("systems: [presence, movement]", "");
    let pack = fixture.read().expect("the key is optional to the loader");
    assert!(pack.settlement().is_none());
    let refusal = pack.require_settlement().expect_err("the gate requires it");
    assert!(matches!(
        refusal,
        PackError::Settlement {
            code: SettlementCode::Undeclared,
            ..
        }
    ));
    pack.load(WorldTime::EPOCH).expect("and it still loads");
}

#[test]
fn every_malformed_settlement_is_refused_at_its_line() {
    let cases = [
        (
            "settlement:\n  kind: town\n  station: station\n  landmark: station\n  depot: x\n",
            "unknown key `depot`",
        ),
        // Either this loader's visitor or the YAML parser may refuse the second `kind` first; both
        // name the key, and the code travels either way.
        ("settlement:\n  kind: none\n  kind: none\n", "kind"),
        ("settlement:\n  station: station\n", "`kind` is required"),
        (
            "settlement:\n  kind: village\n",
            "neither `town` nor `none`",
        ),
        (
            "settlement:\n  kind: none\n  landmark: station\n",
            "kind `none` names no `landmark`",
        ),
        (
            "settlement:\n  kind: town\n  station: station\n",
            "requires `landmark`",
        ),
        (
            "settlement:\n  kind: town\n  station: Not A Key\n  landmark: station\n",
            "SETTLEMENT_MALFORMED",
        ),
        // The visitor's `settlement is a map` or the parser's `expected mapping start`, by build.
        ("settlement: town\n", "map"),
    ];
    for (index, (settlement, expect)) in cases.into_iter().enumerate() {
        let fixture = Fixture::town(&format!("town-malformed-{index}"));
        fixture.manifest("systems: [presence, movement]", settlement);
        fixture.malformed(expect);
    }
}

#[test]
fn an_unknown_station_is_refused() {
    let fixture = Fixture::town("town-station-unknown");
    fixture.manifest(
        "systems: [presence, movement]",
        "settlement:\n  kind: town\n  station: depot\n  landmark: station\n",
    );
    fixture.refused(SettlementCode::StationUnknown, "world.yaml", Some("depot"));
}

#[test]
fn an_unknown_landmark_is_refused() {
    let fixture = Fixture::town("town-landmark-unknown");
    fixture.manifest(
        "systems: [presence, movement]",
        "settlement:\n  kind: town\n  station: station\n  landmark: tower\n",
    );
    fixture.refused(SettlementCode::LandmarkUnknown, "world.yaml", Some("tower"));
}

#[test]
fn a_hall_without_public_is_refused() {
    let fixture = Fixture::town("town-station-untagged");
    fixture.write(
        "places/station.yaml",
        "tags: [station, landmark]\npassages:\n  - to: street\n",
    );
    fixture.refused(
        SettlementCode::StationUntagged,
        "station.yaml",
        Some("station"),
    );
}

#[test]
fn a_landmark_without_its_tag_is_refused() {
    let fixture = Fixture::town("town-landmark-untagged");
    fixture.manifest(
        "systems: [presence, movement]",
        "settlement:\n  kind: town\n  station: station\n  landmark: street\n",
    );
    fixture.refused(
        SettlementCode::LandmarkUntagged,
        "street.yaml",
        Some("street"),
    );
}

#[test]
fn a_station_without_a_stop_is_refused() {
    let fixture = Fixture::town("town-no-stop");
    fixture.write(
        "places/station-platform.yaml",
        "tags: [station, platform, public]\npassages:\n  - to: station\n",
    );
    fixture.refused(SettlementCode::StationNoStop, "world.yaml", Some("station"));
}

#[test]
fn a_station_without_a_ticket_office_is_refused() {
    let fixture = Fixture::town("town-no-ticket-office");
    fixture.write(
        "places/station-tickets.yaml",
        "tags: [station, public]\npassages:\n  - to: station\n",
    );
    fixture.refused(
        SettlementCode::StationNoTicketOffice,
        "world.yaml",
        Some("station"),
    );
}

#[test]
fn a_scattered_station_is_refused() {
    // The platform opens onto the street, not the hall: it is tagged `station` but outside the group.
    let fixture = Fixture::town("town-scattered");
    fixture.write(
        "places/station-platform.yaml",
        "tags: [station, stop, platform, public, outdoor]\npassages:\n  - to: street\n",
    );
    fixture.write(
        "places/station-tickets.yaml",
        "tags: [station, ticket-office, stop, platform, public]\npassages:\n  - to: station\n",
    );
    fixture.refused(
        SettlementCode::StationScattered,
        "station-platform.yaml",
        Some("station-platform"),
    );
}

#[test]
fn a_station_no_seat_can_reach_is_refused() {
    let fixture = Fixture::town("town-unreachable");
    fixture.write("places/station.yaml", "tags: [station, public, landmark]\n");
    fixture.refused(SettlementCode::StationUnreachable, "ada.yaml", Some("ada"));
}

#[test]
fn a_landmark_no_seat_can_reach_is_refused() {
    let fixture = Fixture::town("town-landmark-unreachable");
    // A separate landmark, joined to nothing.
    let id = fixture.id();
    fixture.write(
        "world.yaml",
        &format!(
            "world:\n  id: {id}\n  name: A Test Town\nsystems: [presence, movement]\nplaces: [pier, \
             station, station-platform, station-tickets, street]\npopulation: [ada]\nseats: [ada]\n\
             settlement:\n  kind: town\n  station: station\n  landmark: pier\n"
        ),
    );
    fixture.write("places/pier.yaml", "tags: [pier, landmark, outdoor]\n");
    fixture.refused(SettlementCode::LandmarkUnreachable, "ada.yaml", Some("ada"));
}

#[test]
fn without_movement_the_group_is_the_tagged_set() {
    let fixture = Fixture::new("town-no-movement");
    let id = fixture.id();
    let manifest = |settlement_platform: &str| {
        fixture.write("places/station-platform.yaml", settlement_platform);
        fixture.write(
            "world.yaml",
            &format!(
                "world:\n  id: {id}\n  name: A Test Town\nsystems: [presence]\nplaces: [station, \
                 station-platform, station-tickets, street]\npopulation: [ada]\nseats: [ada]\n\
                 settlement:\n  kind: town\n  station: station\n  landmark: station\n"
            ),
        );
    };
    fixture.write("places/street.yaml", STREET);
    fixture.write("places/station.yaml", "tags: [station, public, landmark]\n");
    fixture.write(
        "places/station-tickets.yaml",
        "tags: [station, ticket-office, public]\n",
    );
    fixture.write(
        "people/ada.yaml",
        "tags: [resident]\nlocation:\n  place: street\n",
    );
    manifest("tags: [station, stop, platform, public, outdoor]\n");
    fixture
        .read()
        .expect("without passages, the tagged set is the group");
    manifest("tags: [station, platform, public, outdoor]\n");
    fixture.refused(SettlementCode::StationNoStop, "world.yaml", Some("station"));
}

#[test]
fn a_seat_without_a_location_is_not_checked() {
    let fixture = Fixture::town("town-seat-unlocated");
    fixture.write("places/station.yaml", "tags: [station, public, landmark]\n");
    fixture.write("places/station-tickets.yaml", TICKETS);
    fixture.write("people/ada.yaml", "tags: [resident]\n");
    // The hall is joined to nothing, but no seat starts anywhere, so nothing is unreachable.
    fixture
        .read()
        .expect("a seat with no starting place is skipped");
}

#[test]
fn every_code_is_reached() {
    // A structural guard: each code is produced by the test named beside it above. A code added to
    // `SettlementCode` without a test here fails this list.
    let tested = [
        (
            SettlementCode::Undeclared,
            "a_world_without_a_settlement_still_loads",
        ),
        (
            SettlementCode::Malformed,
            "every_malformed_settlement_is_refused_at_its_line",
        ),
        (
            SettlementCode::StationUnknown,
            "an_unknown_station_is_refused",
        ),
        (
            SettlementCode::LandmarkUnknown,
            "an_unknown_landmark_is_refused",
        ),
        (
            SettlementCode::StationUntagged,
            "a_hall_without_public_is_refused",
        ),
        (
            SettlementCode::LandmarkUntagged,
            "a_landmark_without_its_tag_is_refused",
        ),
        (
            SettlementCode::StationNoStop,
            "a_station_without_a_stop_is_refused",
        ),
        (
            SettlementCode::StationNoTicketOffice,
            "a_station_without_a_ticket_office_is_refused",
        ),
        (
            SettlementCode::StationScattered,
            "a_scattered_station_is_refused",
        ),
        (
            SettlementCode::StationUnreachable,
            "a_station_no_seat_can_reach_is_refused",
        ),
        (
            SettlementCode::LandmarkUnreachable,
            "a_landmark_no_seat_can_reach_is_refused",
        ),
    ];
    let codes: Vec<SettlementCode> = tested.iter().map(|(code, _)| *code).collect();
    assert_eq!(
        codes,
        SettlementCode::ALL.to_vec(),
        "every code has its test"
    );
    let source = include_str!("settlement.rs");
    for (code, test) in tested {
        assert!(
            source.contains(&format!("fn {test}()")),
            "{code}'s test {test} exists"
        );
    }
}
