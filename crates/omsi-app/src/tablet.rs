//! (the fork) The tablet: a web IBIS on a phone or tablet beside the screen. With the setting
//! `tablet` on, the game serves a small web page on the local network (port `tablet_port`,
//! 8095 unless set): a driver's login (any number does), the lines of the map's timetable
//! that the bus's depot file knows, a line's trips of the day in both directions - one
//! tapped is taken on as the duty, the bus showing its destination - and the duty under way
//! with its stops, times and delay.
//!
//! The pictures the page is drawn on are the player's own, in the folder `WebIBIS` of the
//! OMSI folder (`login.png`, 1280 x 720: the login screen; its top strip is the bar of the
//! other screens): none are shipped. Without them the page draws plain screens.
//!
//! The frame hands the state over at most twice a second ([`publish`]); the server answers
//! on threads of its own from the newest one. What only the game can answer (the lines, a
//! line's trips, taking a trip on) waits in a queue the frame empties ([`take_asks`]).

use crate::nav_duty::{self, DutyState, SheetRow, StopState, TripState};
use crate::schedule::PlayerDuty;
use omsi_ui::Color;
use serde_json::{json, Value};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The port unless the settings name another.
pub const DEFAULT_PORT: u16 = 8095;
const EVERY: Duration = Duration::from_millis(500);
/// How long a request waits for the game to answer it.
const ANSWER_WITHIN: Duration = Duration::from_secs(4);
/// The page.
const PAGE: &str = include_str!("tablet.html");
/// The folder of the page's pictures, in the OMSI folder.
pub const PICTURES: &str = "WebIBIS";

static ON: AtomicBool = AtomicBool::new(false);
static PORT: AtomicU16 = AtomicU16::new(DEFAULT_PORT);
/// The newest state as JSON.
static STATE: Mutex<Vec<u8>> = Mutex::new(Vec::new());
/// Where the page is to be opened (`http://192.168.1.20:8095`), or why it cannot be.
static WHERE: Mutex<Option<Result<String, String>>> = Mutex::new(None);
/// The OMSI folder (its `WebIBIS` has the pictures).
static ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);
/// What the page asked that the game answers, with where the answer goes.
static ASKS: Mutex<Vec<(Ask, Sender<Value>)>> = Mutex::new(Vec::new());

/// A question only the game can answer.
#[derive(Debug, Clone, PartialEq)]
pub enum Ask {
    /// The lines to pick from.
    Lines,
    /// A line's trips of the day.
    Trips(String),
    /// Take on trip `pos` (in order of departure) of tour `tour` of line `line`.
    Start { line: String, tour: String, pos: usize },
    /// (the fork) The console's GPS key: off puts the duty into manual stop selection,
    /// where the stops are served by hand instead of by where the bus is.
    Gps(bool),
    /// (the fork) With the GPS off: **Ankommet** (true) or **Afgået** (false).
    AtStop(bool),
    /// (the fork) With the GPS off: the bus is driving to this stop of the current trip.
    HeadFor(usize),
    /// (the fork) The card reader setup's keys (the manual's page 33).
    Readers(String),
    /// (the fork) KONTANTBILLET (the manual's pages 13 to 18): a cash ticket sold, which
    /// hands one to whoever stands waiting for it at the cash desk.
    Sell,
    /// (the fork) **Afslut tur**: the trip given back, so another can be taken on.
    EndTrip,
}

/// Switch the page on or off, on `port` (from the settings; live).
pub fn set(on: bool, port: u16) {
    PORT.store(if port == 0 { DEFAULT_PORT } else { port }, Ordering::Relaxed);
    ON.store(on, Ordering::Relaxed);
    if on {
        server();
    }
}

/// The OMSI folder, for the pictures.
pub fn set_root(root: &Path) {
    *ROOT.lock().unwrap_or_else(|e| e.into_inner()) = Some(root.to_path_buf());
}

/// The HUD's line while the page is on: where to open it.
pub fn hud_line() -> Option<String> {
    if !ON.load(Ordering::Relaxed) {
        return None;
    }
    match WHERE.lock().unwrap_or_else(|e| e.into_inner()).as_ref()? {
        Ok(url) => Some(format!("Web-IBIS: {url}")),
        Err(why) => Some(format!("Web-IBIS: {why}")),
    }
}

/// The questions waiting for the game (the frame answers each, then sends the answer).
/// (the fork) Ask the game for something without waiting for the answer, for the console's
/// own screen in the bus: it is drawn **on** the game's thread, so it cannot stand still
/// waiting for that thread to come round and answer. The reply is dropped.
pub fn ask_later(q: Ask) {
    let (tx, _) = channel();
    ASKS.lock().unwrap_or_else(|e| e.into_inner()).push((q, tx));
}

pub fn take_asks() -> Vec<(Ask, Sender<Value>)> {
    if !ON.load(Ordering::Relaxed) {
        return Vec::new();
    }
    std::mem::take(&mut *ASKS.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Put a question to the game and wait for its answer.
fn ask(q: Ask) -> Value {
    let (tx, rx) = channel();
    ASKS.lock().unwrap_or_else(|e| e.into_inner()).push((q, tx));
    rx.recv_timeout(ANSWER_WITHIN).unwrap_or_else(|_| json!({ "ok": false, "error": "the game did not answer" }))
}

/// The server thread, started the first time the page is switched on: it listens while the
/// page is on (on the port the settings have now) and lets go of the port while it is off.
fn server() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        let _ = std::thread::Builder::new().name("tablet".into()).spawn(|| loop {
            if !ON.load(Ordering::Relaxed) {
                *WHERE.lock().unwrap_or_else(|e| e.into_inner()) = None;
                std::thread::sleep(Duration::from_millis(300));
                continue;
            }
            let port = PORT.load(Ordering::Relaxed);
            let listener = match TcpListener::bind(("0.0.0.0", port)) {
                Ok(l) => l,
                Err(e) => {
                    log::warn!("tablet: port {port} not to be had: {e}");
                    *WHERE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(format!("port {port} is taken")));
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };
            let url = format!("http://{}:{port}", address());
            log::info!("tablet: the web IBIS at {url}");
            *WHERE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Ok(url));
            let _ = listener.set_nonblocking(true);
            while ON.load(Ordering::Relaxed) && PORT.load(Ordering::Relaxed) == port {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = std::thread::Builder::new().name("tablet-client".into()).spawn(move || answer(stream));
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(40)),
                    Err(e) => {
                        log::info!("tablet: {e}");
                        std::thread::sleep(Duration::from_millis(200));
                    }
                }
            }
        });
    });
}

