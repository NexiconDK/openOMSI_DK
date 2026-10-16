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

/// What the console has decided so far. One of these stands for the whole session, whether
/// it is being drawn in a tablet or on the console's own screen in the bus.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Console {
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
                Did::Sold
            }
            // KONTANTBILLET - ANNULLER, the manual's pages 19 and 20
            "void" => {
                if self.last.take().is_none() {
                    return Did::Nothing;
                }
                Did::Moved
            }
            // CHECK IND EKSTRA, the manual's pages 24 to 26
            "extra" => self.extra_press(rest),
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
        let mut v = json!({ "can_void": self.last.is_some(), "shut": self.shut, "slow": self.slow });
        if let Some(s) = self.sale.as_ref() {
            v["sale"] = s.value();
        }
        if let Some(s) = self.last.as_ref() {
            v["last"] = s.value();
        }
        if let Some(x) = self.extra.as_ref() {
            v["extra"] = x.value();
        }
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
}
