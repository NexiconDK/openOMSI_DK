//! The journey's timetable (#800, #1061): the stops of the duty's trips with their planned
//! and actual arrival and departure and how far off those were, as the logbooks of the
//! virtual bus companies keep them. It is written to the content folder's `Journeys` every
//! time the bus reaches or leaves a stop, so it is there however the session ends.

use crate::career::{EARLY_DEPARTURE, LATE_ARRIVAL};
use crate::schedule::{PlannedTrip, PlayerDuty};
use std::path::PathBuf;

/// One stop of a trip in the log.
struct Row {
    trip: usize,
    stop: usize,
    name: String,
    /// Planned arrival and departure, and the actual ones (s of the duty's day).
    plan: (f64, f64),
    arrived: Option<f64>,
    left: Option<f64>,
    /// The trip begins here (no arrival to tell) / ends here (no departure).
    first: bool,
    last: bool,
    /// (the fork) The passengers who got on and off here, and the odometer (km) when the
    /// bus stood here (`trip_summary`).
    on: u32,
    off: u32,
    odo: Option<f64>,
}

/// (the fork) One trip as the summary after it shows it (`Journey::summary`).
pub(crate) struct TripSummary {
    /// The lines above (driver, map, bus, date; line and tour) and the trip's title.
    pub head: String,
    pub title: String,
    /// Per stop: name, arrival planned, real, difference, departure planned, real,
    /// difference, got on, got off, odometer; then the status and how it is coloured
    /// (0 OK, 1 late or early, 2 missed, 3 not yet).
    pub rows: Vec<([String; 10], String, u8)>,
    pub footer: String,
    /// All of it as text (copied, saved).
    pub text: String,
}

pub(crate) struct Journey {
    /// The duty it is the log of: line, tour and where its trips begin in the tour.
    key: (String, String, usize),
    /// The lines above the trips (driver, map, bus, date; line and tour).
    head: String,
    /// Each trip's title.
    trips: Vec<String>,
    /// (the fork) The same in parts: number, line, terminus, departure (the summary's own
    /// title in the interface's language).
    titles: Vec<(usize, String, String, String)>,
    rows: Vec<Row>,
    dir: PathBuf,
    /// The file, chosen when there is first something to write.
    path: Option<PathBuf>,
    /// (the fork) The row of the stop the bus stood at last, and the people who had got
    /// into the bus and the riders when last looked (`count_people`).
    at_row: Option<usize>,
    people: Option<(u32, usize)>,
}

impl Journey {
    /// The log of the duty `key` (line, tour, where its `trips` begin in the tour), nothing
    /// driven yet; `head` comes first in it.
    fn new(key: (String, String, usize), trips_planned: &[PlannedTrip], head: &str, dir: PathBuf) -> Journey {
        let mut trips = Vec::new();
        let mut titles = Vec::new();
        let mut rows = Vec::new();
        for (t, trip) in trips_planned.iter().enumerate() {
            let line = if trip.line.trim().is_empty() { key.0.trim() } else { trip.line.trim() };
            trips.push(format!("Trip {}  ·  {} to {}  ·  {}", key.2 + t + 1, line, trip.terminus.trim(), hms(trip.departure)));
            titles.push((key.2 + t + 1, omsi_sim::timetable_run::display_line(line).to_string(), trip.terminus.trim().to_string(), hms(trip.departure)));
            let n = trip.stops.len();
            // (the stations a depot run passes are not stops)
            for (k, s) in trip.stops.iter().enumerate().filter(|(_, s)| s.stops) {
                rows.push(Row { trip: t, stop: k, name: s.name.trim().to_string(), plan: (s.arr, s.dep), arrived: None, left: None, first: k == 0, last: k + 1 == n, on: 0, off: 0, odo: None });
            }
        }
        let head = format!("{}\nLine {}, tour {}", head.trim_end(), key.0.trim(), key.1.trim());
        Journey { key, head, trips, titles, rows, dir, path: None, at_row: None, people: None }
    }

