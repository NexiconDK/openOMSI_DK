//! (the fork) The driver console's own decisions, in the game rather than in the page.
//!
//! The Web-IBIS began as a page that decided things for itself: what a cash ticket cost,
//! which press of OK sold it, what the number pad was typing into. That holds while there
//! is one screen. It stops holding the moment there are two - the page in a tablet, and
//! the console's own screen in the bus, which the game draws itself onto the model's screen
//! face. Two screens deciding the same things drift apart in a week, and the faults that
//! come of it are the kind nobody notices until a player does.
//!
//! So the decisions live here. A screen - either screen - draws what it is told and says
//! which key was pressed. [`Console::press`] is that whole conversation.

use serde_json::{json, Value};
use std::sync::Mutex;

/// Who is travelling on a line of the ticket. A dog travels as a child, which the manual's
/// page 17 says in so many words.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    Adult,
    Child,
    Bike,
    /// Only Check ind ekstra offers one by name (the manual's page 26); a cash ticket for
    /// a dog is written as a child's, which the manual's page 17 says in so many words.
    Dog,
}

impl Who {
    /// The name the screens ask for the word by (`labels()`).
    pub fn key(self) -> &'static str {
        match self {
            Who::Adult => "adult",
            Who::Child => "child",
            Who::Bike => "bike",
            Who::Dog => "dog",
        }
    }

    fn of(name: &str) -> Option<Who> {
        match name {
            "adult" => Some(Who::Adult),
            "child" => Some(Who::Child),
            "bike" => Some(Who::Bike),
            "dog" => Some(Who::Dog),
            _ => None,
        }
    }
}

/// (the fork) Midttrafik's own fares, read off the manual's own worked examples: 22,00 kr
/// for an adult over two zones and 10,00 for every zone after that (page 18 sells seven
/// zones for 72,00), half of that for a child, and 13,00 for a bicycle however far it goes
/// (page 18 sells one beside a two-zone adult ticket for 35,00 together).
///
/// In øre, so that the half fare of an odd number of kroner cannot be lost to a rounding.
pub fn fare(who: Who, zones: u32) -> u32 {
    let adult = 2200 + 1000 * zones.saturating_sub(2);
    match who {
        Who::Adult => adult,
        Who::Child | Who::Dog => adult / 2,
        Who::Bike => 1300,
    }
}

/// An amount in øre as the console writes it: "22,00".
pub fn kr(ore: u32) -> String {
    format!("{},{:02}", ore / 100, ore % 100)
}

/// The most that may travel on one Rejsekort (the manual's page 25).
pub const MOST: u32 = 29;

/// A cash ticket being written out, or one just sold.
#[derive(Clone, PartialEq, Debug)]
pub struct Sale {
    /// The ticket's two lines: how many, and of what.
    pub rows: [(u32, Who); 2],
    pub zones: u32,
    /// Flere...: the ticket is still being written and its fields can be pressed.
    pub edit: bool,
    /// What the customer put in the driver's hand, in øre; none while he had it counted
    /// out, and then there is nothing to give back.
    pub paid: Option<u32>,
}

impl Sale {
    /// The everyday ticket of the sales menu: one adult, or one child, over `zones` zones.
    /// No zones is Flere..., which opens the ticket to be written out by hand.
    pub fn quick(who: Who, zones: u32) -> Sale {
        Sale {
            rows: [(u32::from(who == Who::Adult), Who::Adult), (u32::from(who == Who::Child), Who::Child)],
            zones: zones.max(1),
            edit: zones == 0,
            paid: None,
        }
    }

    pub fn price(&self) -> u32 {
        self.rows.iter().map(|(n, who)| n * fare(*who, self.zones)).sum()
    }

    /// What the customer handed over: the fare itself while he had it counted out.
    pub fn paid(&self) -> u32 {
        self.paid.unwrap_or_else(|| self.price())
    }

    /// What he has to have back. Nothing when the money does not cover the fare.
    pub fn back(&self) -> u32 {
        self.paid().saturating_sub(self.price())
    }

    fn value(&self) -> Value {
        json!({
            "n1": self.rows[0].0, "t1": self.rows[0].1.key(),
            "n2": self.rows[1].0, "t2": self.rows[1].1.key(),
            "z": self.zones,
            "edit": self.edit,
            "price": kr(self.price()),
            "paid": kr(self.paid()),
            "back": kr(self.back()),
            "owed": self.back() > 0,
        })
    }
}

/// (the fork) CHECK IND EKSTRA, the manual's pages 24 to 26: where the console stands in
/// its four steps. It waits for the customer's card, puts the change up to be set, and
/// then waits for the card once more - that second hold is the check in itself.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// *Hold rejsekort hen på salgskortlæser.*
    Hold,
    /// **Midlertidig ændring**: the three slots, to be set.
    Form,
    /// Set and taken: the card is to be held up once more.
    Again,
}

/// A temporary change being made to one card.
#[derive(Clone, PartialEq, Debug)]
pub struct Extra {
    pub step: Step,
    /// The three slots: how many, and of what. The first is the card itself.
    pub rows: [(u32, Who); 3],
    /// **Ml. landsdele** rather than **Lokal**.
    pub far: bool,
    /// What the card itself was, which **Nulstil** puts it back to.
    pub was: Who,
}

impl Extra {
    /// The change as the console first puts it up: the card itself, and two empty slots.
    fn stock(was: Who) -> Extra {
        Extra { step: Step::Form, rows: [(1, was), (0, Who::Child), (0, Who::Bike)], far: false, was }
    }

    fn total(&self) -> u32 {
        self.rows.iter().map(|(n, _)| n).sum()
    }

    /// The least a slot may hold: the card holder is always travelling.
    fn least(slot: usize) -> u32 {
        u32::from(slot == 0)
    }

    /// Whether the minus or the plus key of a slot does anything at all. A key that does
    /// not is marked dead on the screen and sounds like it.
    fn can(&self, slot: usize, d: i32) -> bool {
        match self.rows.get(slot) {
            Some(&(n, _)) if d < 0 => n > Self::least(slot),
            Some(_) => self.total() < MOST,
            None => false,
        }
    }

    fn value(&self) -> Value {
        let rows: Vec<Value> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, (n, who))| json!({ "n": n, "t": who.key(), "less": self.can(i, -1), "more": self.can(i, 1) }))
            .collect();
        json!({
            "step": match self.step { Step::Hold => "hold", Step::Form => "form", Step::Again => "again" },
            "rows": rows,
            "far": self.far,
        })
    }
}

