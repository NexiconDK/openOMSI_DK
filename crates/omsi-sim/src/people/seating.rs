//! (the fork) Where a passenger sits, as people choose (the setting `pax_seating`):
//! a window seat before the one by the aisle, a double seat to themselves before one beside
//! a stranger, and an elderly passenger (`ELDERLY` years and older, the `.hum`'s `[age]`)
//! in the seat next to the driver while it is free - others leave it to them while they
//! have another. Omsi.exe takes any free place at random (`choose_place`).
//!
//! All of it from where the cabin's places are (bus frame: x to the right, y forward):
//! two seats side by side are a pair (the same row, a seat's width apart), the outer one of
//! a pair the window seat, a seat with no other beside it a single (by the window too).

use super::cabin::Seat;
use glam::Vec3;

/// From this age (years) a passenger counts as elderly: the seat next to the driver is
/// theirs, and somebody stands up for them.
pub const ELDERLY: f32 = 65.0;

/// Two seats are in one row when their fronts are this close along the bus (m).
const ROW: f32 = 0.35;
/// Two seats of a row are side by side when their centres are between these apart across
/// the bus (m): further apart, the aisle lies between them.
const BESIDE: (f32, f32) = (0.3, 0.75);
/// The seat next to the driver lies within this of the driver's place (m) - further, the
/// front seats are well back in the saloon and are not "next to the driver".
const BY_DRIVER: f32 = 2.6;

/// What the passengers know of a cabin's seats.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SeatMap {
    /// Per place: a seat by the window (or a single seat).
    pub window: Vec<bool>,
    /// Per place: the seat beside it, the two making a pair.
    pub beside: Vec<Option<usize>>,
    /// The passenger seat next to the driver.
    pub by_driver: Option<usize>,
}

impl SeatMap {
    /// The map of `places`, the driver at `driver` (bus frame) when known.
    pub fn of(places: &[Seat], driver: Option<Vec3>) -> SeatMap {
        let n = places.len();
        let mut beside = vec![None; n];
        for i in 0..n {
            if !places[i].seated {
                continue;
            }
            // the nearest seat of the same row a seat's width away, on the same side of the
            // aisle (they face the same way)
            let mut best: Option<(usize, f32)> = None;
            for j in 0..n {
                if j == i || !places[j].seated || places[j].group != places[i].group {
                    continue;
                }
                let (a, b) = (places[i].pos, places[j].pos);
                let across = (a.x - b.x).abs();
                if (a.y - b.y).abs() < ROW && across > BESIDE.0 && across < BESIDE.1 && same_way(places[i].rot, places[j].rot) && best.is_none_or(|(_, d)| across < d) {
                    best = Some((j, across));
                }
            }
            beside[i] = best.map(|b| b.0);
        }
        // (only pairs both ends agree on: a seat between two others is in no pair)
        for i in 0..n {
            if let Some(j) = beside[i] {
                if beside[j] != Some(i) {
                    beside[i] = None;
                }
            }
        }
        let centre = centre_x(places);
        let window = (0..n)
            .map(|i| {
                places[i].seated
                    && match beside[i] {
                        Some(j) => (places[i].pos.x - centre).abs() > (places[j].pos.x - centre).abs(),
                        None => true,
                    }
            })
            .collect();
        let by_driver = driver.and_then(|d| {
            (0..n)
                .filter(|&i| places[i].seated && places[i].group == 0)
                .map(|i| (i, (places[i].pos - d).truncate().length()))
                .filter(|(_, dist)| *dist < BY_DRIVER)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i)
        });
        SeatMap { window, beside, by_driver }
    }

    /// The place somebody takes of the `free` ones (`taken`: every place of the bus, true
    /// where somebody is): a seat when `seat` (and one is free), else a standing place;
    /// `elderly`: the seat next to the driver first. Among the seats: one of a pair with
    /// nobody in the other first, by the window first; then a single; the seat next to the
    /// driver last but for the elderly. `pick` breaks ties.
    pub fn choose(&self, free: &[usize], taken: &[bool], places: &[Seat], seat: bool, elderly: bool, pick: usize) -> Option<usize> {
        if free.is_empty() {
            return None;
        }
        let seats: Vec<usize> = free.iter().copied().filter(|k| places.get(*k).is_some_and(|p| p.seated)).collect();
        let standing: Vec<usize> = free.iter().copied().filter(|k| places.get(*k).is_some_and(|p| !p.seated)).collect();
        let want_seat = seat || elderly;
        if want_seat && !seats.is_empty() {
            if let Some(d) = self.by_driver.filter(|d| elderly && seats.contains(d)) {
                return Some(d);
            }
            let score = |k: usize| -> i32 {
                let alone = match self.beside.get(k).copied().flatten() {
                    Some(j) => !taken.get(j).copied().unwrap_or(false),
                    None => true,
                };
                let window = self.window.get(k).copied().unwrap_or(false);
                let by_driver = self.by_driver == Some(k) && !elderly;
                (alone as i32) * 4 + (window as i32) * 2 - (by_driver as i32) * 8
            };
            let top = seats.iter().map(|&k| score(k)).max()?;
            let best: Vec<usize> = seats.into_iter().filter(|&k| score(k) == top).collect();
            return Some(best[pick % best.len()]);
        }
        let from = if standing.is_empty() { free.to_vec() } else { standing };
        Some(from[pick % from.len()])
    }
}

