//! The considerations an option is scored by: each reads one input and maps it through a curve to a per-mille
//! value (`pr-dl-b-motivated.md` §5.4, SD-DLb-4). The table in §5.4 decides which considerations each choice
//! carries; this module decides their values.

use mineworld_contracts::Tag;
use mineworld_rule_controller::needs::Felt;
use mineworld_weather::day::Condition;

use crate::options::{Choice, Context, Door};
use crate::settings::{Roles, Settings};

/// What a consideration reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consideration {
    /// The hour's sociability (ATUS, socializing and communicating).
    Sociable,
    /// Hours since my newest contact with anybody.
    Lonely,
    /// My familiarity with the person the choice names.
    Liking,
    /// My regard for the person the choice names.
    Regard,
    /// The hour's rest (ATUS, sleeping), while I am at home.
    Rested,
    /// The hour's restlessness: the inverse of rest, while I am away from home.
    Restless,
    /// The hour's eating (ATUS, eating and drinking).
    HungryHour,
    /// The hour's work (ATUS, working).
    WorkHour,
    /// The hour's shopping (ATUS, purchasing goods and services).
    ShopHour,
    /// The hour's leisure (ATUS, leisure and sports).
    LeisureHour,
    /// How far through my agenda slot I am.
    Late,
    /// The weather, for the outdoors.
    Dry,
    /// How long I have been part of my activity.
    Settled,
    /// How urgent setting out for a punctual next part is (`ARC-282`).
    Departure,
    /// The inverse: how readily I take up small talk while a commute presses.
    Unhurried,
    /// How many times the member of staff a penalty is for was late within the window (`ARC-287`).
    Strictness,
    /// My hunger, for eating, a door to somewhere to eat and buying (`ARC-286`).
    Hungry,
    /// The inverse of `Hungry`: defined and overridable, carried by no choice in MVP-0 (F-Nc-5).
    Sated,
    /// My tiredness: defined and overridable, carried by no choice since `pressure` replaced it for sleep and
    /// a door home (`pr-s35-nc3` SD-Nc3-2).
    Tired,
    /// The inverse: how readily I greet, approach, invite and wander while tired.
    Wakeful,
    /// My tiredness against the two-process model's upper threshold now: when sleep wins, and when to
    /// head home (`pr-s35-nc3` SD-Nc3-2).
    Pressure,
}

/// The offers whose score my hunger raises, by their slugs: consumption's `eat` and `drink`, and
/// economy's `buy` — named, never linked (`ARC-35` check 2).
const FOOD_OFFERS: [&str; 3] = ["eat", "drink", "buy"];

/// The role a place plays: the first of these whose tags a place carries (§5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A home.
    Home,
    /// Somewhere to eat.
    Eat,
    /// Somewhere to work.
    Work,
    /// Somewhere to shop.
    Shop,
    /// Outdoors.
    Outdoor,
}

/// The role of a place with these tags, if any of the role lists names one of them.
pub fn role_of(roles: &Roles, tags: &[Tag]) -> Option<Role> {
    let carries = |list: &[Tag]| list.iter().any(|tag| tags.contains(tag));
    if carries(&roles.home) {
        Some(Role::Home)
    } else if carries(&roles.eat) {
        Some(Role::Eat)
    } else if carries(&roles.work) {
        Some(Role::Work)
    } else if carries(&roles.shop) {
        Some(Role::Shop)
    } else if carries(&roles.outdoor) {
        Some(Role::Outdoor)
    } else {
        None
    }
}

/// The considerations `choice` carries, each with its per-mille value. An input the observation does not
/// disclose is neutral: its value is 1 000 (I-DLb-4).
pub fn of(
    choice: &Choice,
    context: &Context<'_>,
    settings: &Settings,
) -> Vec<(Consideration, u32)> {
    let curves = &settings.curves;
    let hour = context.hour;
    let atus = |name: &str| {
        curves
            .atus
            .get(name)
            .map(|curve| curve.at(hour))
            .unwrap_or(1_000)
    };
    let here_role = role_of(&settings.roles, &context.here_tags);
    let at_home = here_role == Some(Role::Home);
    let mut list = by_choice(choice, context, settings, here_role, at_home, atus);
    // A person about to be late tends to decline small talk: only while a commute presses, so that in
    // a world with no punctual part every score is what it was (I-LE-4).
    if let Some(pending) = context.commute.filter(|pending| pending.pressing())
        && matches!(
            choice,
            Choice::Greet(_) | Choice::Approach(_) | Choice::Invite(_) | Choice::Wander
        )
    {
        list.push((
            Consideration::Unhurried,
            curves.unhurried.at(i64::from(pending.urgency)),
        ));
    }
    // My needs, only when `needs` discloses them: absent, no consideration is added, so every score is
    // what it was (I-Nc-1) — a neutral 1 000 would still change the compensation's count.
    if let Some(felt) = &context.felt {
        list.extend(needs_considerations(choice, context, settings, felt));
    }
    list
}