/// (the fork) The console's number pad, as the manual's page 8 has it: tapping a field
/// brings it up, the number is typed into it and ENTER takes it away again.
#[derive(Clone, PartialEq, Debug)]
pub struct Pad {
    /// Which field it is typing into, by the name the screens use for it.
    pub into: String,
    pub typed: String,
}

impl Pad {
    /// How many digits the field takes. The driver's number is eight and his pass code
    /// four, which the manual's page 8 says; the ticket's own fields are short, and the
    /// money takes a large note.
    fn room(&self) -> usize {
        match self.into.as_str() {
            "driver" => 8,
            "password" => 4,
            "sale:paid" => 4,
            _ if self.into.starts_with("vol:") => 3,
            _ => 2,
        }
    }

    /// The pass code is not shown as it is typed, any more than it is on the console.
    fn shown(&self) -> String {
        if self.into == "password" {
            "*".repeat(self.typed.len())
        } else {
            self.typed.clone()
        }
    }

    fn value(&self) -> Value {
        // ENTER with nothing typed does nothing, so it is marked dead and sounds like it
        json!({ "into": self.into, "shown": self.shown(), "enter": !self.typed.is_empty() })
    }
}

/// (the fork) One of the console's three sounds, as its sound menu sets them: on or off,
/// and how loud out of a hundred.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Sound {
    pub on: bool,
    pub vol: u32,
}

/// The three sounds, in the order the sound menu lists them: the key that does something,
/// the key that does not, and the fault.
pub const SOUNDS: [&str; 3] = ["key", "dead", "fault"];

/// Which of [`SOUNDS`] that is, if it is one of them at all.
fn sound_at(name: &str) -> Option<usize> {
    SOUNDS.iter().position(|s| *s == name)
}

/// Which of the console's screens is up. There is one console, however many screens are
/// drawing it, so the tablet and the screen in the bus always stand on the same one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Start,
    Login,
    Lines,
    Trips,
    Duty,
    Menu,
    Sound,
    Serve,
    Rds,
    Inf,
    Tix,
    Sale,
    Void,
    Extra,
    Kind,
}

impl Screen {
    pub fn key(self) -> &'static str {
        match self {
            Screen::Start => "start",
            Screen::Login => "login",
            Screen::Lines => "lines",
            Screen::Trips => "trips",
            Screen::Duty => "duty",
            Screen::Menu => "menu",
            Screen::Sound => "sound",
            Screen::Serve => "serve",
            Screen::Rds => "rds",
            Screen::Inf => "inf",
            Screen::Tix => "tix",
            Screen::Sale => "sale",
            Screen::Void => "void",
            Screen::Extra => "extra",
            Screen::Kind => "kind",
        }
    }

    fn of(name: &str) -> Option<Screen> {
        [
            Screen::Start, Screen::Login, Screen::Lines, Screen::Trips, Screen::Duty,
            Screen::Menu, Screen::Sound, Screen::Serve, Screen::Rds, Screen::Inf,
            Screen::Tix, Screen::Sale, Screen::Void, Screen::Extra, Screen::Kind,
        ]
        .into_iter()
        .find(|s| s.key() == name)
    }

    /// Which of the console's menus this screen belongs under, for the bar along the foot:
    /// the manual's page 5 puts the menu you are standing in on yellow.
    fn under(self) -> Option<&'static str> {
        match self {
            Screen::Menu | Screen::Sound => Some("menu"),
            Screen::Serve | Screen::Rds | Screen::Inf | Screen::Void | Screen::Extra | Screen::Kind => Some("serve"),
            Screen::Tix | Screen::Sale => Some("tix"),
            Screen::Lines | Screen::Trips | Screen::Duty => Some("linje"),
            _ => None,
        }
    }
}

/// How a key of the bar along the foot stands: the manual's page 5 gives the colours their
/// meaning. White is a key you can press, grey one you cannot - and it sounds dead - and
/// yellow is the menu you are standing in.
fn key_look(key: &str, screen: Screen, on_duty: bool) -> &'static str {
    if screen.under() == Some(key) {
        return "now";
    }
    // Billetter sells against the trip being driven: there is nothing to sell against
    // until one is chosen
    if key == "tix" && !on_duty {
        return "off dead";
    }
    "white"
}

/// What the console has decided so far. One of these stands for the whole session, whether
/// it is being drawn in a tablet or on the console's own screen in the bus.
#[derive(Clone, PartialEq, Debug)]
pub struct Console {
    /// The screen that is up.
    pub screen: Screen,
    /// The screen **Billetter** was pressed from, which ESC and a finished sale go back to.
    pub tix_back: Option<Screen>,
    /// Whether a trip is being driven; the game tells the console so (`set_duty`).
    pub on_duty: bool,
    /// The ticket on the screen now.
    pub sale: Option<Sale>,
    /// The ticket last sold. Only that one can be taken back, which the manual's page 19
    /// says in red, so the console keeps it and no other.
    pub last: Option<Sale>,
    /// Check ind ekstra, while it is being done.
    pub extra: Option<Extra>,
    /// (the fork) The manual's page 34: the card readers shut, with the padlock hanging on
    /// the icon. Nobody can hold a card to a shut reader.
    pub shut: bool,
    /// (the fork) **Forsinkelse** (page 32): a switch that stands on until it is put off.
    pub slow: bool,
    /// The number pad, while it is up over whichever screen asked for it.
    pub pad: Option<Pad>,
    /// The driver's number, once he has typed it. He is logged in while it is there and
    /// the console is not locked.
    pub driver: Option<String>,
    /// **Lås konsol** (the manual's pages 17 to 21): the console held over a short break,
    /// which the same driver opens again with his pass code.
    pub locked: bool,
    /// (the fork) The three sounds of the sound menu, in the order of [`SOUNDS`].
    pub sound: [Sound; 3],
}

impl Default for Console {
    fn default() -> Console {
        Console {
            screen: Screen::Start,
            tix_back: None,
            on_duty: false,
            sale: None,
            last: None,
            extra: None,
            shut: false,
            slow: false,
            pad: None,
            driver: None,
            locked: false,
            // the fault is the one the driver must hear over the bus, so it stands louder
            sound: [Sound { on: true, vol: 50 }, Sound { on: true, vol: 50 }, Sound { on: true, vol: 60 }],
        }
    }
}