/// (the fork) Seconds between two looks round the player's bus for a seat to change to.
const RESEAT_EVERY: f32 = 2.0;

impl super::PeopleSim {
    /// (the fork) In the player's bus, while it is not boarding (rolling, or its doors
    /// shut), every couple of seconds: an elderly passenger standing takes a free seat - and
    /// with none free, a younger one sitting stands up for them; else one passenger standing
    /// takes a seat somebody got up from. One at a time, so they do not all rush at once.
    pub fn reseat_player_bus(&mut self, dt: f32, buses: &[super::BusNow], bus_ix: &hashbrown::HashMap<super::BusId, usize>) {
        use super::{pax::Task, BusId, State};
        if !self.seating {
            return;
        }
        self.reseat_t -= dt;
        if self.reseat_t > 0.0 {
            return;
        }
        self.reseat_t = RESEAT_EVERY;
        let Some(bn) = bus_ix.get(&BusId::Player).map(|k| &buses[*k]) else { return };
        if bn.speed.abs() < 1.0 && bn.entry_open.iter().chain(&bn.exit_open).any(|o| *o) {
            return;
        }
        let places = &bn.cabin.seats;
        let n = places.len();
        let mut taken = self.seats.get(&BusId::Player).cloned().unwrap_or_default();
        taken.resize(n, false);
        let off = |k: usize| bn.places_off.get(k).copied().unwrap_or(false);
        let free: Vec<usize> = (0..n).filter(|&k| !taken[k] && !off(k)).collect();
        // the riders at their places: (person, place, elderly)
        let riders: Vec<(usize, usize, bool)> = self
            .people
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match &p.state {
                State::Pax(x) if x.inside == Some(BusId::Player) && x.task == Task::SittingInBus => x.seat.filter(|k| *k < n).map(|k| (i, k, p.age >= ELDERLY)),
                _ => None,
            })
            .collect();
        let map = SeatMap::of(places, bn.cabin.data.driver_positions.first().map(|d| Vec3::from(d.pos)));
        let pick = self.rand() as usize;
        // an elderly passenger standing
        if let Some(&(old, at, _)) = riders.iter().find(|(_, k, e)| *e && !places[*k].seated) {
            if let Some(k) = map.choose(&free, &taken, places, true, true, pick).filter(|k| places[*k].seated) {
                self.move_rider(old, k, bn);
                return;
            }
            // nothing free: the younger one sitting nearest stands up and takes a standing place
            let standing: Vec<usize> = free.iter().copied().filter(|k| !places[*k].seated).collect();
            let near = |k: usize| (places[k].pos - places[at].pos).length();
            let young = riders.iter().filter(|(_, k, e)| !*e && places[*k].seated).min_by(|a, b| near(a.1).total_cmp(&near(b.1))).copied();
            if let (Some((y, seat, _)), false) = (young, standing.is_empty()) {
                // (the standing place nearest the seat)
                let to = standing.iter().copied().min_by(|a, b| (places[*a].pos - places[seat].pos).length().total_cmp(&(places[*b].pos - places[seat].pos).length())).unwrap();
                self.move_rider(y, to, bn);
                self.move_rider(old, seat, bn);
                log::info!("player bus: a passenger stands up for an elderly one (seat {seat})");
            }
            return;
        }
        // somebody standing takes a free seat
        let standing_riders: Vec<usize> = riders.iter().filter(|(_, k, _)| !places[*k].seated).map(|r| r.0).collect();
        if standing_riders.is_empty() {
            return;
        }
        if let Some(k) = map.choose(&free, &taken, places, true, false, pick).filter(|k| places[*k].seated) {
            let who = standing_riders[pick % standing_riders.len()];
            self.move_rider(who, k, bn);
        }
    }

    /// (the fork) Rider `i` of the player's bus goes from their place to place `to`: the old
    /// one is free, the new one theirs, and they walk there along the cabin's paths.
    fn move_rider(&mut self, i: usize, to: usize, bn: &super::BusNow) {
        use super::pax::{Task, TICKET_NONE};
        let Some(old) = self.pax(i).and_then(|x| x.seat) else { return };
        if let Some(t) = self.seats.get_mut(&super::BusId::Player).and_then(|v| v.get_mut(to)) {
            *t = true;
        }
        self.free_seat(super::BusId::Player, old);
        let from = bn.cabin.seats.get(old).map(|s| s.floor).unwrap_or(Vec3::ZERO);
        let all = bn.cabin.all_points();
        let start = bn.cabin.omsi_nearest(from, &all, false, true, None, None);
        let at = start.and_then(|k| bn.cabin.graph.points.get(k)).copied();
        let Some(p) = self.pax_mut(i) else { return };
        p.seat = Some(to);
        p.pt = start;
        if let Some(q) = at {
            p.pos = q.as_dvec3();
        }
        p.seat_h = 0.0;
        p.pax_state = 1.0;
        p.ticket = TICKET_NONE;
        p.task = Task::InBusToPlace;
        self.route_to_place(i, bn);
    }
}