/// This computer's address on the local network, as the tablet reaches it (the home
/// network's before a VPN's).
fn address() -> String {
    let all = omsi_net::addrs::joinable_addresses();
    all.iter()
        .find(|a| a.kind == omsi_net::addrs::AddrKind::Lan)
        .or(all.first())
        .map(|a| a.ip.to_string())
        .unwrap_or_else(|| "127.0.0.1".into())
}

/// One request, read whole (its head and, by its `Content-Length`, its body), answered.
fn answer(mut stream: TcpStream) {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut want = None;
    loop {
        if want.is_none() {
            if let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buf[..end]).to_ascii_lowercase();
                let length = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
                want = Some(end + 4 + length.min(64 * 1024));
            }
        }
        if want.is_some_and(|w| buf.len() >= w) || buf.len() > 80 * 1024 {
            break;
        }
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
    let _ = stream.write_all(&response(&String::from_utf8_lossy(&buf), &|q| ask(q), &picture));
}

/// A picture of the `WebIBIS` folder by its file name (none outside it).
fn picture(name: &str) -> Option<Vec<u8>> {
    if name.is_empty() || name.starts_with('.') || !name.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)) {
        return None;
    }
    let root = ROOT.lock().unwrap_or_else(|e| e.into_inner()).clone()?;
    std::fs::read(root.join(PICTURES).join(name)).ok()
}

/// `a%20b+c` as `a b c`.
fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match (b[i], b.get(i + 1).copied().and_then(hex), b.get(i + 2).copied().and_then(hex)) {
            (b'+', ..) => out.push(b' '),
            (b'%', Some(h), Some(l)) => {
                out.push(h * 16 + l);
                i += 2;
            }
            (c, ..) => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `key` of a `a=1&b=2` form.
fn field(form: &str, key: &str) -> Option<String> {
    form.split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (url_decode(k) == key).then(|| url_decode(v))
    })
}

/// The answer to a request: `ask` puts a question to the game, `picture` reads a picture.
fn response(request: &str, ask: &dyn Fn(Ask) -> Value, picture: &dyn Fn(&str) -> Option<Vec<u8>>) -> Vec<u8> {
    let (head, body) = request.split_once("\r\n\r\n").unwrap_or((request, ""));
    let mut words = head.lines().next().unwrap_or("").split_whitespace();
    let (method, target) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let json_of = |v: Value| ("200 OK", "application/json; charset=utf-8", serde_json::to_vec(&v).unwrap_or_default());
    let (status, kind, bytes): (&str, &str, Vec<u8>) = match (method, path) {
        ("GET", "/" | "/index.html") => ("200 OK", "text/html; charset=utf-8", PAGE.as_bytes().to_vec()),
        ("GET", "/state") => {
            let s = STATE.lock().unwrap_or_else(|e| e.into_inner()).clone();
            ("200 OK", "application/json; charset=utf-8", if s.is_empty() { b"{}".to_vec() } else { s })
        }
        ("GET", "/lines") => json_of(ask(Ask::Lines)),
        ("GET", "/trips") => match field(query, "line") {
            Some(line) => json_of(ask(Ask::Trips(line))),
            None => ("400 Bad Request", "text/plain; charset=utf-8", b"which line?".to_vec()),
        },
        ("POST", "/start") => match (field(body, "line"), field(body, "tour"), field(body, "pos").and_then(|p| p.parse().ok())) {
            (Some(line), Some(tour), Some(pos)) => json_of(ask(Ask::Start { line, tour, pos })),
            _ => ("400 Bad Request", "text/plain; charset=utf-8", b"which trip?".to_vec()),
        },
        // (the fork) the console's GPS key and, with it off, its manual stop selection
        ("POST", "/gps") => json_of(ask(Ask::Gps(field(body, "on").as_deref() == Some("1")))),
        ("POST", "/at_stop") => json_of(ask(Ask::AtStop(field(body, "at").as_deref() == Some("1")))),
        ("POST", "/readers") => match field(body, "op") {
            Some(op) => json_of(ask(Ask::Readers(op))),
            None => ("400 Bad Request", "text/plain; charset=utf-8", b"which key?".to_vec()),
        },
        // (the fork) KONTANTBILLET: the ticket is the console's own, the customer waiting
        // for it at the cash desk is the game's
        ("POST", "/sell") => json_of(ask(Ask::Sell)),
        // (the fork) A key of the console's own screens. The console decides what it means
        // (`crate::console`) - the page only says which key it was, so that the screen in
        // the bus can say the same thing and the two cannot drift apart.
        ("POST", "/press") => match field(body, "key") {
            Some(key) => {
                let did = crate::console::press(&key);
                // a ticket sold has to reach the game, where a customer may be waiting,
                // and so does a trip given back
                let sold = did == crate::console::Did::Sold && ask(Ask::Sell)["ok"] == Value::Bool(true);
                if did == crate::console::Did::Ended {
                    ask(Ask::EndTrip);
                }
                // a trip chosen on either screen is taken on by the game
                if did == crate::console::Did::Chose {
                    if let Some((line, tour, pos)) = crate::console::with(|c| c.take_chosen()) {
                        ask(Ask::Start { line, tour, pos });
                    }
                }
                // the console comes back with the answer: a key press is not worth a
                // second journey for the whole state, which carries every word with it
                let con = crate::console::with(|c| c.value());
                json_of(json!({ "ok": did != crate::console::Did::Nothing, "sold": sold, "con": con }))
            }
            None => ("400 Bad Request", "text/plain; charset=utf-8", b"which key?".to_vec()),
        },
        // (the fork) Afslut tur: the trip given back to the game, not only to the page
        ("POST", "/end_trip") => json_of(ask(Ask::EndTrip)),
        ("POST", "/head_for") => match field(body, "stop").and_then(|s| s.parse().ok()) {
            Some(stop) => json_of(ask(Ask::HeadFor(stop))),
            None => ("400 Bad Request", "text/plain; charset=utf-8", b"which stop?".to_vec()),
        },
        ("GET", p) if p.starts_with("/tex/") => match picture(&p[5..]) {
            Some(b) => {
                let kind = if p.to_ascii_lowercase().ends_with(".png") { "image/png" } else { "image/jpeg" };
                ("200 OK", kind, b)
            }
            None => ("404 Not Found", "text/plain; charset=utf-8", b"no such picture".to_vec()),
        },
        ("GET", _) => ("404 Not Found", "text/plain; charset=utf-8", b"not here".to_vec()),
        _ => ("405 Method Not Allowed", "text/plain; charset=utf-8", b"no".to_vec()),
    };
    let mut out = format!("HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", bytes.len()).into_bytes();
    out.extend_from_slice(&bytes);
    out
}

