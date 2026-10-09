//! Night tours: trips a .ttl gives past 24:00 run the morning after a day of the tour's
//! mask. A Friday tour (784) with trips at 24:10 and 25:10 runs them on Saturday at 0:10 and
//! 1:10; a Thursday (776) or Saturday (800) tour's do not run on Friday evening, and the
//! Friday tour's bus goes on with its night trips past midnight.

use super::*;
use parking_lot::Mutex;

struct NightWorld {
    dir: PathBuf,
    ai: omsi_map::AiLists,
    positions: Mutex<HashMap<i64, (DVec3, [f64; 3])>>,
}

impl TimetableWorld for NightWorld {
    fn root(&self) -> &Path {
        &self.dir
    }
    fn map_dir(&self) -> &Path {
        &self.dir
    }
    fn chrono_dirs(&self) -> Vec<PathBuf> {
        Vec::new()
    }
    fn ailists(&self) -> &omsi_map::AiLists {
        &self.ai
    }
    fn raw_tiles(&self) -> Vec<(i32, i32)> {
        Vec::new()
    }
    fn has_tile(&self, _: (i32, i32)) -> bool {
        false
    }
    fn date(&self) -> i32 {
        20261009
    }
    fn object_positions(&self) -> MutexGuard<'_, HashMap<i64, (DVec3, [f64; 3])>> {
        self.positions.lock()
    }
    fn stop_side(&self, _: i64) -> f32 {
        0.0
    }
}

/// Tours of line TEST, the trip `T` (40 min): Friday's at 22:00, 23:00, 24:10, 25:10, and
/// Thursday's and Saturday's five and ten minutes later.
fn night_map() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("openomsi-night-{}", std::process::id()));
    let tt = dir.join("TTData");
    std::fs::create_dir_all(&tt).unwrap();
    std::fs::write(tt.join("T.ttp"), "[trip]\r\n\r\nNochem\r\nTEST\r\n\r\n[profile]\r\nStandard\r\n40\r\n").unwrap();
    let mut ttl = String::new();
    for (name, mask, off) in [("TEST-FRE", 784, 0), ("TEST-TOR", 776, 5), ("TEST-LOR", 800, 10)] {
        ttl += &format!("[newtour]\r\n{name}\r\nBus\r\n{mask}\r\n\r\n");
        for m in [1320, 1380, 1450, 1510] {
            ttl += &format!("[addtrip]\r\nT\r\n0\r\n{}.000\r\n\r\n", m + off);
        }
    }
    std::fs::write(tt.join("TEST.ttl"), ttl).unwrap();
    dir
}

fn friday_evening() -> (ScheduleSim, PathBuf) {
    let dir = night_map();
    let world = NightWorld { dir: dir.clone(), ai: Default::default(), positions: Mutex::new(HashMap::new()) };
    let mut clock = crate::SimClock::default();
    clock.set_date(2026, 10, 9); // a Friday
    assert_eq!(clock.weekday(), 4);
    clock.time = 22.0 * 3600.0 + 45.0 * 60.0;
    let mut s = ScheduleSim::new(&dir, &world, &clock);
    s.begin_tick(clock.time);
    (s, dir)
}

fn dep(s: &ScheduleSim, tour: &str, minute: i32) -> usize {
    let at = (minute % 1440) as f64 * 60.0;
    (0..s.departures.len())
        .find(|&i| s.departures[i].tour == tour && (s.departures[i].time - at).abs() < 1.0 && s.departures[i].night == (minute >= 1440))
        .unwrap_or_else(|| panic!("no departure {tour} {minute}"))
}

fn tour<'a>(s: &'a ScheduleSim, name: &str) -> &'a omsi_timetable::Tour {
    s.data.lines.iter().flat_map(|l| &l.tours).find(|t| t.number == name).unwrap()
}

#[test]
fn a_night_tour_runs_on_its_own_day_and_on_past_midnight() {
    let (mut s, dir) = friday_evening();
    assert_eq!(s.departures.len(), 12);
    // Friday evening: only Friday's tour runs, its night trips not yet
    assert!(s.runs(dep(&s, "TEST-FRE", 1380)));
    assert!(!s.runs(dep(&s, "TEST-TOR", 1385)));
    assert!(!s.runs(dep(&s, "TEST-LOR", 1390)));
    let fre_night = dep(&s, "TEST-FRE", 1450);
    assert!(!s.runs(fre_night) && s.runs_tonight(fre_night));
    // Thursday's night trips ran this morning; Saturday's do not run tonight
    assert!(!s.runs_tonight(dep(&s, "TEST-TOR", 1455)));
    assert!(!s.runs_tonight(dep(&s, "TEST-LOR", 1460)));
    // the lists offer Friday's tour only (Thursday's night is over, Saturday's not begun)
    assert!(s.tour_available(tour(&s, "TEST-FRE")));
    assert!(!s.tour_available(tour(&s, "TEST-TOR")));
    assert!(!s.tour_available(tour(&s, "TEST-LOR")));
    // the 23:00 bus goes on with the 24:10 trip, which leaves after midnight
    assert_eq!(s.tour_next[dep(&s, "TEST-FRE", 1380)], Some(fre_night));
    assert_eq!(s.dep_time(fre_night), DAY + 10.0 * 60.0);
    assert!(s.may_take_on(fre_night, 23.0 * 3600.0 + 50.0 * 60.0));
    // Saturday 0:20: the Friday tour's night trips run, Saturday's own tour waits for the evening
    s.begin_tick(DAY + 20.0 * 60.0);
    assert_eq!(s.day_base, DAY);
    assert!(s.runs(fre_night));
    assert_eq!(s.dep_time(fre_night), DAY + 10.0 * 60.0);
    assert!(s.runs(dep(&s, "TEST-FRE", 1510)));
    assert!(!s.runs(dep(&s, "TEST-TOR", 1515)));
    assert!(!s.runs(dep(&s, "TEST-LOR", 1460)));
    assert!(s.runs(dep(&s, "TEST-LOR", 1390)));
    assert!(s.tour_available(tour(&s, "TEST-FRE")));
    assert!(!s.tour_available(tour(&s, "TEST-TOR")));
    std::fs::remove_dir_all(dir).ok();
}