/// What a press did, for the screen that sent it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Did {
    /// The key meant nothing here: it sounds dead and nothing moved.
    Nothing,
    /// The console moved; the screen draws itself again.
    Moved,
    /// A ticket was sold, so the game is to hand one to whoever waits at the cash desk.
    Sold,
    /// **Afslut tur**: the trip is given back, so the game is to end it.
    Ended,
}

impl Console {
    /// A key of the cash ticket's screens, by the name the screens use for it.
    ///
    /// * `buy:<who>:<zones>` - a zone key of the sales menu, zones 0 being Flere...
    /// * `count:<0|1>:<n>`, `who:<0|1>:<who>`, `zones:<n>`, `paid:<ore>` - the ticket's fields
    /// * `ok` - the manual's two presses: the first takes the ticket as written, the second
    ///   is the one for when the driver HAS the money in his hand
    /// * `void` - Annuller salg: the ticket last sold given back
    pub fn press(&mut self, key: &str) -> Did {
        let (verb, rest) = key.split_once(':').unwrap_or((key, ""));
        match verb {
            "buy" => {
                let (who, zones) = rest.split_once(':').unwrap_or((rest, "0"));
                let Some(who) = Who::of(who) else { return Did::Nothing };
                self.sale = Some(Sale::quick(who, zones.parse().unwrap_or(0)));
                self.screen = Screen::Sale;
                Did::Moved
            }
            "count" | "who" => {
                let (line, val) = rest.split_once(':').unwrap_or((rest, ""));
                let Ok(line) = line.parse::<usize>() else { return Did::Nothing };
                let Some(s) = self.sale.as_mut().filter(|s| s.edit && line < 2) else { return Did::Nothing };
                if verb == "count" {
                    let Ok(n) = val.parse::<u32>() else { return Did::Nothing };
                    s.rows[line].0 = n.min(MOST);
                } else {
                    let Some(who) = Who::of(val) else { return Did::Nothing };
                    s.rows[line].1 = who;
                }
                Did::Moved
            }
            "zones" => {
                let Ok(n) = rest.parse::<u32>() else { return Did::Nothing };
                let Some(s) = self.sale.as_mut().filter(|s| s.edit) else { return Did::Nothing };
                s.zones = n.clamp(1, 99);
                Did::Moved
            }
            // the white field: what the customer came with, and the field under it works
            // the change out
            "paid" => {
                let Ok(ore) = rest.parse::<u32>() else { return Did::Nothing };
                let Some(s) = self.sale.as_mut().filter(|s| !s.edit) else { return Did::Nothing };
                s.paid = Some(ore.min(999_900));
                Did::Moved
            }
            "ok" => {
                let Some(s) = self.sale.as_mut() else { return Did::Nothing };
                if s.edit {
                    // the manual's page 17: "Derefter trykker du på ok"
                    s.edit = false;
                    return Did::Moved;
                }
                self.last = self.sale.take();
                self.screen = self.tix_back.unwrap_or_else(|| self.home());
                Did::Sold
            }
            // KONTANTBILLET - ANNULLER, the manual's pages 19 and 20
            "void" => {
                if self.last.take().is_none() {
                    return Did::Nothing;
                }
                self.screen = Screen::Serve;
                Did::Moved
            }
            // (the fork) Afslut tur on the driving screen: the trip given back. The game
            // ends it and tells the console so, which takes it off the driving screen.
            "end" => {
                if !self.on_duty {
                    return Did::Nothing;
                }
                Did::Ended
            }
            // CHECK IND EKSTRA, the manual's pages 24 to 26
            "extra" => self.extra_press(rest),
            // the screen asked for by name, and ESC, which steps back out of it
            // the number pad, and the login it types into
            "pad" => self.pad_press(rest),
            "lock" => {
                self.locked = true;
                self.screen = Screen::Login;
                self.pad = Some(Pad { into: "password".into(), typed: String::new() });
                Did::Moved
            }
            // Log ud comes back to the garage and takes the number with it; Chaufførskift
            // hands the bus on and leaves the trip standing
            "logout" | "change" => {
                self.locked = false;
                if verb == "logout" {
                    self.driver = None;
                }
                self.screen = Screen::Login;
                self.pad = Some(Pad { into: "driver".into(), typed: String::new() });
                Did::Moved
            }
            "go" => match Screen::of(rest) {
                Some(s) => self.go(s),
                None => Did::Nothing,
            },
            // (the fork) a sound of the sound menu switched on or off, and `set` for the
            // screen handing back what it kept from the session before this one
            "snd" => {
                if let Some(set) = rest.strip_prefix("set:") {
                    let mut p = set.split(':');
                    let (Some(i), Some(on), Some(vol)) = (p.next().and_then(sound_at), p.next(), p.next()) else {
                        return Did::Nothing;
                    };
                    self.sound[i] = Sound { on: on == "1", vol: vol.parse().unwrap_or(50).min(100) };
                    return Did::Moved;
                }
                match sound_at(rest) {
                    Some(i) => {
                        self.sound[i].on = !self.sound[i].on;
                        Did::Moved
                    }
                    None => Did::Nothing,
                }
            }
            "esc" => {
                // the manual's page 8: "tast ESC for chaufførskift". The console is handed
                // on, so the number goes with the driver and the pad comes back up empty.
                if self.screen == Screen::Login {
                    self.driver = None;
                    self.locked = false;
                    self.pad = Some(Pad { into: "driver".into(), typed: String::new() });
                    return Did::Moved;
                }
                // the pad stands over the screen, so ESC takes the pad before the screen
                if self.pad.take().is_some() {
                    return Did::Moved;
                }
                self.screen = self.back();
                // out of the list of lines there is nowhere back but out of the console
                // altogether, and that takes the driver's number with it
                if self.screen == Screen::Login {
                    self.driver = None;
                    self.pad = Some(Pad { into: "driver".into(), typed: String::new() });
                }
                Did::Moved
            }
            // (the fork) the card reader key in the strip (page 34) and Forsinkelse (32):
            // both are switches that stand where they are put
            "shut" => {
                self.shut = !self.shut;
                Did::Moved
            }
            "slow" => {
                self.slow = !self.slow;
                Did::Moved
            }
            _ => Did::Nothing,
        }
    }