/// Whether two seats face the same way (their headings within 30°).
fn same_way(a: f32, b: f32) -> bool {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d) < 30.0
}

/// The middle of the cabin across: halfway between its outermost places.
fn centre_x(places: &[Seat]) -> f32 {
    let (lo, hi) = places.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p.pos.x), hi.max(p.pos.x)));
    if lo <= hi {
        (lo + hi) * 0.5
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seat(x: f32, y: f32) -> Seat {
        Seat { pos: Vec3::new(x, y, 0.9), floor: Vec3::new(x, y, 0.4), rot: 0.0, seated: true, height: 0.45, omsi_seat: 0, switch_var: None, taken_var: None, group: 0 }
    }

    fn stand(x: f32, y: f32) -> Seat {
        Seat { seated: false, height: 0.0, ..seat(x, y) }
    }

    /// A small bus: the driver front left; a single seat front right by the door; then two
    /// rows of pairs left and right of the aisle; two standing places in the aisle.
    fn bus() -> (Vec<Seat>, Vec3) {
        let places = vec![
            seat(0.9, 4.2),   // 0: next to the driver
            seat(-1.0, 2.0),  // 1: left pair, window
            seat(-0.55, 2.0), // 2: left pair, aisle
            seat(0.55, 2.0),  // 3: right pair, aisle
            seat(1.0, 2.0),   // 4: right pair, window
            seat(-1.0, 1.2),  // 5
            seat(-0.55, 1.2), // 6
            stand(0.0, 3.0),  // 7
            stand(0.0, 0.5),  // 8
        ];
        (places, Vec3::new(-0.8, 4.5, 1.0))
    }

    #[test]
    fn the_cabin_knows_its_pairs_windows_and_the_seat_by_the_driver() {
        let (places, driver) = bus();
        let m = SeatMap::of(&places, Some(driver));
        assert_eq!(m.beside[1], Some(2));
        assert_eq!(m.beside[3], Some(4));
        assert_eq!(m.beside[0], None, "a single seat");
        assert!(m.window[1] && !m.window[2] && m.window[4] && !m.window[3] && m.window[0]);
        assert!(!m.window[7], "a standing place is no window seat");
        assert_eq!(m.by_driver, Some(0));
    }

    #[test]
    fn people_take_a_window_seat_of_an_empty_pair_and_leave_the_drivers_neighbour_to_the_elderly() {
        let (places, driver) = bus();
        let m = SeatMap::of(&places, Some(driver));
        let mut taken = vec![false; places.len()];
        let free = |taken: &[bool]| (0..places.len()).filter(|k| !taken[*k]).collect::<Vec<_>>();
        // an empty bus: a window seat of a pair, never the one by the driver
        for pick in 0..6 {
            let k = m.choose(&free(&taken), &taken, &places, true, false, pick).unwrap();
            assert!([1, 4, 5].contains(&k), "{k}");
        }
        // somebody in 1 and in 4: the next takes the last empty pair's window (5), not 2 or 3
        taken[1] = true;
        taken[4] = true;
        assert_eq!(m.choose(&free(&taken), &taken, &places, true, false, 0), Some(5));
        // an elderly passenger: the seat next to the driver
        assert_eq!(m.choose(&free(&taken), &taken, &places, false, true, 3), Some(0));
        // every pair has somebody: beside a stranger by the window rather than ... (2, 3 and 6 are aisle seats)
        taken[5] = true;
        let k = m.choose(&free(&taken), &taken, &places, true, false, 0).unwrap();
        assert!([2, 3, 6].contains(&k), "{k}");
        // only the seat by the driver left: a younger one takes it rather than stand
        for k in [2, 3, 6] {
            taken[k] = true;
        }
        assert_eq!(m.choose(&free(&taken), &taken, &places, true, false, 0), Some(0));
        // standing wanted: a standing place
        let k = m.choose(&free(&taken), &taken, &places, false, false, 1).unwrap();
        assert!(!places[k].seated);
    }
}
