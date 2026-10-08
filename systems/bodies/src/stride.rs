//! A stride within one place (step-11 SD-B6 with objects, SD-O9; the prototype's mode R′, §9.7).
//!
//! ```text
//! clear corridor (integers): people and objects' footprints   → exactly `to`
//! head-on bias (people only)
//! objects pushed:  walls-only reach W, contact reach B (people and objects), candidate,
//!                  nudge pass against the walls; the pushes of every arrival predicted —
//!                  accepted iff nothing jams, nothing is pushed twice, and the final state holds
//! objects solid:   otherwise the same with the objects as walls, and nothing pushed
//! verify V1–V4 on integers; degrade: blocked at first contact → halved ×8 → stay
//! ```

use mineworld_contracts::EntityId;

use crate::footprint::{Footprint, Placed};
use crate::geometry::{
    CHAIN_MAX, CLEARANCE, GAP, HALVINGS, NUDGE_MAX, NUDGED_MAX, PERSON_RADIUS, Point, Room, SNAP,
    TOLERANCE, at_least, closest_pair, distance2, first_met, head_on, no_longer_than, scaled_down,
    turned_right,
};
use crate::push::Lay;
use crate::rapier::{Against, Scene, Touch};
use crate::resolve::{Answer, Degraded, Here, Objects, Outcome, Policy, Route};
use crate::walls::walled;

/// Everybody's position in one candidate result: the walker's and each displaced person's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) walker: Point,
    /// Indexes into the others, with their new positions, in the order they were moved.
    pub(crate) displaced: Vec<(usize, Point)>,
}

impl Candidate {
    fn only(walker: Point) -> Self {
        Self {
            walker,
            displaced: Vec::new(),
        }
    }

    /// Every arrival of the emission list this candidate becomes, in its order: the walker's end,
    /// then each displaced person's.
    fn arrivals(&self) -> Vec<Point> {
        core::iter::once(self.walker)
            .chain(self.displaced.iter().map(|(_, at)| *at))
            .collect()
    }

    /// Everybody in the place where this candidate leaves them: the others, moved, and the walker.
    fn everyone(&self, others: &[(EntityId, Point)]) -> Vec<Point> {
        let mut points: Vec<Point> = others.iter().map(|(_, at)| *at).collect();
        for (index, at) in &self.displaced {
            points[*index] = *at;
        }
        points.push(self.walker);
        points
    }
}

/// One mode's sweeps and nudges: its candidate, and what the sweeps found.
struct Tried {
    candidate: Candidate,
    reach: Reach,
    nudge_failed: bool,
    generations: usize,
}