/// Called every frame: the state for the page, at most twice a second while it is on.
/// (the fork) The console's row of card readers, as the manual's pages 23, 27 and 28 have
/// it: a box for each of the last few cards held to a reader, the newest first, with the
/// customer type written in it and a cross over it when the card was checked out.
///
/// The game has no Rejsekort and so no card types to colour the boxes by: every card here
/// is the blue personal one, which is what most of them are. Who got on and off is real,
/// though - the riders are told apart by their ids, whatever door they took - and the
/// customer type is the passenger's own age.
/// (the fork) What a reader in the bus is set to do, as the manual's page 33 has it: take
/// a card in (Ci) or take one out (Cu).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Reads {
    In,
    Out,
}

struct Cards {
    /// Who was in the bus when this was last looked at (id and age, by id).
    seen: Vec<(u32, f32)>,
    /// What was last held to each reader: (customer type, card type, checked out, when).
    /// A box clears itself again a few seconds later, so that the next card shows as new.
    slot: Vec<Option<(String, &'static str, bool, Instant)>>,
    /// What each reader is set to.
    role: Vec<Reads>,
    /// A reader turned the other way round for one card only (the pram key, Annuller ind).
    once: Option<(usize, Reads)>,
    /// Which of the check-out readers the next card goes to, so they take turns.
    turn: usize,
    /// The first look only takes down who is aboard: a bus already full when the page is
    /// switched on has nobody checking in.
    begun: bool,
    /// The card last held to a reader, for the console's Information screen: (who, card
    /// type, checked out, the rider's id), and how many have been read in all - the page
    /// tells a new one by the count going up.
    read: Option<(String, &'static str, bool, u32)>,
    reads: u64,
}

impl Default for Cards {
    fn default() -> Cards {
        Cards { seen: Vec::new(), slot: vec![None; READERS], role: Cards::stock(), once: None, turn: 0, begun: false, read: None, reads: 0 }
    }
}

impl Cards {
    /// How the bus is fitted from the yard: the front reader takes cards in, the rest take
    /// them out. Nulstil puts it back to this.
    fn stock() -> Vec<Reads> {
        (0..READERS).map(|i| if i == SALES_READER { Reads::In } else { Reads::Out }).collect()
    }

    /// The reader a card of this kind is held to, and what it leaves behind there. A reader
    /// turned round for one card only is used first and then turns back.
    fn reader_for(&mut self, going_out: bool) -> Option<usize> {
        let want = if going_out { Reads::Out } else { Reads::In };
        if let Some((k, way)) = self.once {
            if way == want {
                self.once = None;
                return Some(k);
            }
        }
        let fit: Vec<usize> = (0..READERS).filter(|&k| self.role[k] == want).collect();
        if fit.is_empty() {
            return None;
        }
        self.turn = self.turn.wrapping_add(1);
        Some(fit[self.turn % fit.len()])
    }

    /// The manual's four keys on the card reader setup.
    fn set(&mut self, op: &str) -> bool {
        match op {
            "all_in" => self.role = vec![Reads::In; READERS],
            "all_out" => self.role = vec![Reads::Out; READERS],
            "reset" => {
                self.role = Cards::stock();
                self.once = None;
            }
            // the pram: reader 3, the one by the middle door a pram is wheeled in at, takes
            // one card in and is a check-out again after it. (It used to be whichever
            // check-out reader came first, which is reader 2.)
            "pram" => {
                if self.role.get(PRAM_READER) != Some(&Reads::In) {
                    self.once = Some((PRAM_READER, Reads::In));
                }
            }
            // Annuller ind: the front reader takes one card out instead
            "cancel_in" => self.once = (0..READERS).find(|&k| self.role[k] == Reads::In).map(|k| (k, Reads::Out)),
            _ => return false,
        }
        true
    }
}

static CARDS: Mutex<Option<Cards>> = Mutex::new(None);

/// How many boxes the row holds (the manual's picture has five).
const READERS: usize = 5;

/// (the fork) Reader 1 is the sales reader: it is the one the driver holds a card to for
/// the Information screen and for Check ind ekstra, and the one a cash ticket is sold
/// over. A fault on it will take cash tickets with it, once the console's faults are
/// built.
const SALES_READER: usize = 0;

/// (the fork) The reader the pram key turns round for one card: reader 3, by the middle
/// door a pram is wheeled in at.
const PRAM_READER: usize = 2;

/// (the fork) How long a card stands in its box before the reader is white again. The box
/// is the driver's glance at whether the customer remembered, not a record to keep.
const CARD_SHOWS: Duration = Duration::from_secs(5);

/// (the fork) Which card a rider is carrying, as the manual's page 28 colours them: blue
/// for the personal card and the season tickets, green for the anonymous one, yellow for
/// flex, and now and then a red one the system says is barred.
///
/// The game has no Rejsekort to ask, so the console's own system decides - but it decides
/// the same thing about the same person every time, from their id, so a card cannot change
/// colour between the customer checking in and checking out again.
fn card_type(id: u32) -> &'static str {
    // (a cheap spread, so that neighbouring ids do not all draw the same card)
    match (id.wrapping_mul(2_654_435_761) >> 13) % 100 {
        0 => "blocked",
        1..=14 => "flex",
        15..=34 => "anon",
        _ => "personal",
    }
}

/// (the fork) A spread of a rider's id, so that two riders beside each other do not share
/// a card number or a purse.
fn spread(id: u32, salt: u32) -> u32 {
    (id ^ salt).wrapping_mul(2_246_822_519).rotate_left(13).wrapping_mul(3_266_489_917) >> 8
}

/// (the fork) What the console's Information screen knows about a rider's card (the
/// manual's pages 35 to 37). The game has no Rejsekort, so the card is the console's own
/// invention - but it is the same invention about the same person every time.
fn card_facts(id: u32, year: i32) -> Value {
    let n = spread(id, 0x1234) % 1_000_000;
    let number = format!("308430 400 {:03} {:03} {}", n / 1000, n % 1000, spread(id, 0x55) % 10);
    let purse = spread(id, 0x99) % 40_000;
    json!({
        "no": number,
        "exp": format!("{:02}.{:02}.{}", spread(id, 7) % 28 + 1, spread(id, 11) % 12 + 1, year + 2 + (spread(id, 3) % 4) as i32),
        "purse": format!("{},{:02} kr.", purse / 100, purse % 100),
    })
}

/// The customer type of a passenger that age, in the manual's own words.
fn customer(age: f32) -> &'static str {
    match age {
        a if a < 16.0 => "Barn",
        a if a < 26.0 => "Ung",
        a if a >= 65.0 => "Pensionist",
        _ => "Voksen",
    }
}