    /// The number pad, the manual's page 8. A field tapped brings it up over whatever
    /// screen asked for it, the number is typed in, and ENTER takes the pad away again
    /// and hands the number to the field it was typed for.
    ///
    /// * `open:<field>` - `driver`, `password`, `sale:n1`, `sale:n2`, `sale:z`,
    ///   `sale:paid`, `vol:<sound>`
    /// * `<digit>`, `ce` (the field emptied), `x` (the pad put away), `enter`
    fn pad_press(&mut self, key: &str) -> Did {
        let (verb, rest) = key.split_once(':').unwrap_or((key, ""));
        match verb {
            "open" => {
                if rest.is_empty() {
                    return Did::Nothing;
                }
                self.pad = Some(Pad { into: rest.to_string(), typed: String::new() });
                Did::Moved
            }
            "x" => match self.pad.take() {
                Some(_) => Did::Moved,
                None => Did::Nothing,
            },
            "ce" => {
                let Some(p) = self.pad.as_mut().filter(|p| !p.typed.is_empty()) else { return Did::Nothing };
                p.typed.clear();
                Did::Moved
            }
            "enter" => self.pad_enter(),
            // a digit, while the field has room for one more
            d => {
                let digit = d.len() == 1 && d.as_bytes()[0].is_ascii_digit();
                let Some(p) = self.pad.as_mut().filter(|p| digit && p.typed.len() < p.room()) else {
                    return Did::Nothing;
                };
                p.typed.push_str(d);
                Did::Moved
            }
        }
    }

    /// ENTER on the pad: the pad goes, and the number lands in the field it was typed for.
    /// ENTER with nothing typed does nothing at all - it used to put the empty field away
    /// and open the other one, and with both of them empty the pad could not be got rid of.
    fn pad_enter(&mut self) -> Did {
        let Some(p) = self.pad.clone().filter(|p| !p.typed.is_empty()) else { return Did::Nothing };
        self.pad = None;
        let n: u32 = p.typed.parse().unwrap_or(0);
        let did = match p.into.as_str() {
            // the manual's page 8 counts four steps, and the third of them is "tryk på
            // adgangskode-feltet": the console does not carry the driver on by itself
            "driver" => {
                self.driver = Some(p.typed.clone());
                Did::Moved
            }
            // the pass code is what opens the console, whether it was locked over a break
            // or is being taken on for the first time. Any number does, as it always did.
            "password" => {
                if self.driver.is_none() {
                    self.pad = Some(Pad { into: "driver".into(), typed: String::new() });
                    return Did::Moved;
                }
                self.locked = false;
                self.screen = self.home();
                Did::Moved
            }
            // the ticket's own fields. The money is typed in kroner and the console counts
            // in øre, so that is where the two meet.
            "sale:paid" => self.press(&format!("paid:{}", n.saturating_mul(100))),
            "sale:z" => self.press(&format!("zones:{n}")),
            "sale:n1" => self.press(&format!("count:0:{n}")),
            "sale:n2" => self.press(&format!("count:1:{n}")),
            other => match other.strip_prefix("vol:").and_then(sound_at) {
                Some(i) => {
                    self.sound[i].vol = n.min(100);
                    Did::Moved
                }
                None => Did::Nothing,
            },
        };
        // whatever the number meant, the pad went away, and that is a move in itself
        if did == Did::Nothing { Did::Moved } else { did }
    }

    /// The game tells the console whether a trip is being driven: Billetter has nothing to
    /// sell against until one is, and ESC goes back to the driving screen rather than to
    /// the list of lines.
    pub fn set_duty(&mut self, on: bool) {
        // the trip given back under the driver's feet: there is nothing to sell against
        // any more, so the console leaves the cash ticket rather than standing on a screen
        // whose keys have all died
        if self.on_duty && !on && matches!(self.screen, Screen::Tix | Screen::Sale) {
            self.screen = Screen::Lines;
            self.sale = None;
        }
        self.on_duty = on;
    }

    /// Where the console puts itself down: the driving screen while a trip is being driven,
    /// else the list of lines.
    fn home(&self) -> Screen {
        if self.on_duty {
            Screen::Duty
        } else {
            Screen::Lines
        }
    }

    /// A screen asked for by name. **Billetter** remembers where it was pressed from, so
    /// that ESC and a finished sale both come back to it.
    fn go(&mut self, to: Screen) -> Did {
        // there is nothing to sell against until a trip is taken on
        if to == Screen::Tix && !self.on_duty {
            return Did::Nothing;
        }
        if to == Screen::Tix && !matches!(self.screen, Screen::Tix | Screen::Sale) {
            self.tix_back = Some(self.screen);
        }
        self.screen = to;
        Did::Moved
    }

    /// Where ESC steps back to, screen by screen, as the manual's own menus nest.
    fn back(&self) -> Screen {
        match self.screen {
            Screen::Sound => Screen::Menu,
            Screen::Kind => Screen::Extra,
            Screen::Rds | Screen::Inf | Screen::Void | Screen::Extra => Screen::Serve,
            Screen::Sale => Screen::Tix,
            Screen::Tix => self.tix_back.unwrap_or_else(|| self.home()),
            Screen::Serve | Screen::Menu | Screen::Duty | Screen::Trips => self.home(),
            // from the list of lines there is nowhere further back but out
            Screen::Lines => Screen::Login,
            other => other,
        }
    }

    /// How the bar along the foot stands now (the manual's page 5).
    fn keys(&self) -> Value {
        let look = |k: &str| key_look(k, self.screen, self.on_duty);
        json!({ "menu": look("menu"), "tix": look("tix"), "linje": look("linje"), "serve": look("serve") })
    }