/// A stride within the place (step-11 SD-B6, SD-O9).
pub(crate) fn stride(
    here: &Here,
    walker: EntityId,
    start: Point,
    target: Point,
    policy: Policy,
) -> Answer {
    let (room, standing) = (&here.room, &here.standing);
    let me = standing
        .iter()
        .position(|(person, _)| *person == walker)
        .expect("a walker with a position in the place stands in it");
    let others: Vec<(EntityId, Point)> = standing
        .iter()
        .copied()
        .filter(|(person, _)| *person != walker)
        .collect();
    let other_points: Vec<Point> = others.iter().map(|(_, at)| *at).collect();
    let lay = Lay::new(room, &here.objects);
    let laid = lay.footprints();
    let margin = PERSON_RADIUS.value() + GAP.value();
    if room.corridor_clear(start, target, &other_points, policy.exact_corridor)
        && laid
            .iter()
            .all(|footprint| footprint.clear_of_segment(start, target, margin))
    {
        return Answer::plain(target, Route::Clear);
    }
    // Only the floor's edge can stop it: integers answer, no scene is built (step-11 SD-Z4). V1–V4
    // still hold the answer; one that failed them would go on to Rapier.
    if policy.integer_walls
        && let Some(end) = walled(room, &other_points, &laid, start, target)
        && (!policy.verify || verifies(room, &others, &Candidate::only(end), &laid))
    {
        return Answer::plain(end, Route::Walled);
    }

    // The head-on bias (step-11 SD-B10): a walker whose first person met stands within the band of
    // their line aims to their right instead — "keep right" — so that two walkers meeting head-on
    // pass rather than pushing each other straight back. Only the walker's own aim turns; nudges stay
    // straight away from their pusher.
    let asked = target.minus(start);
    let met = first_met(start, asked, &other_points)
        .filter(|(_, cross2)| policy.bias && head_on(asked, *cross2))
        .map(|(index, _)| others[index].0);
    let aim = match met {
        Some(_) => start.plus(turned_right(asked)),
        None => target,
    };

    let points: Vec<Point> = standing.iter().map(|(_, at)| *at).collect();
    let placed: Vec<Placed> = here.objects.iter().map(|(_, placed)| *placed).collect();
    let scene = Scene::build(room, &points, &placed);
    // The scene index of each other person: they keep their place in `standing`.
    let in_scene: Vec<usize> = (0..standing.len()).filter(|index| *index != me).collect();
    let desired = aim.minus(start);
    let fallback = if desired == Point::new(0, 0) {
        Point::new(1, 0)
    } else {
        desired
    };
    let attempt = |walls: Against| {
        let reach = reach(&scene, me, start, aim, walls);
        match nudge(&scene, &others, &in_scene, reach.candidate, fallback, walls) {
            Ok(nudged) => Tried {
                candidate: Candidate {
                    walker: reach.candidate,
                    displaced: nudged.moved,
                },
                reach,
                nudge_failed: false,
                generations: nudged.generations,
            },
            Err(()) => Tried {
                candidate: Candidate::only(reach.blocked),
                reach,
                nudge_failed: true,
                generations: 0,
            },
        }
    };

    // SD-O9 step 6: the pushes every arrival of the list will make, predicted with the reactions'
    // own function; objects solid when they cannot all be made.
    let first = attempt(Against::Walls);
    let prediction = (!first.nudge_failed)
        .then(|| {
            lay.predict(
                &first.candidate.arrivals(),
                &first.candidate.everyone(&others),
            )
        })
        .flatten();
    let (tried, objects, after) = match prediction {
        Some(predicted) if !predicted.pushes.is_empty() => {
            let count = predicted.pushes.len();
            (first, Objects::Pushed(count), predicted.after)
        }
        Some(_) => (first, Objects::Untouched, laid.clone()),
        None if first.nudge_failed => (first, Objects::Untouched, laid.clone()),
        None => (
            attempt(Against::WallsAndObjects),
            Objects::Solid,
            laid.clone(),
        ),
    };

    let (result, degraded, objects) =
        if !policy.verify || verifies(room, &others, &tried.candidate, &after) {
            (tried.candidate.clone(), Degraded::No, objects)
        } else {
            let (result, degraded) = degrade(room, &others, start, tried.reach.blocked, &laid);
            (result, degraded, Objects::Untouched)
        };
    let generations = if result.displaced.is_empty() {
        0
    } else {
        tried.generations
    };
    let touched = |touch: Touch| match touch {
        Touch::Person(index) => standing[index].0,
        Touch::Object(index) => here.objects[index].0.entity_id(),
    };
    let stopped_by = if result.walker == target {
        None
    } else if result.walker == aim {
        // The bias turned a walker who then reached the turned aim: what turned them is the person
        // they met head-on.
        met
    } else if degraded == Degraded::No && !tried.nudge_failed && tried.reach.walls_stopped {
        // The walls-only reach ended the stride — or, objects solid, the object it ran into.
        tried.reach.walls_touched.map(touched)
    } else {
        tried.reach.touched.map(touched)
    };
    Answer {
        reached: result.walker,
        displaced: result
            .displaced
            .iter()
            .map(|(index, at)| (others[*index].0, *at))
            .collect(),
        stopped_by,
        outcome: Outcome {
            route: Route::Swept,
            generations,
            nudged: result.displaced.len(),
            nudge_failed: tried.nudge_failed,
            degraded,
            biased: met.is_some(),
            objects,
        },
    }
}

