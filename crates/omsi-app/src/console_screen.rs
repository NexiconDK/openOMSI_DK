//! (the fork) The driver console drawn on the bus's own screen.
//!
//! The fork already draws HTML pages into a vehicle's textures (`[htmltexture]`, see
//! `omsi_sim::htmlengine`), but only for a page a bus declares in its own `model.cfg`.
//! The console is to go into buses nobody has prepared, so this drives the same engine
//! by hand: the page is ours, the texture is ours, and the bus is not touched at all.
//!
//! What goes down to the page is [`crate::console::Console::value`] flattened to plain
//! strings, since the engine's JavaScript has no `JSON.parse`. What comes back up is
//! `omsi.trigger("<key>")`, which is a console key by the name [`crate::console::Console::press`]
//! knows it by - so the screen in the bus and the tablet press the very same keys.

use omsi_sim::htmlengine::EngineRenderer;
use omsi_sim::htmltex::{HtmlRenderer, PointerKind};
use serde_json::Value;

/// The page, drawn into the texture at this size. It is the shape of the console's own
/// screen in the player's model (12.8 by 12.2 cm), so nothing is stretched on the way.
pub const WIDTH: u32 = 1024;
pub const HEIGHT: u32 = 976;

const PAGE: &str = include_str!("console_screen.html");

/// Flatten `v` into the plain strings the page reads, under `name` and the names of
/// whatever it holds, joined with `_`: `con`, `con_screen`, `con_sale_price`,
/// `con_keys_menu`. A list numbers its own entries from zero (`con_rows_0_who`) and says
/// how long it is (`con_rows_n`).
///
/// The engine's JavaScript has no `JSON.parse`, so a page cannot be handed the console as
/// one lump - and it reads plain names more easily than it would pick a lump apart.
pub fn flatten(name: &str, v: &Value, out: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            for (k, val) in m {
                flatten(&format!("{name}_{k}"), val, out);
            }
        }
        Value::Array(a) => {
            out.push((format!("{name}_n"), a.len().to_string()));
            for (i, val) in a.iter().enumerate() {
                flatten(&format!("{name}_{i}"), val, out);
            }
        }
        // a thing the console has not decided is left out rather than sent as "null", so
        // the page can tell "no ticket" from "a ticket costing nothing"
        Value::Null => {}
        Value::String(s) => out.push((name.to_string(), s.clone())),
        Value::Bool(b) => out.push((name.to_string(), if *b { "1" } else { "" }.to_string())),
        other => out.push((name.to_string(), other.to_string())),
    }
}

/// Everything the page in the bus reads: the same thing the tablet is sent, under `s`
/// (so the console is `s_con_screen`, the trip `s_duty_line`, the words `s_labels_menu`).
pub fn page_vars(state: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    flatten("s", state, &mut out);
    out
}

/// The console's screen in one bus: the page, the pixels it last drew, and the keys it
/// has pressed since anybody asked.
pub struct ConsoleScreen {
    page: EngineRenderer,
    /// What was last sent down, so only what changed is sent again.
    last: Vec<(String, String)>,
    /// Whether a frame has been drawn at all yet.
    started: bool,
}

impl Default for ConsoleScreen {
    fn default() -> ConsoleScreen {
        ConsoleScreen::new()
    }
}

impl ConsoleScreen {
    pub fn new() -> ConsoleScreen {
        ConsoleScreen { page: EngineRenderer::new(WIDTH, HEIGHT, PAGE), last: Vec::new(), started: false }
    }

    /// Hand the game down to the page and the page's presses back up. The keys come
    /// back in the order they were pressed, to be given to [`crate::console::press`].
    pub fn update(&mut self, state: &Value) -> Vec<String> {
        let now = page_vars(state);
        let changed: Vec<(String, String)> = if self.started {
            now.iter().filter(|(k, v)| self.last.iter().find(|(lk, _)| lk == k).map(|(_, lv)| lv) != Some(v)).cloned().collect()
        } else {
            now.clone()
        };
        // a name that was there and is gone again is sent as nothing, so the page does not
        // go on drawing a ticket that has been sold
        let gone: Vec<(String, String)> =
            self.last.iter().filter(|(k, _)| !now.iter().any(|(nk, _)| nk == k)).map(|(k, _)| (k.clone(), String::new())).collect();
        if !self.started || !changed.is_empty() || !gone.is_empty() {
            let all: Vec<(String, String)> = changed.into_iter().chain(gone).collect();
            self.page.set_vars(&[], &all);
            self.last = now;
            self.started = true;
        }
        self.page.take_triggers()
    }