    /// CHECK IND EKSTRA's own keys, the manual's four steps on pages 24 to 26.
    ///
    /// * `open` - the key in Betjening, or the crowd in the strip: the console waits
    /// * `card:<who>` - a card was held to the reader; what it is fills the first slot
    /// * `step:<slot>:<-1|1>`, `who:<slot>:<who>`, `zone:<local|far>`, `reset`
    /// * `ok` - set and taken, so the card is to be held up once more
    /// * `done` - that second hold: the card is checked in and the change stands
    fn extra_press(&mut self, key: &str) -> Did {
        let (verb, rest) = key.split_once(':').unwrap_or((key, ""));
        match verb {
            "open" => {
                self.extra = Some(Extra { step: Step::Hold, ..Extra::stock(Who::Adult) });
                self.screen = Screen::Extra;
                Did::Moved
            }
            // a pensioner travelling along is checked in as an adult, which the manual's
            // page 26 says in so many words: anything that is not a child is one
            "card" => {
                let x = self.extra.as_mut().filter(|x| x.step == Step::Hold);
                let Some(x) = x else { return Did::Nothing };
                let was = if Who::of(rest) == Some(Who::Child) { Who::Child } else { Who::Adult };
                *x = Extra::stock(was);
                Did::Moved
            }
            "done" => {
                let had = self.extra.as_ref().is_some_and(|x| x.step == Step::Again);
                if !had {
                    return Did::Nothing;
                }
                self.extra = None;
                self.screen = Screen::Serve;
                Did::Moved
            }
            _ => {
                let Some(x) = self.extra.as_mut().filter(|x| x.step == Step::Form) else { return Did::Nothing };
                match verb {
                    "step" => {
                        let (slot, d) = rest.split_once(':').unwrap_or((rest, ""));
                        let (Ok(slot), Ok(d)) = (slot.parse::<usize>(), d.parse::<i32>()) else { return Did::Nothing };
                        if !x.can(slot, d) {
                            return Did::Nothing;
                        }
                        x.rows[slot].0 = x.rows[slot].0.saturating_add_signed(d);
                        Did::Moved
                    }
                    "who" => {
                        let (slot, who) = rest.split_once(':').unwrap_or((rest, ""));
                        let (Ok(slot), Some(who)) = (slot.parse::<usize>(), Who::of(who)) else { return Did::Nothing };
                        if slot >= x.rows.len() {
                            return Did::Nothing;
                        }
                        x.rows[slot].1 = who;
                        Did::Moved
                    }
                    "zone" => {
                        x.far = rest == "far";
                        Did::Moved
                    }
                    "reset" => {
                        *x = Extra::stock(x.was);
                        Did::Moved
                    }
                    "ok" => {
                        x.step = Step::Again;
                        Did::Moved
                    }
                    _ => Did::Nothing,
                }
            }
        }
    }

    /// What the screens draw. Anything the console has not decided is left out rather than
    /// sent as nothing, so a screen can tell "no ticket" from "a ticket costing nothing".
    pub fn value(&self) -> Value {
        let mut v = json!({
            "screen": self.screen.key(),
            "keys": self.keys(),
            "can_void": self.last.is_some(),
            "shut": self.shut,
            "slow": self.slow,
        });
        if let Some(s) = self.sale.as_ref() {
            v["sale"] = s.value();
        }
        if let Some(s) = self.last.as_ref() {
            v["last"] = s.value();
        }
        if let Some(x) = self.extra.as_ref() {
            v["extra"] = x.value();
        }
        if let Some(p) = self.pad.as_ref() {
            v["pad"] = p.value();
        }
        v["driver"] = self.driver.clone().map_or(Value::Null, Value::String);
        v["locked"] = Value::Bool(self.locked);
        let mut snd = serde_json::Map::new();
        for (name, s) in SOUNDS.iter().zip(self.sound.iter()) {
            snd.insert(name.to_string(), json!({ "on": s.on, "vol": s.vol }));
        }
        v["sound"] = Value::Object(snd);
        v
    }
}

static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

/// The console this session is driving, for the page and for the screen in the bus alike.
pub fn with<T>(f: impl FnOnce(&mut Console) -> T) -> T {
    let mut guard = CONSOLE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Console::default))
}