/// What the two sweeps of a stride say: where the walker would end before anybody is nudged, where it
/// stops if blocked, and what it touched.
struct Reach {
    /// R′ step 3's candidate: the walls-only reach W, or W cut `NUDGE_MAX` beyond contact.
    candidate: Point,
    /// Where a blocked walker stands: the contact reach B's first touch of a person or an object.
    blocked: Point,
    /// Whether the candidate is W itself — the walls (or, objects solid, an object) ended the stride.
    walls_stopped: bool,
    /// What W touched first: an object, when objects are solid; nothing otherwise.
    walls_touched: Option<Touch>,
    /// The first person or object the contact sweep touched.
    touched: Option<Touch>,
}

/// R′ steps 1–3 (step-11 §4.5.1): the walls-only reach W (against `walls`) and the contact reach B
/// (the fixed geometry, the people and the objects), each quantized, snapped to `target` within
/// [`SNAP`], and never longer than asked; then the candidate.
///
/// How far the walker gets with people solid is the contact sweep's end, sliding included — the
/// prototype's B, which the candidate rule measures. Where a blocked walker stops is where that sweep
/// first touched a person or an object, not where the controller's slide along its curve carried it
/// on to: a blocked walker ends at contact (step-11 §17.11, DB-4).
fn reach(scene: &Scene, me: usize, start: Point, target: Point, walls: Against) -> Reach {
    let desired = target.minus(start);
    let settle = |end: Point| {
        let snapped = if distance2(end, target) <= i64::from(SNAP.value()).pow(2) {
            target
        } else {
            end
        };
        no_longer_than(start, snapped, target)
    };
    let walled = scene.sweep(Some(me), start, desired, walls);
    let reached = settle(walled.end);
    let contact = scene.sweep(Some(me), start, desired, Against::Contact);
    let swept = settle(contact.end);
    let (reach_walls, reach_contact) = (reached.minus(start).length(), swept.minus(start).length());
    let beyond = reach_contact + i64::from(NUDGE_MAX.value());
    let walls_stopped = reach_walls <= beyond;
    Reach {
        candidate: if walls_stopped {
            reached
        } else {
            start.plus(scaled_down(reached.minus(start), beyond, reach_walls))
        },
        blocked: contact.contact.map_or(swept, settle),
        walls_stopped,
        walls_touched: walled.touched,
        touched: contact.touched,
    }
}

/// Verify-then-degrade's fallbacks, after the resolved result failed verification (step-11 DC-8):
/// blocked at contact; then the advance along the blocked path halved, up to [`HALVINGS`] times;
/// then stay — each with the objects where they lay, so nothing is pushed. Staying is valid by
/// induction: the place as it stood passed the guard.
fn degrade(
    room: &Room,
    others: &[(EntityId, Point)],
    start: Point,
    blocked: Point,
    laid: &[Footprint],
) -> (Candidate, Degraded) {
    if verifies(room, others, &Candidate::only(blocked), laid) {
        return (Candidate::only(blocked), Degraded::Blocked);
    }
    let advance = blocked.minus(start);
    for k in 1..=HALVINGS {
        let walker = start.plus(Point::new(advance.x / (1 << k), advance.y / (1 << k)));
        if verifies(room, others, &Candidate::only(walker), laid) {
            return (Candidate::only(walker), Degraded::Halved(k));
        }
    }
    (Candidate::only(start), Degraded::Stayed)
}