/// The row of boxes after `riders` got on and off since it was last looked at: a card
/// checked in goes to a reader set to take them in, one checked out to a reader set to
/// take them out, and the box shows what each reader last had held to it.
fn cards_of(cards: &mut Cards, riders: &[(u32, f32)]) -> Vec<Value> {
    let same = cards.seen.len() == riders.len() && cards.seen.iter().zip(riders).all(|(a, b)| a.0 == b.0);
    if !same {
        if cards.begun {
            let has = |v: &[(u32, f32)], id: u32| v.binary_search_by_key(&id, |(i, _)| *i).is_ok();
            let mut held: Vec<(u32, f32, bool)> = riders.iter().filter(|(id, _)| !has(&cards.seen, *id)).map(|(i, a)| (*i, *a, false)).collect();
            held.extend(cards.seen.iter().filter(|(id, _)| !has(riders, *id)).map(|(i, a)| (*i, *a, true)));
            for (id, age, out) in held {
                if let Some(k) = cards.reader_for(out) {
                    cards.slot[k] = Some((customer(age).to_string(), card_type(id), out, Instant::now()));
                    cards.read = Some((customer(age).to_string(), card_type(id), out, id));
                    cards.reads += 1;
                }
            }
        }
        cards.seen = riders.to_vec();
        cards.begun = true;
    }
    // (a card that has stood its few seconds leaves the box white again)
    for slot in cards.slot.iter_mut() {
        if slot.as_ref().is_some_and(|(_, _, _, at)| at.elapsed() >= CARD_SHOWS) {
            *slot = None;
        }
    }
    (0..READERS)
        .map(|k| {
            let role = if cards.role[k] == Reads::In { "in" } else { "out" };
            match &cards.slot[k] {
                Some((who, card, out, _)) => json!({ "who": omsi_ui::tr(who).into_owned(), "card": card, "out": out, "role": role }),
                None => json!({ "role": role }),
            }
        })
        .collect()
}

/// (the fork) The card reader setup's keys (the manual's page 33). True when the key meant
/// something.
/// (the fork) Which way round each reader in the bus is set, for the standers drawn in it
/// (`crate::readers`): true takes cards in (Ci, the green **Check ind** skin), false takes
/// them out (Cu, the red **Check ud**).
pub fn reader_roles() -> Vec<bool> {
    let mut c = CARDS.lock().unwrap_or_else(|e| e.into_inner());
    let c = c.get_or_insert_with(Cards::default);
    (0..READERS).map(|k| c.role.get(k) == Some(&Reads::In)).collect()
}

pub fn set_readers(op: &str) -> bool {
    let mut c = CARDS.lock().unwrap_or_else(|e| e.into_inner());
    c.get_or_insert_with(Cards::default).set(op)
}