/// A key pressed on either screen.
pub fn press(key: &str) -> Did {
    with(|c| c.press(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (the fork) The manual's own worked examples, which are the only fares it prints.
    #[test]
    fn the_fares_are_the_manuals_own() {
        // page 15: one adult over two zones, 22,00
        assert_eq!(fare(Who::Adult, 2), 2200);
        // page 18: seven zones, 72,00
        assert_eq!(fare(Who::Adult, 7), 7200);
        // one zone is the shortest journey there is, and costs the same as two
        assert_eq!(fare(Who::Adult, 1), 2200);
        // a child travels at half
        assert_eq!(fare(Who::Child, 2), 1100);
        assert_eq!(fare(Who::Child, 3), 1600);
        // a bicycle costs the same however far it goes: page 18 sells one beside a
        // two-zone adult ticket for 35,00 together
        assert_eq!(fare(Who::Bike, 2), 1300);
        assert_eq!(fare(Who::Bike, 9), 1300);
        assert_eq!(fare(Who::Adult, 2) + fare(Who::Bike, 2), 3500);
        // and the half fare of an odd number of kroner is not lost to a rounding
        assert_eq!(kr(fare(Who::Child, 2)), "11,00");
        assert_eq!(kr(7200), "72,00");
        assert_eq!(kr(50), "0,50");
    }

    /// (the fork) The sales menu's quick keys: one adult or one child, the manual's page 14.
    #[test]
    fn a_zone_key_writes_the_everyday_ticket() {
        let mut c = Console::default();
        assert_eq!(c.press("buy:adult:2"), Did::Moved);
        let s = c.sale.clone().expect("a ticket");
        assert_eq!(s.rows, [(1, Who::Adult), (0, Who::Child)]);
        assert_eq!((s.zones, s.edit), (2, false));
        assert_eq!(kr(s.price()), "22,00");

        // the child row: nought adults and one child, at half the fare
        assert_eq!(c.press("buy:child:3"), Did::Moved);
        let s = c.sale.clone().expect("a ticket");
        assert_eq!(s.rows, [(0, Who::Adult), (1, Who::Child)]);
        assert_eq!(kr(s.price()), "16,00");

        // Flere... opens the ticket to be written out by hand
        assert_eq!(c.press("buy:adult:0"), Did::Moved);
        assert!(c.sale.as_ref().is_some_and(|s| s.edit && s.zones == 1));
    }

    /// (the fork) The manual's page 18, second example: one adult and one bicycle over two
    /// zones, written out by hand, 35,00 together.
    #[test]
    fn the_ticket_is_written_out_by_hand() {
        let mut c = Console::default();
        c.press("buy:adult:0");
        assert_eq!(c.press("who:1:bike"), Did::Moved);
        assert_eq!(c.press("count:1:1"), Did::Moved);
        assert_eq!(c.press("zones:2"), Did::Moved);
        assert_eq!(kr(c.sale.as_ref().unwrap().price()), "35,00");

        // the fields cannot be pressed once the ticket is taken as written
        assert_eq!(c.press("ok"), Did::Moved);
        assert!(!c.sale.as_ref().unwrap().edit);
        assert_eq!(c.press("zones:9"), Did::Nothing, "a settled ticket does not change");
        assert_eq!(c.press("who:0:child"), Did::Nothing);
    }

    /// (the fork) The white field is what the customer came with, and the field under it
    /// works the change out - the first player asked for it.
    #[test]
    fn the_console_works_the_change_out() {
        let mut c = Console::default();
        c.press("buy:adult:2");
        assert_eq!(c.press("paid:5000"), Did::Moved);
        let s = c.sale.as_ref().unwrap();
        assert_eq!((kr(s.price()), kr(s.paid()), kr(s.back())), ("22,00".into(), "50,00".into(), "28,00".into()));

        // money that does not cover the fare leaves nothing to give back
        c.press("paid:1000");
        assert_eq!(kr(c.sale.as_ref().unwrap().back()), "0,00");

        // and while he had it counted out there is nothing to give back either
        c.press("buy:adult:2");
        let s = c.sale.as_ref().unwrap();
        assert_eq!((s.paid, kr(s.paid()), kr(s.back())), (None, "22,00".into(), "0,00".into()));
    }

    /// (the fork) The manual's two presses, and KONTANTBILLET - ANNULLER after them.
    #[test]
    fn selling_keeps_the_ticket_for_annuller_salg() {
        let mut c = Console::default();
        assert_eq!(c.press("void"), Did::Nothing, "nothing sold, nothing to take back");

        c.press("buy:adult:7");
        assert_eq!(c.press("ok"), Did::Sold);
        assert!(c.sale.is_none(), "the ticket is off the screen once it is sold");
        assert_eq!(c.last.as_ref().map(|s| kr(s.price())), Some("72,00".into()));
        assert_eq!(c.value()["can_void"], true);

        // taking it back leaves nothing to take back a second time
        assert_eq!(c.press("void"), Did::Moved);
        assert_eq!(c.press("void"), Did::Nothing);
        assert_eq!(c.value()["can_void"], false);
    }

    /// (the fork) CHECK IND EKSTRA walks the manual's four steps and will not skip one.
    #[test]
    fn check_ind_ekstra_takes_the_manuals_four_steps_in_order() {
        let mut c = Console::default();
        // nothing happens before the key is pressed
        assert_eq!(c.press("extra:card:adult"), Did::Nothing);

        // 1. the key, and the console waits for the card
        assert_eq!(c.press("extra:open"), Did::Moved);
        assert_eq!(c.extra.as_ref().unwrap().step, Step::Hold);
        // the slots cannot be set while it waits
        assert_eq!(c.press("extra:step:1:1"), Did::Nothing);
        assert_eq!(c.press("extra:ok"), Did::Nothing);

        // 2. the card is read, and what it is fills the first slot
        assert_eq!(c.press("extra:card:child"), Did::Moved);
        let x = c.extra.clone().unwrap();
        assert_eq!(x.step, Step::Form);
        assert_eq!(x.rows[0], (1, Who::Child));
        assert_eq!((x.rows[1].0, x.rows[2].1), (0, Who::Bike));

        // 3. two children along, and the third slot made a dog
        assert_eq!(c.press("extra:step:1:1"), Did::Moved);
        c.press("extra:step:1:1");
        c.press("extra:who:2:dog");
        c.press("extra:step:2:1");
        assert_eq!(c.press("extra:zone:far"), Did::Moved);
        let x = c.extra.clone().unwrap();
        assert_eq!((x.rows[1].0, x.rows[2].1, x.far), (2, Who::Dog, true));

        // Nulstil puts it back to the card itself
        assert_eq!(c.press("extra:reset"), Did::Moved);
        let x = c.extra.clone().unwrap();
        assert_eq!((x.rows[0], x.rows[1].0, x.far), ((1, Who::Child), 0, false));

        // 4. OK, and the card is held up once more - that second hold is the check in
        assert_eq!(c.press("extra:ok"), Did::Moved);
        assert_eq!(c.extra.as_ref().unwrap().step, Step::Again);
        assert_eq!(c.press("extra:step:1:1"), Did::Nothing, "it is taken; nothing moves now");
        assert_eq!(c.press("extra:done"), Did::Moved);
        assert!(c.extra.is_none());
        assert_eq!(c.press("extra:done"), Did::Nothing);
    }

    /// (the fork) The manual's page 25 stops at 29 on one Rejsekort, and the card holder is
    /// always travelling - so the minus key under the first slot does nothing at 1.
    #[test]
    fn a_counter_key_that_would_run_past_its_end_is_dead() {
        let mut c = Console::default();
        c.press("extra:open");
        c.press("extra:card:adult");
        let now = |c: &Console| c.extra.clone().expect("a change being made");
        assert!(!now(&c).can(0, -1), "the card holder is always travelling");
        assert!(!now(&c).can(1, -1), "an empty slot cannot go below nought");
        assert!(now(&c).can(1, 1));
        assert_eq!(c.press("extra:step:0:-1"), Did::Nothing);

        // fill it to the manual's 29 and the plus keys all die
        for _ in 0..MOST - 1 {
            c.press("extra:step:1:1");
        }
        assert_eq!(now(&c).total(), MOST);
        let x = now(&c);
        assert!(!x.can(0, 1) && !x.can(1, 1) && !x.can(2, 1));
        assert_eq!(c.press("extra:step:2:1"), Did::Nothing);
        // but there is always room to take one off again
        assert!(now(&c).can(1, -1));
        assert_eq!(c.press("extra:step:1:-1"), Did::Moved);
        assert!(now(&c).can(2, 1));
    }

    /// (the fork) The two switches of the console stand where they are put, and the page is
    /// no longer the one remembering them.
    #[test]
    fn the_two_switches_stand_where_they_are_put() {
        let mut c = Console::default();
        assert_eq!((c.shut, c.slow), (false, false));
        assert_eq!(c.press("shut"), Did::Moved);
        assert_eq!(c.press("slow"), Did::Moved);
        assert_eq!((c.shut, c.slow), (true, true));
        assert_eq!(c.value()["shut"], true);
        c.press("shut");
        assert_eq!((c.shut, c.slow), (false, true));
    }

    /// (the fork) ESC steps back out of the console's menus the way they nest, and where
    /// it puts the console down depends on whether a trip is being driven.
    #[test]
    fn esc_steps_back_out_of_the_menus() {
        let mut c = Console::default();
        c.set_duty(true);
        for (from, to) in [
            (Screen::Sound, Screen::Menu),
            (Screen::Kind, Screen::Extra),
            (Screen::Extra, Screen::Serve),
            (Screen::Rds, Screen::Serve),
            (Screen::Inf, Screen::Serve),
            (Screen::Void, Screen::Serve),
            (Screen::Sale, Screen::Tix),
            (Screen::Serve, Screen::Duty),
            (Screen::Menu, Screen::Duty),
            (Screen::Lines, Screen::Login),
        ] {
            // (backing out to the login leaves the pad up for the next driver, and ESC
            // would take that before it took the screen)
            c.pad = None;
            c.screen = from;
            c.press("esc");
            assert_eq!(c.screen, to, "ESC from {:?}", from);
        }
        // without a trip the console puts itself down on the list of lines instead
        c.pad = None;
        c.set_duty(false);
        c.screen = Screen::Menu;
        c.press("esc");
        assert_eq!(c.screen, Screen::Lines);
    }

    /// (the fork) Billetter leads nowhere until a trip is taken on, and remembers where it
    /// was pressed from.
    #[test]
    fn billetter_remembers_where_it_was_pressed_from() {
        let mut c = Console::default();
        c.screen = Screen::Lines;
        assert_eq!(c.press("go:tix"), Did::Nothing, "nothing to sell against yet");
        assert_eq!(c.screen, Screen::Lines);
        assert_eq!(c.value()["keys"]["tix"], "off dead");

        c.set_duty(true);
        c.screen = Screen::Serve;
        assert_eq!(c.press("go:tix"), Did::Moved);
        assert_eq!(c.value()["keys"]["tix"], "now");
        // down into the ticket and back out again lands where Billetter was pressed
        c.press("go:sale");
        c.press("esc");
        assert_eq!(c.screen, Screen::Tix);
        c.press("esc");
        assert_eq!(c.screen, Screen::Serve, "back where Billetter was pressed from");

        // and the trip ending under the driver's feet takes him off the ticket: there is
        // nothing to sell against any more
        c.press("go:tix");
        c.press("buy:adult:2");
        assert_eq!(c.screen, Screen::Sale);
        c.set_duty(false);
        assert_eq!(c.screen, Screen::Lines);
        assert!(c.sale.is_none(), "the half-written ticket goes with it");
    }

    /// (the fork) The manual's page 5 gives the bar's colours their meaning: white can be
    /// pressed, grey cannot, yellow is the menu you are standing in.
    #[test]
    fn the_bar_marks_the_menu_you_are_standing_in() {
        let mut c = Console::default();
        c.set_duty(true);
        c.screen = Screen::Duty;
        let k = c.value()["keys"].clone();
        assert_eq!((&k["linje"], &k["menu"], &k["serve"], &k["tix"]), (&json!("now"), &json!("white"), &json!("white"), &json!("white")));

        // every screen under a menu puts that menu on yellow, not only the menu itself
        for s in [Screen::Menu, Screen::Sound] {
            c.screen = s;
            assert_eq!(c.value()["keys"]["menu"], "now", "{s:?}");
        }
        for s in [Screen::Serve, Screen::Rds, Screen::Inf, Screen::Void, Screen::Extra, Screen::Kind] {
            c.screen = s;
            assert_eq!(c.value()["keys"]["serve"], "now", "{s:?}");
        }
        for s in [Screen::Tix, Screen::Sale] {
            c.screen = s;
            assert_eq!(c.value()["keys"]["tix"], "now", "{s:?}");
        }
    }

    /// (the fork) A key that means nothing here sounds dead and moves nothing.
    #[test]
    fn a_key_that_means_nothing_does_nothing() {
        let mut c = Console::default();
        for key in ["ok", "zones:3", "paid:5000", "count:0:2", "who:0:adult"] {
            assert_eq!(c.press(key), Did::Nothing, "{key} without a ticket");
        }
        c.press("buy:adult:0");
        for key in ["buy:lorry:2", "zones:x", "count:9:1", "who:0:lorry", "nonsense"] {
            assert_eq!(c.press(key), Did::Nothing, "{key}");
        }
        // the most that may travel on one Rejsekort is the manual's own 29
        c.press("count:0:99");
        assert_eq!(c.sale.as_ref().unwrap().rows[0].0, MOST);
    }

    /// (the fork) The manual's page 8, step by step: the driver taps his number field, types
    /// it, ENTER takes the pad away - and it is he who taps the pass code field next, not
    /// the console that carries him there.
    #[test]
    fn the_login_follows_the_manuals_four_steps() {
        let mut c = Console::default();
        c.screen = Screen::Login;

        c.press("pad:open:driver");
        for d in ["1", "3", "5", "0", "0", "1"] {
            assert_eq!(c.press(&format!("pad:{d}")), Did::Moved);
        }
        assert_eq!(c.pad.as_ref().unwrap().shown(), "135001");
        assert_eq!(c.press("pad:enter"), Did::Moved);
        assert!(c.pad.is_none(), "ENTER puts the pad away");
        assert_eq!(c.driver.as_deref(), Some("135001"));
        assert_eq!(c.screen, Screen::Login, "and leaves him on the login to tap the next field");

        c.press("pad:open:password");
        c.press("pad:4");
        c.press("pad:2");
        assert_eq!(c.pad.as_ref().unwrap().shown(), "**", "the pass code is not shown as it is typed");
        c.press("pad:enter");
        assert!(c.pad.is_none());
        assert!(!c.locked);
        assert_eq!(c.screen, Screen::Lines, "and with no trip under way, in at the list of lines");
    }

    /// (the fork) The bug the player found: the pad would not go away. ENTER on an empty
    /// field used to open the other one, and with both empty it came straight back.
    #[test]
    fn the_pad_can_always_be_got_rid_of() {
        let mut c = Console::default();
        c.screen = Screen::Login;
        c.press("pad:open:driver");
        assert_eq!(c.press("pad:enter"), Did::Nothing, "ENTER with nothing typed is dead");
        assert!(c.pad.is_some(), "and so leaves the pad standing rather than ping-ponging");
        assert_eq!(c.value()["pad"]["enter"], false, "which the screen draws as a dead key");

        // the three ways out of it
        assert_eq!(c.press("pad:x"), Did::Moved);
        assert!(c.pad.is_none());
        c.press("pad:open:driver");
        c.press("pad:7");
        assert_eq!(c.press("pad:ce"), Did::Moved, "CE empties the field");
        assert_eq!(c.pad.as_ref().unwrap().typed, "");
        assert_eq!(c.press("pad:ce"), Did::Nothing, "and on an empty field is dead");
        c.press("pad:7");
        c.press("pad:enter");
        assert!(c.pad.is_none());

        // a pass code typed before there is any driver asks for the number first
        let mut c = Console::default();
        c.screen = Screen::Login;
        c.press("pad:open:password");
        c.press("pad:1");
        c.press("pad:enter");
        assert_eq!(c.pad.as_ref().unwrap().into, "driver");
        assert_eq!(c.screen, Screen::Login, "the console is not opened on a pass code alone");
        assert_eq!(c.driver, None);
    }

    /// (the fork) Each field takes as many digits as it holds, and no more: the driver's
    /// number eight and the pass code four, which the manual's page 8 says.
    #[test]
    fn a_field_takes_what_it_holds() {
        let mut c = Console::default();
        for (field, room) in [("driver", 8), ("password", 4), ("sale:paid", 4), ("vol:key", 3), ("sale:z", 2)] {
            c.press(&format!("pad:open:{field}"));
            for _ in 0..12 {
                c.press("pad:9");
            }
            assert_eq!(c.pad.as_ref().unwrap().typed.len(), room, "{field}");
            assert_eq!(c.press("pad:9"), Did::Nothing, "{field} is full");
            c.press("pad:x");
        }
        // and only a digit is typed at all
        c.press("pad:open:driver");
        for key in ["pad:a", "pad:12", "pad:-", "pad:"] {
            assert_eq!(c.press(key), Did::Nothing, "{key}");
        }
        assert_eq!(c.pad.as_ref().unwrap().typed, "");
    }

    /// (the fork) The pad over the ticket hands its number to the ticket: the money is typed
    /// in kroner and the console counts in øre, so the two meet at ENTER.
    #[test]
    fn the_pad_over_the_ticket_fills_it_in() {
        let mut c = Console::default();
        c.set_duty(true);
        // the manual's page 18: seven zones, and the customer comes with a hundred
        c.press("buy:adult:0");
        c.press("pad:open:sale:z");
        c.press("pad:7");
        c.press("pad:enter");
        assert_eq!(c.sale.as_ref().unwrap().zones, 7);
        c.press("pad:open:sale:n1");
        c.press("pad:2");
        c.press("pad:enter");
        assert_eq!(c.sale.as_ref().unwrap().rows[0].0, 2);

        c.press("ok");
        c.press("pad:open:sale:paid");
        c.press("pad:1");
        c.press("pad:0");
        c.press("pad:0");
        c.press("pad:enter");
        assert_eq!(c.sale.as_ref().unwrap().paid, Some(10_000), "a hundred kroner is ten thousand øre");
        assert!(c.pad.is_none());
    }

    /// (the fork) The sound menu, which the pad also types into: a volume out of a hundred,
    /// and a sound switched on or off.
    #[test]
    fn the_sound_menu_is_the_consoles_own() {
        let mut c = Console::default();
        assert_eq!(c.value()["sound"]["fault"]["vol"], 60, "the fault is heard over the bus");
        c.press("pad:open:vol:key");
        c.press("pad:9");
        c.press("pad:9");
        c.press("pad:9");
        c.press("pad:enter");
        assert_eq!(c.sound[0].vol, 100, "a hundred is as loud as it goes");
        assert_eq!(c.press("snd:key"), Did::Moved);
        assert_eq!(c.value()["sound"]["key"]["on"], false);
        assert_eq!(c.press("snd:nonsense"), Did::Nothing);

        // the screen handing back what the driver set the session before this one
        assert_eq!(c.press("snd:set:fault:0:80"), Did::Moved);
        assert_eq!(c.sound[2], Sound { on: false, vol: 80 });
        assert_eq!(c.press("snd:set:fault:1:999"), Did::Moved);
        assert_eq!(c.sound[2].vol, 100, "and no louder than it goes");
        for key in ["snd:set:nonsense:1:50", "snd:set:key:1", "snd:set:"] {
            assert_eq!(c.press(key), Did::Nothing, "{key}");
        }
    }

    /// (the fork) ESC takes the pad before it takes the screen, and on the login it hands
    /// the console to the next driver ("tast ESC for chaufførskift").
    #[test]
    fn esc_takes_the_pad_first() {
        let mut c = Console::default();
        c.set_duty(true);
        c.press("go:menu");
        c.press("go:sound");
        c.press("pad:open:vol:dead");
        c.press("esc");
        assert!(c.pad.is_none());
        assert_eq!(c.screen, Screen::Sound, "the screen stands while the pad goes");
        c.press("esc");
        assert_eq!(c.screen, Screen::Menu);

        c.driver = Some("135001".into());
        c.screen = Screen::Login;
        c.press("esc");
        assert_eq!(c.driver, None, "the number goes with the driver");
        assert_eq!(c.pad.as_ref().unwrap().into, "driver");
    }

    /// (the fork) **Afslut tur** is a key like any other now, so the screen in the bus can
    /// press it too - and whichever screen presses it, the game is the one that ends the
    /// trip. It used to be the tablet's own route, which the bus's screen cannot reach.
    #[test]
    fn afslut_tur_is_a_key_of_the_console() {
        let mut c = Console::default();
        assert_eq!(c.press("end"), Did::Nothing, "there is no trip to give back");
        c.set_duty(true);
        c.screen = Screen::Duty;
        assert_eq!(c.press("end"), Did::Ended, "which the game acts on");
        // the console stays where it is: it is the game that ends the trip, and the
        // console follows when it is told the trip is over
        assert_eq!(c.screen, Screen::Duty);
        c.set_duty(false);
        assert_eq!(c.press("end"), Did::Nothing);
    }
}