    /// The bus arrived at stop `at` (trip, stop) `late` s after its time (negative: early).
    /// True when that is news.
    fn arrived(&mut self, at: (usize, usize), late: f64) -> bool {
        match self.row(at) {
            Some(r) if r.arrived.is_none() => {
                r.arrived = Some(r.plan.0 + late);
                true
            }
            _ => false,
        }
    }

    /// The bus left stop `at`, having arrived and left `late` s after its times.
    fn left(&mut self, at: (usize, usize), late: (f64, f64)) -> bool {
        let Some(r) = self.row(at) else { return false };
        r.arrived.get_or_insert(r.plan.0 + late.0);
        r.left = Some(r.plan.1 + late.1);
        true
    }

    fn row(&mut self, (trip, stop): (usize, usize)) -> Option<&mut Row> {
        self.rows.iter_mut().find(|r| r.trip == trip && r.stop == stop)
    }

    /// (the fork) The bus stands at stop `at` (trip, stop), the odometer at `odo` km: the
    /// people who get on and off from now on are its.
    fn standing_at(&mut self, at: (usize, usize), odo: Option<f64>) {
        let Some(i) = self.rows.iter().position(|r| r.trip == at.0 && r.stop == at.1) else { return };
        self.at_row = Some(i);
        if self.rows[i].odo.is_none() {
            self.rows[i].odo = odo;
        }
    }

    /// (the fork) `stepped_in`: everybody who got into the bus so far; `riders`: those in it
    /// now. Who got on since the last look, and who got off (those in it before and since
    /// got on, less those in it now), are the stop's where the bus stood last (nobody gets
    /// on or off between stops).
    fn count_people(&mut self, stepped_in: u32, riders: usize) {
        if let (Some((was_in, was_riding)), Some(i)) = (self.people, self.at_row) {
            let on = stepped_in.saturating_sub(was_in);
            let off = (was_riding + on as usize).saturating_sub(riders) as u32;
            self.rows[i].on += on;
            self.rows[i].off += off;
        }
        self.people = Some((stepped_in, riders));
    }