    /// The screen's pixels, when they have changed since the last time: RGBA, [`WIDTH`]
    /// by [`HEIGHT`].
    pub fn frame(&mut self) -> Option<Vec<u8>> {
        self.page.poll_frame()
    }

    /// A finger on the screen, at `u` and `v` across the texture (0 to 1, `v` from the top).
    pub fn touch(&mut self, u: f32, v: f32, kind: PointerKind) {
        self.page.pointer(u * WIDTH as f32, v * HEIGHT as f32, kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;

    /// (the fork) The console goes down to the page as plain names, since the engine's
    /// JavaScript cannot pick a lump of JSON apart.
    #[test]
    fn the_console_is_flattened_into_plain_names() {
        let mut c = Console::default();
        c.set_duty(true);
        c.press("buy:adult:2");
        let v = page_vars(&serde_json::json!({ "con": c.value() }));
        let get = |k: &str| v.iter().find(|(n, _)| n == k).map(|(_, s)| s.as_str());

        assert_eq!(get("s_con_screen"), Some("sale"));
        assert_eq!(get("s_con_keys_tix"), Some("now"));
        assert_eq!(get("s_con_sale_price"), Some("22,00"), "the manual's page 15");
        assert_eq!(get("s_con_sale_n1"), Some("1"));
        assert_eq!(get("s_con_sale_t1"), Some("adult"));

        // a list numbers its own entries and says how long it is
        c.press("extra:open");
        c.press("extra:card:adult");
        let v = page_vars(&serde_json::json!({ "con": c.value() }));
        let get = |k: &str| v.iter().find(|(n, _)| n == k).map(|(_, s)| s.as_str());
        assert_eq!(get("s_con_extra_rows_n"), Some("3"), "the manual's three slots");
        assert_eq!(get("s_con_extra_rows_0_n"), Some("1"));
        assert_eq!(get("s_con_extra_rows_2_t"), Some("bike"));
        // a switch is a word the page can test for truth, and an empty one is false
        assert_eq!(get("s_con_shut"), Some(""));
        c.press("shut");
        let v = page_vars(&serde_json::json!({ "con": c.value() }));
        assert_eq!(v.iter().find(|(n, _)| n == "s_con_shut").map(|(_, s)| s.as_str()), Some("1"));
        // and what the console has not decided is not sent at all
        assert!(!v.iter().any(|(n, _)| n == "s_con_driver"), "nobody is logged in yet");
    }

    /// (the fork) The page draws, and a key on it presses the console's own key - the same
    /// name the tablet sends, so there is one console behind both screens.
    #[test]
    fn the_screen_draws_and_its_keys_reach_the_console() {
        let mut s = ConsoleScreen::new();
        let mut c = Console::default();
        c.set_duty(true);
        c.screen = crate::console::Screen::Duty;
        let state = |c: &Console| serde_json::json!({ "con": c.value(), "clock": "09:20", "date": "14/3/07" });

        assert!(s.update(&state(&c)).is_empty(), "nothing is pressed by drawing");
        let frame = s.frame().expect("the first frame is always drawn");
        assert_eq!(frame.len(), (WIDTH * HEIGHT * 4) as usize);
        assert!(frame.chunks(4).any(|p| p[3] > 0), "the page is not blank");

        // the foot bar's Menu key, pressed where it stands
        let (u, v) = (0.20, 0.965);
        s.touch(u, v, PointerKind::Down);
        s.touch(u, v, PointerKind::Up);
        let keys = s.update(&state(&c));
        assert_eq!(keys, vec!["go:menu".to_string()], "the key the tablet would send");
        for k in &keys {
            c.press(k);
        }
        assert_eq!(c.screen, crate::console::Screen::Menu);
    }

    /// (the fork) Write the screen out as a picture, to look at while it is being drawn:
    /// `cargo test -p omsi-app console_screen -- --ignored --nocapture`. It is not part of
    /// the suite - it answers "does it look right", which no assertion can.
    #[test]
    #[ignore]
    fn draw_the_screen_to_a_picture() {
        let shots: Vec<(&str, Box<dyn Fn(&mut Console)>)> = vec![
            ("login", Box::new(|c: &mut Console| c.screen = crate::console::Screen::Login)),
            ("pad", Box::new(|c: &mut Console| {
                c.screen = crate::console::Screen::Login;
                c.press("pad:open:driver");
                for d in ["1", "3", "5", "0", "0", "1"] {
                    c.press(&format!("pad:{d}"));
                }
            })),
            ("duty", Box::new(|c: &mut Console| {
                c.set_duty(true);
                c.screen = crate::console::Screen::Duty;
                c.press("slow");
            })),
            ("menu", Box::new(|c: &mut Console| {
                c.set_duty(true);
                c.press("go:menu");
            })),
            ("lines", Box::new(|c: &mut Console| c.screen = crate::console::Screen::Lines)),
            ("trips", Box::new(|c: &mut Console| { c.press("line:Linje 1A"); })),
        ];
        let stops: Vec<Value> = ["AARHUS RUTEBILSTATION", "BANEGÅRDSPLADSEN", "PARK ALLÉ", "RÅDHUSET", "MØLLEPARKEN", "VESTERBRO TORV", "THORVALDSENSGADE", "CEREALIA", "VIBORGVEJ", "HASLE TORV", "JÆRNBROA", "SKEJBY"]
            .iter()
            .map(|n| serde_json::json!({ "name": n }))
            .collect();
        for (name, set) in shots {
            let mut c = Console::default();
            set(&mut c);
            let lines: Vec<Value> = ["1A", "2A", "4A", "11", "12", "13", "14", "15", "16", "17", "18", "100", "200"]
                .iter()
                .map(|s| serde_json::json!({ "name": format!("Linje {s}"), "sign": s }))
                .collect();
            let trips: Vec<Value> = (0..9)
                .map(|i| {
                    serde_json::json!({
                        "tour": format!("135S{:02}", i + 1), "pos": 0,
                        "dep": format!("{:02}:{:02}", 9 + i / 4, (i * 17) % 60), "arr": format!("{:02}:{:02}", 9 + i / 3, (i * 23) % 60),
                        "from": "AARHUS RUTEBILSTATION", "to": "SKEJBY SYGEHUS", "sign": "1A", "driving": i == 2,
                    })
                })
                .collect();
            let v = serde_json::json!({
                "con": c.value(), "clock": "09:20:14", "date": "14/3/07", "sign": "1A",
                "duty": { "line": "1A", "tour": "135S01", "zone": "4301", "next": 3, "stops": stops },
                "lines": lines, "trips": trips,
                "cards": [
                    { "who": "Voksen", "card": "personal", "out": false },
                    { "who": "Barn", "card": "flex", "out": true },
                    { "who": "Voksen", "card": "anon", "out": true },
                    {},
                    { "who": "Voksen", "card": "blocked", "out": false },
                ],
            });
            let mut s = ConsoleScreen::new();
            s.update(&v);
            let px = s.frame().expect("a frame");
            let img = image::RgbaImage::from_raw(WIDTH, HEIGHT, px).expect("the right size");
            let out = std::env::temp_dir().join(format!("console_screen_{name}.png"));
            img.save(&out).expect("written");
            println!("{}", out.display());
        }

        // what one frame of it costs. The page is drawn again only when something has
        // moved - the clock's second, a key - so this is what it costs about once a
        // second, on a thread that is not the one drawing the world.
        let mut s = ConsoleScreen::new();
        let mut c = Console::default();
        c.set_duty(true);
        c.screen = crate::console::Screen::Duty;
        let mut worst = std::time::Duration::ZERO;
        let mut total = std::time::Duration::ZERO;
        for i in 0..20 {
            let v = serde_json::json!({ "con": c.value(), "clock": format!("09:{i:02}"), "duty": { "line": "1A", "stops": stops } });
            let at = std::time::Instant::now();
            s.update(&v);
            s.frame().expect("the minute moved, so the page is drawn again");
            let took = at.elapsed();
            worst = worst.max(took);
            total += took;
        }
        println!("one frame: {:?} on average, {worst:?} at worst", total / 20);

        // and what it costs when the state arrives but nothing the page shows has moved,
        // which is what happens twice a second while driving: the speed and the stops
        // change, the clock does not, and the page must not be drawn again for that
        let mut idle = std::time::Duration::ZERO;
        let mut drawn = 0;
        for kmh in 0..40 {
            let v = serde_json::json!({ "con": c.value(), "clock": "09:20", "speed": kmh, "duty": { "line": "1A", "stops": stops } });
            let at = std::time::Instant::now();
            s.update(&v);
            if s.frame().is_some() {
                drawn += 1;
            }
            idle += at.elapsed();
        }
        println!("a state that shows nothing new: {:?} each, drawn again {drawn} times out of 40", idle / 40);
    }

    /// (the fork) The page is drawn again only when something has moved, so a screen that
    /// is standing still costs nothing.
    #[test]
    fn a_still_screen_is_not_drawn_again() {
        let mut s = ConsoleScreen::new();
        let v = serde_json::json!({ "con": Console::default().value() });
        s.update(&v);
        assert!(s.frame().is_some(), "the first frame");
        s.update(&v);
        assert!(s.frame().is_none(), "and nothing has moved since");
    }
}