pub fn publish(duty: Option<&PlayerDuty>, clock: &omsi_sim::SimClock, bus: Option<glam::DVec3>, speed_kmh: f32, passengers: Option<usize>, riders: &[(u32, f32)], paused: bool) {
    // (the fork) the console's own screen in the bus draws the same thing the tablet
    // does, so the state is built for it as well - whether or not the tablet is on
    let on = ON.load(Ordering::Relaxed);
    if !on && !crate::desk::wanted() {
        return;
    }
    static LAST: Mutex<Option<Instant>> = Mutex::new(None);
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if last.is_some_and(|t| t.elapsed() < EVERY) {
            return;
        }
        *last = Some(Instant::now());
    }
    let (day, month) = clock.day_month();
    let date = format!("{day}/{month}/{:02}", clock.year.rem_euclid(100));
    let mut v = state(duty, clock.time, &date, bus, speed_kmh, passengers, paused);
    // (the fork) what the console has decided, for whichever screen is drawing it. The
    // game tells it whether a trip is being driven: Billetter has nothing to sell against
    // until one is, and ESC puts the console down on the driving screen rather than on
    // the list of lines.
    v["con"] = crate::console::with(|c| {
        c.set_duty(duty.is_some());
        c.value()
    });
    // (the fork) the console's row of card readers, from who came and went in the bus
    {
        let mut guard = CARDS.lock().unwrap_or_else(|e| e.into_inner());
        let c = guard.get_or_insert_with(Cards::default);
        v["cards"] = Value::Array(cards_of(c, riders));
        // (the fork) the card last held to a reader, for the Information screen
        if let Some((who, card, out, id)) = c.read.clone() {
            let mut facts = card_facts(id, clock.year);
            facts["who"] = Value::String(omsi_ui::tr(&who).into_owned());
            facts["card"] = Value::String(card.to_string());
            facts["out"] = Value::Bool(out);
            facts["n"] = Value::from(c.reads);
            v["read"] = facts;
        }
    }
    // (the fork) and the lists for whichever screen the console stands on, which the
    // tablet fetches for itself but the screen in the bus cannot
    if let (Some(Value::Object(lists)), Some(o)) = (LISTS.lock().unwrap_or_else(|e| e.into_inner()).clone(), v.as_object_mut()) {
        o.extend(lists);
    }
    if on {
        *STATE.lock().unwrap_or_else(|e| e.into_inner()) = serde_json::to_vec(&v).unwrap_or_default();
    }
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(v);
}

/// (the fork) The lists the console's screen in the bus needs just now, laid out by the
/// frame (`game_lists::console_lists`) and sent down with the state.
static LISTS: Mutex<Option<Value>> = Mutex::new(None);

/// The frame hands over the lists for whichever screen the console stands on.
pub fn offer_lists(v: Value) {
    *LISTS.lock().unwrap_or_else(|e| e.into_inner()) = v.as_object().is_some().then_some(v);
}

/// (the fork) The last state built, as the console's screen in the bus reads it. The
/// tablet has it as JSON over the network; the screen in the bus is in the same process,
/// so it is handed the thing itself.
static LIVE: Mutex<Option<Value>> = Mutex::new(None);

/// Run `f` on the last state built, when there is one.
pub fn with_live<T>(f: impl FnOnce(&Value) -> T) -> Option<T> {
    LIVE.lock().unwrap_or_else(|e| e.into_inner()).as_ref().map(f)
}