    /// (the fork) Trip `t` of the duty (its index in the duty's trips) as the summary
    /// after it shows it; None for a trip not in this log.
    pub(crate) fn summary(&self, t: usize) -> Option<TripSummary> {
        let t = t.checked_sub(self.key.2).filter(|t| *t < self.trips.len())?;
        let rows: Vec<&Row> = self.rows.iter().filter(|r| r.trip == t).collect();
        let tr = |s: &str| omsi_ui::tr(s).into_owned();
        let (mut served, mut late, mut early, mut on, mut off) = (0, 0, 0, 0, 0);
        let mut out = Vec::new();
        for (i, r) in rows.iter().enumerate() {
            let arr = if r.first { Default::default() } else { times(r.plan.0, r.arrived) };
            let dep = if r.last { Default::default() } else { times(r.plan.1, r.left) };
            let is_late = !r.first && r.arrived.is_some_and(|a| (a - r.plan.0).round() > LATE_ARRIVAL);
            let is_early = !r.last && r.left.is_some_and(|l| (l - r.plan.1).round() < EARLY_DEPARTURE);
            on += r.on;
            off += r.off;
            let (status, colour) = if r.arrived.is_some() || r.left.is_some() {
                served += 1;
                late += is_late as usize;
                early += is_early as usize;
                match (is_late, is_early) {
                    (true, true) => ("late, left early", 1),
                    (true, false) => ("late", 1),
                    (false, true) => ("left early", 1),
                    _ => ("OK", 0),
                }
            } else if rows[i + 1..].iter().any(|x| x.arrived.is_some() || x.left.is_some()) {
                ("missed", 2)
            } else {
                ("", 3)
            };
            let count = |n: u32, seen: bool| if seen { n.to_string() } else { String::new() };
            let seen = r.arrived.is_some() || r.left.is_some();
            let [a0, a1, a2] = arr;
            let [d0, d1, d2] = dep;
            out.push(([r.name.clone(), a0, a1, a2, d0, d1, d2, count(r.on, seen), count(r.off, seen), r.odo.map(|k| format!("{k:.1}")).unwrap_or_default()], tr(status), colour));
        }
        let odo: Vec<f64> = rows.iter().filter_map(|r| r.odo).collect();
        let km = match (odo.first(), odo.last()) {
            (Some(a), Some(b)) => format!(", {}: {:.1} km", tr("distance"), (b - a).max(0.0)),
            _ => String::new(),
        };
        let footer = format!(
            "{}: {served} {} {}, {}: {late}, {}: {early}, {}: {on}, {}: {off}{km}",
            tr("Stops served"),
            tr("of"),
            rows.len(),
            tr("late"),
            tr("left early"),
            tr("got on"),
            tr("got off")
        );
        let (n, line, terminus, dep) = &self.titles[t];
        let title = format!("{} {n}  ·  {line} {} {terminus}  ·  {dep}", tr("Trip"), tr("to"));
        // as text: the columns lined up
        let heads = ["Stop", "Arr. plan", "Arr. real", "Diff", "Dep. plan", "Dep. real", "Diff", "Pax in", "Pax out", "Odo km"].map(tr);
        let widths = [30usize, 9, 9, 10, 9, 9, 10, 4, 4, 10];
        let line = |cells: &[String], status: &str| {
            let mut l = String::new();
            for (k, c) in cells.iter().enumerate() {
                let c: String = c.chars().take(widths[k]).collect();
                if k == 0 {
                    l.push_str(&format!("{c:<w$}", w = widths[k]));
                } else {
                    l.push_str(&format!("  {c:>w$}", w = widths[k]));
                }
            }
            format!("{}   {status}", l).trim_end().to_string()
        };
        let rule = "-".repeat(widths.iter().sum::<usize>() + 2 * (widths.len() - 1) + 12);
        let mut text = format!("{}\n\n{title}\n{rule}\n{}\n{rule}\n", self.head, line(&heads, &tr("Status")));
        for (cells, status, _) in &out {
            text.push_str(&line(cells, status));
            text.push('\n');
        }
        text.push_str(&format!("{rule}\n{footer}\n"));
        Some(TripSummary { head: self.head.clone(), title, rows: out, footer, text })
    }

    /// (the fork) The summary of trip `t` written beside the log; the file it went to.
    pub(crate) fn save_summary(&self, t: usize) -> Option<PathBuf> {
        let s = self.summary(t)?;
        let (y, mo, d, h, mi) = omsi_launcher_lib::local_now().unwrap_or((1970, 1, 1, 0, 0));
        let clean = |s: &str| s.trim().chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect::<String>();
        let stem = format!("{y:04}-{mo:02}-{d:02} {h:02}-{mi:02} line {} tour {} trip {}", clean(&self.key.0), clean(&self.key.1), t + 1);
        std::fs::create_dir_all(&self.dir).ok()?;
        let path = (1..100).map(|n| self.dir.join(if n == 1 { format!("{stem}.txt") } else { format!("{stem} ({n}).txt") })).find(|p| !p.exists())?;
        match std::fs::write(&path, s.text) {
            Ok(()) => Some(path),
            Err(e) => {
                log::warn!("trip summary: {}: {e}", path.display());
                None
            }
        }
    }