/// The considerations my needs add to `choice` (step-35 §6.3; `pr-s35-nc-controllers.md` SD-Nc-7).
fn needs_considerations(
    choice: &Choice,
    context: &Context<'_>,
    settings: &Settings,
    felt: &Felt,
) -> Vec<(Consideration, u32)> {
    let curves = &settings.curves;
    let hungry = (
        Consideration::Hungry,
        curves.hungry.at(i64::from(felt.hunger)),
    );
    match choice {
        Choice::Offered(index) => context
            .observation
            .affordances()
            .get(*index)
            .filter(|affordance| FOOD_OFFERS.contains(&affordance.action_type().as_str()))
            .map(|_| vec![hungry])
            .unwrap_or_default(),
        Choice::Door(place) => {
            let door = context.doors.iter().find(|door| door.to == *place);
            match door.and_then(|door| role_of(&settings.roles, &door.tags)) {
                Some(Role::Eat) => vec![hungry],
                // Somewhere to buy food, while I hold nothing that nourishes me (`pr-s35-nc3` SD-Nc3-5):
                // the same disclosure `rule`'s fetch band reads.
                Some(Role::Shop) if !holds_food(felt) => vec![hungry],
                // Home about an hour before sleep would win, not whenever tired (SD-Nc3-2).
                Some(Role::Home) => vec![pressure(
                    context,
                    settings,
                    i64::from(felt.tiredness) + i64::from(settings.homeward),
                )],
                _ => Vec::new(),
            }
        }
        Choice::Greet(_) | Choice::Approach(_) | Choice::Invite(_) | Choice::Wander => vec![(
            Consideration::Wakeful,
            curves.wakeful.at(i64::from(felt.tiredness)),
        )],
        _ => Vec::new(),
    }
}

/// `pressure` at `tiredness`: its margin over the upper threshold `sleep_from` at this minute of the day.
fn pressure(context: &Context<'_>, settings: &Settings, tiredness: i64) -> (Consideration, u32) {
    let minute = context.now.rem_euclid(86_400) / 60;
    let margin = tiredness - i64::from(settings.curves.sleep_from.at(minute));
    (Consideration::Pressure, settings.curves.pressure.at(margin))
}

/// Whether I carry anything that nourishes me, by my own `needs` record's `nourishment` list.
fn holds_food(felt: &Felt) -> bool {
    felt.nourishment.iter().any(|food| food.hunger > 0)
}