/// A colour as CSS.
fn css(c: Color) -> String {
    let [r, g, b, _] = c.0.map(|x| (x.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// How early or late, by name (the page colours it for its own background).
fn tone(delay: f64) -> &'static str {
    match nav_duty::punctuality(delay) {
        nav_duty::Punctuality::Early => "early",
        nav_duty::Punctuality::OnTime => "ontime",
        nav_duty::Punctuality::Late => "late",
    }
}

fn state_name(s: StopState) -> &'static str {
    match s {
        StopState::Done => "done",
        StopState::Now => "now",
        StopState::Ahead => "ahead",
    }
}

/// The page's own words, in the interface's language.
///
/// They are written as a list rather than as one `json!` object: the macro builds an
/// object by one expansion step for every field, and the list has outgrown the recursion
/// limit the doctests are built with twice already.
fn labels() -> Value {
    let tr = |k: &str| omsi_ui::tr(k).into_owned();
    let words: &[(&str, &str)] = &[
        ("in_game", "In the game"),
        ("delay", "Delay"),
        ("trip", "Trip"),
        ("terminus", "terminus"),
        ("next_stop", "Next stop"),
        ("no_duty", "No duty running"),
        ("passengers", "Passengers"),
        ("paused", "Paused"),
        ("pick_line", "Choose a line"),
        ("line", "Line"),
        ("tour", "Tour"),
        // (the fork) the console's own word for the trip it is driving. Asked for "Tour",
        // OMSI's Danish table answers "Vagt" - which is OMSI's word for a vehicle's whole
        // duty and not this console's for one trip of it. The Rejsekort console says Tur,
        // so the word is written here in Danish, as the rest of its own words are.
        ("rk_tour", "Tur"),
        ("all", "All"),
        ("start_q", "You have chosen this trip. Is that right?"),
        ("question", "Question"),
        ("warning", "Warning"),
        ("end_trip", "End trip"),
        ("arrived", "Arrived"),
        ("departed", "Departed"),
        ("manual_pos", "((41)) : Manual position"),
        ("menu", "Menu"),
        ("lock_console", "Lock console"),
        ("driver_change", "Driver change"),
        ("telebus", "Telebus"),
        ("set_sound", "Set sound"),
        ("maintenance", "Maintenance"),
        // the sound settings, as the manual's page 30 has them
        ("snd_key", "Key active"),
        ("snd_dead", "Key not active"),
        ("snd_fault", "Fault"),
        ("volume", "Volume"),
        // the Betjening menu, as the manual's page 32 has it
        ("sale_card", "Rejsekort sale"),
        ("top_up", "Top up"),
        ("reader_setup", "Card reader setup"),
        ("delay_key", "Delay"),
        ("information", "Information"),
        ("cancel_sale", "Cancel sale"),
        ("checkin_extra", "Extra check in"),
        ("readers_on", "Card readers active"),
        ("readers_off", "Card readers not active"),
        // KONTANTBILLET, the manual's pages 13 to 18. These are Midttrafik's own words for
        // its own ticket, and are written in Danish here for the same reason the Rejsekort
        // screen's are: the game's language tables have no word for any of them, so a
        // translation would only have left the screen half in English.
        ("single", "Enkeltbillet"),
        ("zone_count", "Antal zoner"),
        ("dep_zone", "Afgangszone"),
        ("more", "Flere..."),
        ("choice", "Valg"),
        ("destination", "Destination"),
        ("adult", "Voksen"),
        ("child", "Barn"),
        ("bike", "Cykel"),
        ("void_q", "Bekræft annullering?"),
        // CHECK IND EKSTRA, the manual's pages 24 to 26
        ("temp_change", "Midlertidig ændring"),
        ("hold_rk", "Hold rejsekort hen på salgskortlæser"),
        ("hold_again", "Hold rejsekortet hen på salgskortlæseren igen"),
        ("z_local", "Lokal"),
        ("z_far", "Ml. landsdele"),
        ("dog", "Hund"),
        ("extra_done", "Kortet er checket ind. Ændringen slettes, når kortet checkes ud."),
        // the card reader setup, as the manual's page 33 has it
        ("reader_head", "Card reader setup"),
        ("all_in", "All in"),
        ("all_out", "All out"),
        ("cancel_in", "Cancel in"),
        ("reset", "Reset"),
        // the Information screen, as the manual's pages 35 to 37 have it
        ("hold_card", "Hold a card to the sales card reader"),
        ("t_general", "Generelt"),
        ("t_purse", "E-pung"),
        ("t_period", "Periodekort"),
        ("t_trips", "Rejser"),
        ("t_other", "Andet"),
        ("f_type", "Korttype"),
        ("f_exp", "Expiry date"),
        ("f_lang", "Language"),
        ("f_service", "Service level"),
        ("f_scope", "Type"),
        ("f_no", "Card number"),
        ("f_customer", "Customer type"),
        ("f_balance", "Balance"),
        ("f_none", "None"),
        ("f_checked_in", "Checked in"),
        ("f_checked_out", "Checked out"),
        ("c_personal", "Personligt"),
        ("c_flex", "Flex"),
        ("c_anon", "Anonymt"),
        ("c_blocked", "Barred"),
        ("v_danish", "Dansk"),
        ("v_standard", "Standard"),
        ("v_scope", "Landsdækkende / Extension"),
        ("test", "Test"),
        ("save", "Save"),
        ("console_locked", "The driver console is locked"),
        ("lock_hint", "Type the password\nPress ESC for a driver change"),
        ("yes", "Yes"),
        ("no", "No"),
        ("duty", "Duty"),
        ("lines", "Lines"),
        ("log_out", "Log out"),
        ("no_lines", "No timetable on this map"),
        ("no_trips", "No trips today"),
        ("lost", "No connection to the game"),
        ("driver", "Driver number"),
        ("password", "Password"),
        ("login_hint", "Type a driver number and a password (any will do)"),
        // the console's start-up, as the manual's page 7 has it
        ("starting", "Starting managers… Please wait…"),
        ("updating", "Update under way.\nIt takes 3 minutes at most"),
        ("no_wifi_title", "No wi-fi"),
        ("no_wifi", "The console has no wi-fi connection. Press OK to go on."),
    ];
    Value::Object(words.iter().map(|(k, w)| ((*k).to_string(), Value::String(tr(w)))).collect())
}

/// What the page shows, in the interface's language: the texts are made here, the page only
/// lays them out.
fn state(duty: Option<&PlayerDuty>, time: f64, date: &str, bus: Option<glam::DVec3>, speed_kmh: f32, passengers: Option<usize>, paused: bool) -> Value {
    let clock = {
        let s = time.rem_euclid(86_400.0) as i64;
        format!("{:02}:{:02}", s / 3600, s / 60 % 60)
    };
    let mut v = json!({
        "clock": clock,
        "date": date,
        "paused": paused,
        "speed": speed_kmh.max(0.0).round(),
        "passengers": passengers,
        "labels": labels(),
        "duty": Value::Null,
    });
    let Some(d) = duty else { return v };
    let Some(ds) = DutyState::of(d, time) else { return v };
    let status = ds.status();
    let (fill, ink) = nav_duty::chip_colours(status);
    let late = match status {
        nav_duty::Status::Running(l) => Some(l),
        _ => None,
    };
    let (delay, delay_tone) = match late {
        Some(l) => (nav_duty::offset(l), Some(tone(l))),
        None => ("–".to_string(), None),
    };
    let rows: Vec<Value> = nav_duty::sheet(&ds)
        .into_iter()
        .map(|row| match row {
            SheetRow::Trip { departure, arrival, line, terminus, stops, state, .. } => json!({
                "k": "trip",
                "dep": nav_duty::hhmm(departure),
                "arr": nav_duty::hhmm(arrival),
                "sub": nav_duty::trip_sub(line.as_deref(), stops),
                "line": line,
                "terminus": terminus,
                "state": match state { TripState::Done => "done", TripState::Now => "now", TripState::Ahead => "ahead" },
            }),
            SheetRow::Stop { index, planned, name, state, last } => {
                // (off the plan: when the bus will be there, for the stops still to come)
                let coming = state == StopState::Ahead || (state == StopState::Now && !ds.at_stop);
                let expected = late.filter(|_| coming).map(|l| planned + l).filter(|e| nav_duty::hhmm(*e) != nav_duty::hhmm(planned));
                json!({
                    "k": "stop",
                    // (the fork) the console sends this back when a stop is picked by hand
                    "i": index,
                    "time": nav_duty::hhmm(planned),
                    "exp": expected.map(nav_duty::hhmm),
                    "exp_tone": late.map(tone),
                    "name": name,
                    "state": state_name(state),
                    "last": last,
                })
            }
            SheetRow::Pause { minutes } => json!({ "k": "pause", "text": nav_duty::tr_with("%{m} min break", &[("m", minutes.to_string())]) }),
            SheetRow::Change { line, tour } => json!({ "k": "change", "text": nav_duty::tr_with("Continue on line %{line}, tour %{tour}", &[("line", line), ("tour", tour)]) }),
        })
        .collect();
    // the stop the bus heads for, how far, when it is due there
    let next = d.trips.get(d.trip_index).filter(|_| !d.trip_done()).and_then(|t| t.stops.get(d.next_stop)).map(|s| {
        let dist = s.position.zip(bus).map(|(p, b)| (p - b).truncate().length());
        json!({
            "name": s.name.trim(),
            "dist": dist.map(|m| if m >= 1000.0 { format!("{:.1} km", m / 1000.0) } else { format!("{:.0} m", m) }),
            "time": nav_duty::hhmm(s.arr),
        })
    });
    let trip = d.trips.get(d.trip_index);
    v["duty"] = json!({
        "title": nav_duty::sheet_title(&ds),
        "span": nav_duty::sheet_span(&ds),
        "line": trip.map(|t| if t.line.trim().is_empty() { d.line.trim().to_string() } else { t.line.trim().to_string() }),
        "terminus": trip.map(|t| t.terminus.trim().to_string()),
        "status": { "text": status.label(), "fill": css(fill), "ink": css(ink) },
        "delay": delay,
        "delay_tone": delay_tone,
        // (the fork) the console's driving screen wants the tour by number, and the hand
        // it draws beside a stop means the bus is standing at it
        "tour": d.tour.trim(),
        "at_stop": ds.at_stop,
        // (the fork) the trip is over - the bus has reached its last stop. The console
        // ends the trip itself when it sees this, as it would on Afslut tur.
        "done": d.trip_done(),
        // (the fork) the GPS key's own state: off, and the stops are served by hand
        "gps": !d.manual_stops,
        // (the fork) the fare zone the bus stands in, for the cash ticket's Afgangszone
        // (the manual's page 15). OMSI knows nothing of fare zones, so the console works
        // one out from how far along the trip the bus has come: the manual's page 13 has
        // every Midttrafik zone beginning with a 4, and 4301 for the heart of Aarhus.
        "zone": 4301 + (d.next_stop as u32 / 3).min(98),
        "trip": format!("{} / {}", ds.focus() + 1, ds.trips.len()),
        "next": next,
        "rows": rows,
    });
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(request: &str) -> String {
        let asked = Mutex::new(Vec::new());
        let ask = |q: Ask| {
            asked.lock().unwrap().push(q.clone());
            json!({ "asked": format!("{q:?}") })
        };
        let picture = |name: &str| (name == "login.png").then(|| b"PNG".to_vec());
        String::from_utf8_lossy(&response(request, &ask, &picture)).into_owned()
    }

    #[test]
    fn it_serves_the_page_the_state_and_the_pictures() {
        let page = get("GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        assert!(page.starts_with("HTTP/1.1 200 OK\r\nContent-Type: text/html"), "{page}");
        assert!(page.contains("/state") && page.contains("/lines") && page.contains("/start"));
        assert!(get("GET /state?t=1 HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 200 OK\r\nContent-Type: application/json"));
        let png = get("GET /tex/login.png HTTP/1.1\r\n\r\n");
        assert!(png.starts_with("HTTP/1.1 200 OK\r\nContent-Type: image/png") && png.ends_with("PNG"), "{png}");
        assert!(get("GET /tex/other.png HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404"));
        assert!(get("GET /../settings.cfg HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 404"));
        assert!(get("DELETE /state HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 405"));
    }

    #[test]
    fn the_game_is_asked_for_the_lines_the_trips_and_to_take_one_on() {
        assert!(get("GET /lines HTTP/1.1\r\n\r\n").ends_with(r#"{"asked":"Lines"}"#));
        assert!(get("GET /trips?line=6%20N HTTP/1.1\r\n\r\n").ends_with(r#"{"asked":"Trips(\"6 N\")"}"#));
        let started = get("POST /start HTTP/1.1\r\nContent-Length: 30\r\n\r\nline=143&tour=143H01&pos=11");
        assert!(started.ends_with(r#"{"asked":"Start { line: \"143\", tour: \"143H01\", pos: 11 }"}"#), "{started}");
        assert!(get("POST /start HTTP/1.1\r\n\r\nline=143").starts_with("HTTP/1.1 400"));
    }

    #[test]
    fn pictures_come_only_from_their_folder() {
        for bad in ["", "../x.png", "a/b.png", ".hidden", "x\\y.png"] {
            assert!(picture(bad).is_none(), "{bad}");
        }
        assert_eq!(url_decode("Siedlung+B%C3%B6sch%2"), "Siedlung Bösch%2");
        assert_eq!(field("a=1&line=13%2B", "line").as_deref(), Some("13+"));
    }

    /// (the fork) The page asks for its words by name, and the names are written out twice:
    /// once in the page and once in [`labels`]. A word the page asks for and the game does
    /// not send would stand as the Danish the page was written with, so an English player
    /// would meet a Danish word and nobody would see why. The start-up screen asks for two
    /// of them from its script rather than from the markup, so they are named here as well.
    #[test]
    fn every_word_the_page_asks_for_is_sent_to_it() {
        let sent = labels();
        let mut asked = Vec::new();
        for piece in PAGE.split(r#"data-l=""#).skip(1) {
            asked.push(piece.split('"').next().unwrap_or_default());
        }
        assert!(asked.len() >= 8, "only {} words asked for - has the page lost its texts?", asked.len());
        // the ones the page's script asks for by name rather than through the markup
        asked.extend([
            "starting", "updating", "driver", "password", "login_hint", "lost",
            "no_duty", "line", "tour", "rk_tour", "delay", "end_trip",
            // the cash ticket and Check ind ekstra write the customer types, and the
            // console's own messages, into their screens from the script
            "adult", "child", "bike", "dog", "hold_rk", "hold_again", "extra_done",
            "readers_on", "readers_off",
        ]);
        for key in &asked {
            assert!(!sent[key].is_null(), "the page asks for {key:?} and labels() has no such word");
        }
        // and the screens the manual settles, so a rebuild cannot quietly drop one
        for key in ["starting", "updating", "no_wifi", "question", "warning", "start_q"] {
            assert!(asked.contains(&key), "the page lost {key:?}");
        }
    }

    /// (the fork) The pram key turns reader 3 round for one card - the reader by the middle
    /// door a pram is wheeled in at - and it is a check-out again after that one card. It
    /// used to be whichever check-out reader came first, which is reader 2.
    #[test]
    fn the_pram_key_turns_reader_three_round_for_one_card() {
        let mut c = Cards::default();
        assert!(c.set("pram"));
        assert_eq!(c.once, Some((2, Reads::In)), "the pram belongs to reader 3");
        // that one card goes to reader 3, and the next one checking in to reader 1 again
        assert_eq!(c.reader_for(false), Some(2));
        assert_eq!(c.reader_for(false), Some(SALES_READER));
        // reader 1 is the sales reader and takes cards in from the yard
        assert_eq!(Cards::stock()[SALES_READER], Reads::In);

        // with every reader taking cards in there is nothing for the key to turn round
        let mut c = Cards::default();
        assert!(c.set("all_in"));
        assert!(c.set("pram"));
        assert_eq!(c.once, None);
    }

    /// (the fork) A card stands in its box for a few seconds and the box is white again.
    /// The first player found it standing there for good: it had never been written to
    /// clear at all.
    #[test]
    fn a_card_stands_in_its_box_and_then_the_box_is_white_again() {
        let mut cards = Cards::default();
        // (the first look only takes down who is aboard)
        cards_of(&mut cards, &[(1, 40.0)]);
        let row = cards_of(&mut cards, &[(1, 40.0), (2, 9.0)]);
        let shown: Vec<&str> = row.iter().filter_map(|c| c["who"].as_str()).collect();
        assert_eq!(shown, ["Barn"], "the child who got on should stand in a box: {row:?}");

        // the same row a moment later: still there
        let row = cards_of(&mut cards, &[(1, 40.0), (2, 9.0)]);
        assert!(row.iter().any(|c| c["who"].as_str() == Some("Barn")), "gone too soon: {row:?}");

        // and once its few seconds are up
        for slot in cards.slot.iter_mut() {
            if let Some((_, _, _, at)) = slot {
                *at = at.checked_sub(CARD_SHOWS + Duration::from_secs(1)).expect("a time that far back");
            }
        }
        let row = cards_of(&mut cards, &[(1, 40.0), (2, 9.0)]);
        assert!(row.iter().all(|c| c["who"].is_null()), "the boxes should be white again: {row:?}");
        // (and a reader still says which way round it is)
        assert!(row.iter().all(|c| c["role"].is_string()), "{row:?}");
    }

    /// (the fork) KONTANTBILLET, the manual's pages 13 to 18: the ticket's zones and its
    /// fare are the console's own, but selling it has to reach the game, where a customer
    /// may be standing at the cash desk waiting for his ticket.
    #[test]
    fn selling_a_cash_ticket_reaches_the_game() {
        let sold = get("POST /sell HTTP/1.1\r\nContent-Length: 10\r\n\r\nprice=22.00");
        assert!(sold.ends_with(r#"{"asked":"Sell"}"#), "{sold}");
        // the page itself presses keys; selling is what the console makes of the press
        assert!(PAGE.contains("/press"), "the page has lost the way to press a key");
        assert!(PAGE.contains(r#"press("ok")"#), "the page has lost the key that sells");
    }

    /// (the fork) **Afslut tur** has to reach the game too. It used to take the console
    /// back to Turvalg and leave the bus driving the trip behind it.
    #[test]
    fn ending_the_trip_reaches_the_game() {
        let ended = get("POST /end_trip HTTP/1.1\r\n\r\n");
        assert!(ended.ends_with(r#"{"asked":"EndTrip"}"#), "{ended}");
        assert!(PAGE.contains("/end_trip"), "the page has lost the way to end the trip");
    }

    /// (the fork) The cash ticket's Afgangszone: the manual's page 15 has the console know
    /// which zone the bus stands in, and the zone rises as the bus drives out of town.
    #[test]
    fn the_cash_ticket_is_told_which_fare_zone_the_bus_is_in() {
        let (trips, _) = crate::nav_duty::tests::duty();
        let mut d = PlayerDuty::new("35".into(), "35H01".into(), trips, 1, 0, false);
        let when = 12.0 * 3600.0 + 4.0 * 60.0;
        let zone_now = |d: &PlayerDuty| state(Some(d), when, "11/10/26", None, 0.0, Some(0), false)["duty"]["zone"].as_u64();
        d.next_stop = 0;
        assert_eq!(zone_now(&d), Some(4301), "every Midttrafik zone begins with a 4");
        d.next_stop = 12;
        assert!(zone_now(&d).is_some_and(|z| z > 4301), "the zone should rise along the line");
    }

    #[test]
    fn without_a_duty_the_state_says_so() {
        let v = state(None, 3600.0 * 15.0 + 61.0, "11/10/26", None, 12.4, Some(3), false);
        assert_eq!(v["clock"], "15:01");
        assert_eq!(v["date"], "11/10/26");
        assert_eq!(v["speed"], 12.0);
        assert!(v["duty"].is_null());
    }

    /// (`OMSI_TABLET_PREVIEW=<folder> cargo test -p omsi-app --lib tablet -- --ignored`: the
    /// state, lines and trips of the duty sheet's test duty, for a look at the page)
    #[test]
    #[ignore]
    fn preview_state() {
        let Ok(dir) = std::env::var("OMSI_TABLET_PREVIEW") else { return };
        crate::ui_language("DAN");
        let (trips, _) = crate::nav_duty::tests::duty();
        let mut d = PlayerDuty::new("35".into(), "35H01".into(), trips, 1, 0, false);
        d.next_stop = 1;
        let v = state(Some(&d), 12.0 * 3600.0 + 4.0 * 60.0 + 12.0, "11/10/26", None, 32.0, Some(17), false);
        std::fs::write(format!("{dir}/state.json"), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    }
}