    /// The log as it is written.
    fn text(&self) -> String {
        let rule = "-".repeat(108);
        let mut out = format!("openOMSI journey log\n{}\n", self.head);
        let (mut served, mut late, mut early) = (0, 0, 0);
        for (t, title) in self.trips.iter().enumerate() {
            let rows: Vec<&Row> = self.rows.iter().filter(|r| r.trip == t).collect();
            // (a trip is in it once the bus has reached one of its stops: a tour has dozens)
            if rows.iter().all(|r| r.arrived.is_none() && r.left.is_none()) {
                continue;
            }
            out.push_str(&format!("\n{title}\n{rule}\n{:<30}  {:>9} {:>9} {:>10}   {:>9} {:>9} {:>10}   Status\n{rule}\n", "Stop", "Arr. plan", "Arr. real", "Diff", "Dep. plan", "Dep. real", "Diff"));
            for (i, r) in rows.iter().enumerate() {
                let arr = if r.first { [String::new(), String::new(), String::new()] } else { times(r.plan.0, r.arrived) };
                let dep = if r.last { [String::new(), String::new(), String::new()] } else { times(r.plan.1, r.left) };
                let is_late = !r.first && r.arrived.is_some_and(|a| (a - r.plan.0).round() > LATE_ARRIVAL);
                let is_early = !r.last && r.left.is_some_and(|l| (l - r.plan.1).round() < EARLY_DEPARTURE);
                let status = if r.arrived.is_some() || r.left.is_some() {
                    served += 1;
                    late += is_late as usize;
                    early += is_early as usize;
                    match (is_late, is_early) {
                        (true, true) => "late, left early",
                        (true, false) => "late",
                        (false, true) => "left early",
                        _ => "OK",
                    }
                } else if rows[i + 1..].iter().any(|x| x.arrived.is_some() || x.left.is_some()) {
                    "missed"
                } else {
                    ""
                };
                let name: String = r.name.chars().take(30).collect();
                out.push_str(format!("{name:<30}  {:>9} {:>9} {:>10}   {:>9} {:>9} {:>10}   {status}", arr[0], arr[1], arr[2], dep[0], dep[1], dep[2]).trim_end());
                out.push('\n');
            }
        }
        out.push_str(&format!("{rule}\nStops served: {served}, late: {late}, left early: {early}\n"));
        out
    }

    /// Write the log (its file is named by the real date and time it began, and the line and tour).
    fn write(&mut self) {
        if self.path.is_none() {
            let (y, mo, d, h, mi) = omsi_launcher_lib::local_now().unwrap_or((1970, 1, 1, 0, 0));
            let clean = |s: &str| s.trim().chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect::<String>();
            let stem = format!("{y:04}-{mo:02}-{d:02} {h:02}-{mi:02} line {} tour {}", clean(&self.key.0), clean(&self.key.1));
            if let Err(e) = std::fs::create_dir_all(&self.dir) {
                log::warn!("journey log: {}: {e}", self.dir.display());
                return;
            }
            // (another session begun in the same minute keeps its own)
            let path = (1..100).map(|n| self.dir.join(if n == 1 { format!("{stem}.txt") } else { format!("{stem} ({n}).txt") })).find(|p| !p.exists());
            let Some(path) = path else { return };
            log::info!("journey log: {}", path.display());
            self.path = Some(path);
        }
        if let Some(p) = self.path.as_ref() {
            if let Err(e) = std::fs::write(p, self.text()) {
                log::warn!("journey log: {}: {e}", p.display());
            }
        }
    }
}

/// Keep `slot` the log of `duty` and note in it what the duty's update did, written at once
/// when that changed it: the bus left the stop it was due at before the update, `due`,
/// `served` seconds late (arrival, departure: what `PlayerDuty::update` returned), and
/// arrived at the one it is due at now (`PlayerDuty::arrived`). `head` (driver, map, bus,
/// date) is asked for when a log begins.
#[allow(clippy::too_many_arguments)]
pub(crate) fn note(slot: &mut Option<Journey>, duty: &PlayerDuty, due: (usize, usize), served: Option<(f64, f64)>, root: &std::path::Path, people: (u32, usize), odo: Option<f64>, head: impl FnOnce() -> String) {
    // (every frame: the key is compared, not made)
    if slot.as_ref().is_none_or(|j| (j.key.0.as_str(), j.key.1.as_str(), j.key.2) != (duty.line.as_str(), duty.tour.as_str(), duty.first_trip)) {
        let dir = crate::startup::content_dir().unwrap_or_else(|| root.to_path_buf()).join("Journeys");
        *slot = Some(Journey::new((duty.line.clone(), duty.tour.clone(), duty.first_trip), &duty.trips, &head(), dir));
    }
    let Some(j) = slot.as_mut() else { return };
    // (the fork) who got on and off, at the stop the bus stands at
    j.count_people(people.0, people.1);
    if duty.arrived().is_some() {
        j.standing_at((duty.trip_index, duty.next_stop), odo);
    } else if served.is_some() {
        // (left without having been seen standing there: the first stop of a duty)
        if j.at_row.is_none_or(|i| (j.rows[i].trip, j.rows[i].stop) != due) {
            j.standing_at(due, odo);
        }
    }
    let left = served.is_some_and(|s| j.left(due, s));
    let arrived = duty.arrived().is_some_and(|late| j.arrived((duty.trip_index, duty.next_stop), late));
    if left || arrived {
        j.write();
    }
}