/// V1–V4 on integers (step-11 SD-B6 step 6, SD-O9 step 7): every pair in the place at least
/// [`CLEARANCE`] apart; every moved centre inside the floor and out of every solid, to within
/// [`TOLERANCE`]; and no person overlapping any object's footprint as the objects will lie.
pub(crate) fn verifies(
    room: &Room,
    others: &[(EntityId, Point)],
    candidate: &Candidate,
    objects: &[Footprint],
) -> bool {
    let points = candidate.everyone(others);
    let clearance = i64::from(CLEARANCE.value());
    let apart =
        closest_pair(&points).is_none_or(|(_, _, distance2)| distance2 >= clearance * clearance);
    let margin = PERSON_RADIUS.value() - TOLERANCE.value();
    apart
        && room.admits(candidate.walker, margin)
        && candidate
            .displaced
            .iter()
            .all(|(_, at)| room.admits(*at, margin))
        && points
            .iter()
            .all(|person| objects.iter().all(|footprint| !footprint.under(*person)))
}

/// The people a nudge pass moved, in the order it moved them, and how many generations it took.
pub(crate) struct Nudged {
    pub(crate) moved: Vec<(usize, Point)>,
    pub(crate) generations: usize,
}

/// The nudge pass (step-11 SD-B7; QB-10, I-11). Generation 1's pusher is the arriving person at
/// `pusher`; generation g's pushers are the people moved in generation g − 1. In each generation, for
/// each pusher, every person not yet moved, in `EntityId` order, who is closer than two radii to it is
/// swept straight away from its centre — against `walls` — just far enough to stand two radii and a
/// gap from it. When two centres coincide, `fallback` gives the direction.
///
/// The pass fails if a nudge would exceed `NUDGE_MAX + GAP`, if the walls cut one more than [`SNAP`]
/// short, if more than [`NUDGED_MAX`] people would move, or if any pair is still closer than
/// [`CLEARANCE`] after [`CHAIN_MAX`] generations.
pub(crate) fn nudge(
    scene: &Scene,
    others: &[(EntityId, Point)],
    in_scene: &[usize],
    pusher: Point,
    fallback: Point,
    walls: Against,
) -> Result<Nudged, ()> {
    let touching = i64::from(2 * PERSON_RADIUS.value());
    let spacing = i64::from(2 * PERSON_RADIUS.value() + GAP.value());
    let longest = i64::from(NUDGE_MAX.value() + GAP.value());
    let mut now: Vec<Point> = others.iter().map(|(_, at)| *at).collect();
    let mut moved: Vec<(usize, Point)> = Vec::new();
    let mut frontier = vec![pusher];
    let mut generations = 0;
    for _ in 0..CHAIN_MAX {
        let mut next = Vec::new();
        for pushing in &frontier {
            for index in 0..others.len() {
                if moved.iter().any(|(done, _)| *done == index) {
                    continue;
                }
                let at = now[index];
                let between = distance2(at, *pushing);
                if between >= touching * touching {
                    continue;
                }
                let needed = spacing - between.isqrt();
                if needed > longest {
                    return Err(());
                }
                let away = if between == 0 {
                    fallback
                } else {
                    at.minus(*pushing)
                };
                let by = at_least(away, needed);
                if distance2(by, Point::new(0, 0)) > longest * longest {
                    return Err(());
                }
                let swept = scene.sweep(Some(in_scene[index]), at, by, walls).end;
                let end = if distance2(swept, at.plus(by)) <= i64::from(SNAP.value()).pow(2) {
                    at.plus(by)
                } else {
                    swept
                };
                let travelled = end.minus(at).length();
                if travelled + i64::from(SNAP.value()) < by.length()
                    || distance2(end, at) > longest * longest
                {
                    return Err(());
                }
                now[index] = end;
                moved.push((index, end));
                next.push(end);
                if moved.len() > NUDGED_MAX {
                    return Err(());
                }
            }
        }
        if next.is_empty() {
            break;
        }
        generations += 1;
        frontier = next;
    }
    let mut everyone = now;
    everyone.push(pusher);
    let clearance = i64::from(CLEARANCE.value());
    if closest_pair(&everyone).is_some_and(|(_, _, distance2)| distance2 < clearance * clearance) {
        return Err(());
    }
    Ok(Nudged { moved, generations })
}