/// The considerations of `choice` itself, before the commute's.
fn by_choice(
    choice: &Choice,
    context: &Context<'_>,
    settings: &Settings,
    here_role: Option<Role>,
    at_home: bool,
    atus: impl Fn(&str) -> u32,
) -> Vec<(Consideration, u32)> {
    let curves = &settings.curves;
    let sociable = (Consideration::Sociable, atus("sociability"));
    let lonely = (
        Consideration::Lonely,
        context
            .last_contact_hours
            .map(|hours| curves.loneliness.at(hours))
            .unwrap_or(1_000),
    );
    let liking_regard = |person: mineworld_contracts::PersonId| {
        let values = context.knows(person);
        (
            (
                Consideration::Liking,
                values
                    .map(|v| curves.familiarity.at(i64::from(v.familiarity())))
                    .unwrap_or(1_000),
            ),
            (
                Consideration::Regard,
                values
                    .map(|v| curves.regard.at(i64::from(v.regard())))
                    .unwrap_or(1_000),
            ),
        )
    };
    match choice {
        Choice::Commute(_) => {
            let urgency = context.commute.map_or(0, |pending| pending.urgency);
            vec![(
                Consideration::Departure,
                curves.departure.at(i64::from(urgency)),
            )]
        }
        Choice::Agenda => {
            let mut list = Vec::new();
            if let Some(agenda) = &context.agenda {
                list.push((
                    Consideration::Late,
                    curves.lateness.at(late_permille(context)),
                ));
                if agenda.label().as_str() == "work" {
                    list.push((Consideration::WorkHour, atus("working")));
                }
            }
            list
        }
        Choice::Stay => {
            let rested = if at_home { atus("rest") } else { 1_000 };
            vec![(Consideration::Rested, rested)]
        }
        // The two-process model's upper threshold holds both the hour and the need, so `pressure` replaces
        // `rested` × `tired`, which would count each twice (`pr-s35-nc3` SD-Nc3-2). `Sleep` is offered only
        // with needs disclosed.
        Choice::Sleep(_) => {
            let tiredness = context.felt.as_ref().map_or(0, |felt| felt.tiredness);
            vec![pressure(context, settings, i64::from(tiredness))]
        }
        Choice::Greet(person) | Choice::Approach(person) | Choice::Invite(person) => {
            let (liking, regard) = liking_regard(*person);
            vec![sociable, lonely, liking, regard]
        }
        Choice::Join(person) => {
            let (liking, regard) = liking_regard(*person);
            vec![sociable, liking, regard]
        }
        Choice::Leave => {
            let minutes = context
                .activity
                .as_ref()
                .map(|activity| (context.now - activity.since().seconds()).max(0) / 60)
                .unwrap_or(0);
            vec![(Consideration::Settled, curves.activity_age.at(minutes))]
        }
        Choice::Door(place) => {
            let door = context.doors.iter().find(|door| door.to == *place);
            door_considerations(door, context, settings, at_home, atus)
        }
        Choice::Offered(_) => vec![(Consideration::ShopHour, atus("shopping"))],
        Choice::Penalize(index) => {
            let count = context
                .penalties
                .iter()
                .find(|(at, _)| at == index)
                .map_or(0, |(_, count)| *count);
            vec![(
                Consideration::Strictness,
                curves.strictness.at(i64::from(count)),
            )]
        }
        Choice::Wander => {
            let mut list = vec![(Consideration::LeisureHour, atus("leisure"))];
            if here_role == Some(Role::Outdoor) {
                list.push((Consideration::Dry, dry(context.weather)));
            }
            list
        }
    }
}

/// The considerations of a doorway: restlessness away from home, and what the place it leads to offers.
fn door_considerations(
    door: Option<&Door>,
    context: &Context<'_>,
    settings: &Settings,
    at_home: bool,
    atus: impl Fn(&str) -> u32,
) -> Vec<(Consideration, u32)> {
    let mut list = Vec::new();
    let role = door.and_then(|door| role_of(&settings.roles, &door.tags));
    if !at_home {
        let to_home = role == Some(Role::Home);
        let value = if to_home {
            atus("rest")
        } else {
            atus("restless")
        };
        list.push((Consideration::Restless, value));
    }
    match role {
        Some(Role::Eat) => list.push((Consideration::HungryHour, atus("eating"))),
        Some(Role::Work) => list.push((Consideration::WorkHour, atus("working"))),
        Some(Role::Shop) => list.push((Consideration::ShopHour, atus("shopping"))),
        Some(Role::Outdoor) => {
            list.push((Consideration::LeisureHour, atus("leisure")));
            list.push((Consideration::Dry, dry(context.weather)));
        }
        Some(Role::Home) | None => {}
    }
    list
}

/// How far through my agenda slot I am, in per-mille, clamped to 0 … 1 000. Neutral (1 000) when there is no
/// slot to be through.
fn late_permille(context: &Context<'_>) -> i64 {
    let Some(agenda) = &context.agenda else {
        return 1_000;
    };
    let since = agenda.since().seconds();
    let until = agenda.until().seconds();
    let length = until - since;
    if length <= 0 {
        return 1_000;
    }
    ((context.now - since) * 1_000 / length).clamp(0, 1_000)
}

/// The dryness value of a condition (§5.4): dry 1 000, drizzle 600, rain 300, heavier 150. Neutral (1 000)
/// when the weather is not disclosed.
pub fn dry(condition: Option<Condition>) -> u32 {
    match condition {
        None => 1_000,
        Some(Condition::Drizzle) => 600,
        Some(Condition::Rain) => 300,
        Some(condition) if condition_is_wet(condition) => 150,
        Some(_) => 1_000,
    }
}

/// Whether a condition is wet.
fn condition_is_wet(condition: Condition) -> bool {
    matches!(
        condition,
        Condition::Drizzle | Condition::Rain | Condition::HeavyRain | Condition::Thunderstorm
    )
}