/// The log's first line: the driver, the map, the bus and the game's date.
pub(crate) fn head(career: &crate::career::Career, map: &str, bus: &omsi_sim::VehicleInstance, clock: &omsi_sim::SimClock) -> String {
    let driver = career.driver.as_ref().map(|d| d.name.clone()).or_else(|| career.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()));
    let (day, month) = clock.day_month();
    let bus = format!("{} {}", bus.ty.def.manufacturer.trim(), bus.ty.def.type_name.trim());
    let mut parts: Vec<String> = driver.map(|d| format!("Driver {}", d.trim())).into_iter().collect();
    parts.extend([map.trim().to_string(), bus.trim().to_string(), format!("{:04}-{month:02}-{day:02}", clock.year)]);
    parts.join("  ·  ")
}

/// A time of day as HH:MM:SS (a duty's times may run past midnight or before it).
fn hms(t: f64) -> String {
    let s = t.round().rem_euclid(86400.0) as i64;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// Planned, actual and the difference (+ late, − early), blank while it has not happened.
fn times(plan: f64, real: Option<f64>) -> [String; 3] {
    match real {
        Some(r) => {
            let d = (r - plan).round() as i64;
            let a = d.abs();
            [hms(plan), hms(r), format!("{}{:02}:{:02}:{:02}", if d < 0 { "-" } else { "+" }, a / 3600, a / 60 % 60, a % 60)]
        }
        None => [hms(plan), String::new(), String::new()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::PlannedStop;

    fn trip(stops: &[(&str, f64)]) -> PlannedTrip {
        PlannedTrip {
            name: "5_A".into(),
            line: "5".into(),
            terminus: "Muzealna".into(),
            departure: stops[0].1,
            end: stops[stops.len() - 1].1,
            stops: stops.iter().map(|(n, t)| PlannedStop { object_id: 0, name: n.to_string(), arr: *t, dep: *t, position: None, dir: Default::default(), stops: true }).collect(),
        }
    }

    /// A trip driven stop by stop: the first stop tells its departure only, the last its
    /// arrival only, a stop driven past is missed, a late arrival says so - and the file
    /// has it all at every stop.
    #[test]
    fn the_log_has_every_stop_with_its_times() {
        let t0 = 8.0 * 3600.0;
        let trips = [trip(&[("Metro Mlociny", t0), ("Zajezdnia", t0 + 300.0), ("Prozy", t0 + 600.0), ("Muzealna", t0 + 900.0)]), trip(&[("Muzealna", t0 + 1200.0), ("Metro Mlociny", t0 + 2100.0)])];
        let dir = std::env::temp_dir().join(format!("omsi-journey-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut j = Journey::new(("5".into(), "1".into(), 0), &trips, "Driver Test · Grundorf · MAN SD77 · 1989-05-30", dir.clone());
        assert!(j.arrived((0, 0), 20.0));
        assert!(!j.arrived((0, 0), 30.0), "the first arrival counts");
        assert!(j.left((0, 0), (20.0, 57.0)));
        j.write();
        let path = j.path.clone().expect("written");
        assert!(std::fs::read_to_string(&path).unwrap().contains("08:00:57"));
        // four minutes late at the second stop, past the third, early at the last
        assert!(j.arrived((0, 1), 240.0));
        assert!(j.left((0, 1), (240.0, 270.0)));
        assert!(j.arrived((0, 3), -30.0));
        j.write();
        let text = std::fs::read_to_string(&path).unwrap();
        let line = |name: &str| text.lines().find(|l| l.starts_with(name)).unwrap_or_else(|| panic!("no {name} in\n{text}")).to_string();
        assert!(text.contains("Line 5, tour 1") && text.contains("Driver Test") && text.contains("Trip 1  ·  5 to Muzealna  ·  08:00:00"), "{text}");
        assert!(line("Metro Mlociny").contains("08:00:00  08:00:57  +00:00:57"), "{text}");
        assert!(line("Metro Mlociny").ends_with("OK"), "{text}");
        assert!(line("Zajezdnia").contains("08:05:00  08:09:00  +00:04:00"), "{text}");
        assert!(line("Zajezdnia").contains("08:05:00  08:09:30  +00:04:30"), "{text}");
        assert!(line("Zajezdnia").ends_with("late"), "{text}");
        assert!(line("Prozy").ends_with("missed"), "{text}");
        assert!(line("Muzealna").contains("08:15:00  08:14:30  -00:00:30"), "{text}");
        assert!(!text.contains("Trip 2"), "a trip not begun is not in it yet: {text}");
        assert!(text.contains("Stops served: 3, late: 1, left early: 0"), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
        // a duty taken up at its second trip: the first is not in it
        let mut j = Journey::new(("5".into(), "1".into(), 0), &trips, "Grundorf", dir.clone());
        assert!(j.arrived((1, 0), -60.0));
        let text = j.text();
        assert!(text.contains("Trip 2  ·  5 to Muzealna  ·  08:20:00") && !text.contains("Trip 1"), "{text}");
    }

    /// (the fork) The summary of a trip: who got on and off where (the people who got in
    /// and those riding, as the frames go), the odometer and the distance.
    #[test]
    fn the_summary_counts_who_got_on_and_off_at_each_stop() {
        let t0 = 8.0 * 3600.0;
        let trips = [trip(&[("A", t0), ("B", t0 + 300.0), ("C", t0 + 600.0)])];
        let dir = std::env::temp_dir().join(format!("omsi-summary-test-{}", std::process::id()));
        let mut j = Journey::new(("5".into(), "1".into(), 0), &trips, "Grundorf", dir.clone());
        j.count_people(10, 0);
        // at A: three get on
        j.standing_at((0, 0), Some(100.0));
        j.count_people(13, 3);
        j.left((0, 0), (0.0, 10.0));
        // driving: nobody
        j.count_people(13, 3);
        // at B: one off, two on
        j.arrived((0, 1), 0.0);
        j.standing_at((0, 1), Some(101.5));
        j.count_people(15, 4);
        j.left((0, 1), (0.0, 0.0));
        // at C: everybody off
        j.arrived((0, 2), 0.0);
        j.standing_at((0, 2), Some(103.0));
        j.count_people(15, 0);
        let s = j.summary(0).expect("a summary");
        let cells = |name: &str| s.rows.iter().find(|r| r.0[0] == name).map(|r| (r.0[7].clone(), r.0[8].clone(), r.0[9].clone())).unwrap();
        assert_eq!(cells("A"), ("3".into(), "0".into(), "100.0".into()));
        assert_eq!(cells("B"), ("2".into(), "1".into(), "101.5".into()));
        assert_eq!(cells("C"), ("0".into(), "4".into(), "103.0".into()));
        assert!(s.footer.contains(": 5") && s.footer.contains("3.0 km"), "{}", s.footer);
        assert!(s.text.contains("Trip 1") && s.text.contains("B"), "{}", s.text);
        let saved = j.save_summary(0).expect("saved");
        assert!(std::fs::read_to_string(&saved).unwrap().contains("103.0"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
