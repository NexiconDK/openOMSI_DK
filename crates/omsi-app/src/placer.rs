//! (the fork) Putting the Rejsekort things in the bus by hand, in the game.
//!
//! The standers and the console stand where a line in a text file says, and finding the
//! right line means guessing a number, saving, looking, and guessing again. This is the
//! same thing done the way anybody would expect: switch the placer on, look at a stander,
//! press **E**, and move it with the arrow keys until it sits right. **S** writes the
//! lines back to the file, so what was moved by hand stays moved.
//!
//! It is the file that is the truth, not this: the placer only edits the lines that are
//! already being used, and writing them out leaves every comment where it was.

use crate::readers::Spot;

/// What is being moved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum What {
    /// A Rejsekort stander, by its place in `standere.txt` (the first line is stander 1).
    Stander(usize),
    /// The driver console.
    Desk,
}

impl What {
    /// What it is called on the screen.
    pub fn name(&self) -> String {
        match self {
            What::Stander(i) => format!("Stander {}", i + 1),
            What::Desk => "Konsollen".to_string(),
        }
    }

    /// The file its line lives in.
    pub fn file(&self) -> &'static str {
        match self {
            What::Stander(_) => crate::readers::SPOTS,
            What::Desk => crate::desk::SPOT,
        }
    }
}

/// What the arrow keys do just now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Move,
    Turn,
    Size,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Move => "Flyt",
            Mode::Turn => "Drej",
            Mode::Size => "Størrelse",
        }
    }

    fn next(&self) -> Mode {
        match self {
            Mode::Move => Mode::Turn,
            Mode::Turn => Mode::Size,
            Mode::Size => Mode::Move,
        }
    }
}

/// The eight keys the placer takes, in the order [`Placer::arrow`] numbers them.
const LEFT: usize = 0;
const RIGHT: usize = 1;
const UP: usize = 2;
const DOWN: usize = 3;
const RAISE: usize = 4;
const LOWER: usize = 5;
const SLOWER: usize = 6;
const FASTER: usize = 7;

/// How far a second of a held key moves, turns and sizes a thing at speed 1. They are
/// chosen so that one unhurried press is a visible nudge rather than a jump across the
/// bus: a tenth of a metre, ten degrees, a tenth of its size.
const METRES_A_SECOND: f32 = 0.30;
const DEGREES_A_SECOND: f32 = 30.0;
const SIZE_A_SECOND: f32 = 0.30;

/// How slow and how fast the keys may be made, with `-` and `+`.
const SLOWEST: f32 = 0.1;
const FASTEST: f32 = 4.0;

/// Putting the Rejsekort things in the bus by hand.
#[derive(Default)]
pub struct Placer {
    /// What is held and where it now stands, while something is.
    held: Option<(What, Spot)>,
    mode: Mode,
    /// Which of the eight keys are down.
    arrows: [bool; 8],
    /// How fast the keys move it, 1 being the stock speed.
    speed: f32,
    /// What has been moved and not yet written out, so the player is told before he
    /// leaves it behind.
    unsaved: Vec<(What, Spot)>,
    /// The last thing said about saving, to put on the screen.
    said: Option<String>,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Move
    }
}

impl Placer {
    /// What is being moved just now, and where it stands.
    pub fn held(&self) -> Option<(What, &Spot)> {
        self.held.as_ref().map(|(w, s)| (*w, s))
    }

    /// Take hold of a thing, or let go of the one held. Looking at nothing and pressing
    /// lets go, which is what a person means by it.
    pub fn take(&mut self, what: Option<(What, Spot)>) {
        self.said = None;
        self.held = match (self.held.as_ref(), what) {
            // pressed on the thing already held: let it go
            (Some((had, _)), Some((w, _))) if *had == w => None,
            (_, w) => w,
        };
        self.arrows = [false; 8];
        if self.speed <= 0.0 {
            self.speed = 1.0;
        }
    }

    /// Let go of whatever is held, without changing it.
    pub fn let_go(&mut self) {
        self.held = None;
        self.arrows = [false; 8];
    }

    /// Switch what the arrows do.
    pub fn next_mode(&mut self) {
        self.mode = self.mode.next();
        self.said = None;
    }

    /// An arrow or Page key while something is held: the press is taken (true), and a key
    /// let go is not, so that a key the game saw go down comes up there too.
    pub fn arrow(&mut self, code: winit::keyboard::KeyCode, pressed: bool) -> bool {
        use winit::keyboard::KeyCode as K;
        let i = match code {
            K::ArrowLeft => LEFT,
            K::ArrowRight => RIGHT,
            K::ArrowUp => UP,
            K::ArrowDown => DOWN,
            K::PageUp => RAISE,
            K::PageDown => LOWER,
            K::Minus | K::NumpadSubtract => SLOWER,
            K::Equal | K::NumpadAdd => FASTER,
            _ => return false,
        };
        if self.held.is_none() {
            self.arrows = [false; 8];
            return false;
        }
        self.arrows[i] = pressed;
        pressed
    }

    /// Move what is held by however long the keys have been down. Returns true when
    /// something moved, so the game knows to show it in its new place.
    pub fn step(&mut self, dt: f32) -> bool {
        let Some((_, spot)) = self.held.as_mut() else { return false };
        let a = self.arrows;
        // the speed keys work whether or not anything else is held down
        if a[FASTER] != a[SLOWER] {
            let by = if a[FASTER] { 1.0 + dt } else { 1.0 / (1.0 + dt) };
            self.speed = (self.speed * by).clamp(SLOWEST, FASTEST);
        }
        let d = dt * self.speed;
        let pair = |less: usize, more: usize| (a[more] as i32 - a[less] as i32) as f32 * d;
        let (x, y, z) = (pair(LEFT, RIGHT), pair(DOWN, UP), pair(LOWER, RAISE));
        if x == 0.0 && y == 0.0 && z == 0.0 {
            return false;
        }
        match self.mode {
            Mode::Move => {
                spot.at.x += x * METRES_A_SECOND;
                spot.at.y += y * METRES_A_SECOND;
                spot.at.z += z * METRES_A_SECOND;
            }
            Mode::Turn => {
                spot.turn = wrap(spot.turn + x * DEGREES_A_SECOND);
                spot.tip = wrap(spot.tip + y * DEGREES_A_SECOND);
                spot.lean = wrap(spot.lean + z * DEGREES_A_SECOND);
            }
            // (every arrow sizes it, since there is only the one number to set)
            Mode::Size => spot.scale = (spot.scale * (1.0 + (x + y + z) * SIZE_A_SECOND)).clamp(0.01, 100.0),
        }
        self.said = None;
        true
    }

    /// The thing held has been moved, so its line wants writing back. The game calls this
    /// as it puts the thing in its new place.
    pub fn remember(&mut self) {
        let Some((what, spot)) = self.held.as_ref() else { return };
        match self.unsaved.iter_mut().find(|(w, _)| w == what) {
            Some((_, s)) => *s = spot.clone(),
            None => self.unsaved.push((*what, spot.clone())),
        }
    }

    /// Whether anything has been moved and not yet written out.
    pub fn unsaved(&self) -> bool {
        !self.unsaved.is_empty()
    }

    /// Write every line that has been moved back to its file, leaving the comments where
    /// they were. `read` hands back a file's text and `write` puts it back, so the game
    /// says where the files are and this says what goes in them.
    pub fn save(
        &mut self,
        mut read: impl FnMut(&'static str) -> Option<String>,
        mut write: impl FnMut(&'static str, &str) -> bool,
    ) -> String {
        if self.unsaved.is_empty() {
            self.said = Some("Intet at gemme".into());
            return self.said.clone().unwrap_or_default();
        }
        let mut done = 0;
        let mut failed = Vec::new();
        for file in ["standere.txt", "konsol.txt"] {
            let mine: Vec<&(What, Spot)> = self.unsaved.iter().filter(|(w, _)| w.file() == file).collect();
            if mine.is_empty() {
                continue;
            }
            let Some(text) = read(file) else {
                failed.push(file);
                continue;
            };
            let mut out = text;
            for (what, spot) in &mine {
                let n = match what {
                    What::Stander(i) => *i,
                    What::Desk => 0,
                };
                out = put_line(&out, n, spot);
            }
            if write(file, &out) {
                done += mine.len();
            } else {
                failed.push(file);
            }
        }
        self.unsaved.retain(|(w, _)| failed.contains(&w.file()));
        self.said = Some(match failed.as_slice() {
            [] => format!("{done} linje(r) gemt"),
            bad => format!("Kunne ikke skrive {}", bad.join(" og ")),
        });
        self.said.clone().unwrap_or_default()
    }

    /// What the HUD shows while the placer is on.
    pub fn help_lines(&self, under: Option<What>) -> Vec<String> {
        let mut out = vec!["Rejsekort-placering (slå fra i menuen, eller Esc)".to_string()];
        match self.held.as_ref() {
            None => {
                let at = under.map(|w| w.name()).unwrap_or_else(|| "ingenting".into());
                out.push(format!("Kig på en stander eller konsollen og tryk E for at tage fat. Under markøren: {at}"));
            }
            Some((what, spot)) => {
                out.push(format!(
                    "{}: x {:.2}  y {:.2}  z {:.2}  drejning {:.0}°  hældning {:.0}°  kip {:.0}°  skala {:.2}",
                    what.name(),
                    spot.at.x,
                    spot.at.y,
                    spot.at.z,
                    spot.turn,
                    spot.tip,
                    spot.lean,
                    spot.scale
                ));
                out.push(match self.mode {
                    Mode::Move => "Flyt - piletaster: frem/tilbage og til siden, PgUp/PgDn: op og ned".into(),
                    Mode::Turn => "Drej - venstre/højre: drejning, op/ned: hældning, PgUp/PgDn: kip".into(),
                    Mode::Size => "Størrelse - piletaster: større og mindre".into(),
                });
                out.push(format!("Tab: {} - -/+: hastighed ({:.1}x) - E: slip - S: gem i {}", self.mode.next().name(), self.speed, what.file()));
            }
        }
        if let Some(said) = self.said.as_ref() {
            out.push(said.clone());
        } else if self.unsaved() {
            out.push("Ikke gemt endnu - tryk S".into());
        }
        out
    }
}

/// An angle brought back into -180 to 180, so it never grows out of what the file can
/// sensibly hold.
fn wrap(deg: f32) -> f32 {
    let d = deg % 360.0;
    if d > 180.0 {
        d - 360.0
    } else if d < -180.0 {
        d + 360.0
    } else {
        d
    }
}

/// `spot` written as a line of a placement file, with every column, so nothing that was
/// set by hand is lost by being left out.
pub fn line_of(spot: &Spot) -> String {
    format!(
        "{}  {:.2}  {:.2}  {:.2}  {:.0}  {:.2}  {:.0}  {:.0}",
        spot.bus.as_deref().unwrap_or("*"),
        spot.at.x,
        spot.at.y,
        spot.at.z,
        spot.turn,
        spot.scale,
        spot.tip,
        spot.lean
    )
}

/// `text` with its `n`th placement line replaced by `spot`, counting only the lines that
/// place something. Every comment, blank line and `mappe` line stays exactly where it
/// was: the file is the player's, and the placer is a guest in it.
pub fn put_line(text: &str, n: usize, spot: &Spot) -> String {
    let mut seen = 0;
    let mut out: Vec<String> = Vec::new();
    let mut done = false;
    for line in text.lines() {
        let bare = line.split('#').next().unwrap_or("").trim();
        // a line that places something is one the reader would take as a place
        let places = !bare.is_empty() && crate::readers::Spot::read(bare).len() == 1;
        if places && seen == n && !done {
            // (what was written after a # on that line is kept, since it is the player's
            // own note about which stander it is)
            let note = line.split_once('#').map(|(_, c)| format!("   #{c}")).unwrap_or_default();
            out.push(format!("{}{note}", line_of(spot)));
            done = true;
        } else {
            out.push(line.to_string());
        }
        if places {
            seen += 1;
        }
    }
    // a line that was not there yet is put at the end, so a console placed by hand before
    // anything was written down is still written down
    if !done {
        out.push(line_of(spot));
    }
    let mut s = out.join("\n");
    if text.ends_with('\n') || !done {
        s.push('\n');
    }
    s
}

/// (the fork) The placer's side of the game: what the player is looking at, the keys it
/// takes, and moving what is held as the frames go by.
impl crate::app::App {
    /// Which of the Rejsekort things the player is looking at, if any. The console is
    /// taken hold of by its screen, which is the face anybody would look at.
    pub(crate) fn rk_under(&self) -> Option<(What, Spot)> {
        let p = self.player.as_ref()?;
        let (o, d, _) = self.cockpit_cursor_ray(self.camera.as_ref()?, self.gfx.surface.as_ref().map(|s| (s.config.width, s.config.height))?);
        let view = &self.gfx.sim_view.people;
        if let Some(i) = view.readers.as_ref().and_then(|r| r.hit(&p.vehicle, o, d)) {
            let spot = view.readers.as_ref().and_then(|r| r.spot(i))?.clone();
            return Some((What::Stander(i), spot));
        }
        let desk = view.desk.as_ref().filter(|k| k.under(&p.vehicle, o, d))?;
        Some((What::Desk, desk.spot().clone()))
    }

    /// E takes hold of what is looked at and lets go of it again, Tab switches what the
    /// arrows do, S writes the lines back and Esc lets go. True when the key was the
    /// placer's, so the game does not also act on it.
    pub(crate) fn rk_placer_key(&mut self, code: winit::keyboard::KeyCode, _shift: bool) -> bool {
        use winit::keyboard::KeyCode as K;
        match code {
            K::KeyE => {
                let what = self.rk_under();
                let had = self.rk_placer.held().map(|(w, _)| w);
                self.rk_placer.take(what);
                let msg = match (self.rk_placer.held().map(|(w, _)| w), had) {
                    (Some(w), _) => format!("{} taget - flyt den med piletasterne", w.name()),
                    (None, Some(w)) => format!("{} sluppet", w.name()),
                    (None, None) => "Kig på en stander eller konsollen".to_string(),
                };
                self.service_msg = Some((msg, 4.0));
                true
            }
            K::Tab if self.rk_placer.held().is_some() => {
                self.rk_placer.next_mode();
                true
            }
            K::KeyS if self.rk_placer.held().is_some() || self.rk_placer.unsaved() => {
                let msg = self.rk_save();
                self.service_msg = Some((msg, 5.0));
                true
            }
            K::Escape if self.rk_placer.held().is_some() => {
                self.rk_placer.let_go();
                true
            }
            _ => false,
        }
    }

    /// Write what has been moved back to the files beside the models.
    fn rk_save(&mut self) -> String {
        let Some(root) = self.world.as_ref().map(|w| w.root.clone()) else {
            return "Ingen OMSI-mappe".into();
        };
        let dir = crate::readers::Readers::folder(&root);
        self.rk_placer.save(
            |file| std::fs::read_to_string(dir.join(file)).ok(),
            |file, text| match std::fs::write(dir.join(file), text) {
                Ok(()) => true,
                Err(e) => {
                    log::warn!("{}: {e}", dir.join(file).display());
                    false
                }
            },
        )
    }

    /// Move what is held by however long the arrow keys have been down, and put the thing
    /// where it now stands. Called every frame while the placer is on.
    pub(crate) fn rk_placer_step(&mut self, dt: f32) {
        if !self.settings.rk_placer || !self.rk_placer.step(dt) {
            return;
        }
        self.rk_placer.remember();
        let Some((what, spot)) = self.rk_placer.held().map(|(w, s)| (w, s.clone())) else { return };
        let view = &mut self.gfx.sim_view.people;
        match what {
            What::Stander(i) => {
                if let Some(r) = view.readers.as_mut() {
                    r.set(i, spot);
                }
            }
            What::Desk => {
                if let Some(k) = view.desk.as_mut() {
                    k.set(spot);
                }
            }
        }
    }

    /// What the HUD says while the placer is on.
    pub(crate) fn rk_placer_lines(&self) -> Vec<String> {
        if !self.settings.rk_placer {
            return Vec::new();
        }
        self.rk_placer.help_lines(self.rk_under().map(|(w, _)| w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn spot() -> Spot {
        Spot { bus: None, at: Vec3::new(-1.0, 5.2, 1.05), turn: 0.0, scale: 0.45, tip: 0.0, lean: 0.0 }
    }

    /// (the fork) Taking hold of a thing and letting go of it again, which is the same key.
    #[test]
    fn e_takes_hold_and_lets_go() {
        let mut p = Placer::default();
        assert!(p.held().is_none());

        p.take(Some((What::Stander(0), spot())));
        assert_eq!(p.held().map(|(w, _)| w), Some(What::Stander(0)));

        // the same thing again lets it go
        p.take(Some((What::Stander(0), spot())));
        assert!(p.held().is_none(), "pressing on what is already held lets it go");

        // another thing takes that one instead
        p.take(Some((What::Stander(0), spot())));
        p.take(Some((What::Desk, spot())));
        assert_eq!(p.held().map(|(w, _)| w), Some(What::Desk));

        // and looking at nothing lets go
        p.take(None);
        assert!(p.held().is_none());
    }

    /// (the fork) The arrows move what is held, and nothing at all when nothing is - a key
    /// the placer does not take goes on to the game, as it always did.
    #[test]
    fn the_arrows_move_what_is_held() {
        use winit::keyboard::KeyCode as K;
        let mut p = Placer::default();
        assert!(!p.arrow(K::ArrowLeft, true), "nothing is held, so the game keeps the key");

        p.take(Some((What::Stander(0), spot())));
        assert!(p.arrow(K::ArrowRight, true), "the placer takes it");
        assert!(!p.arrow(K::ArrowRight, false), "and lets the game see it come up");
        assert!(!p.arrow(K::KeyR, true), "a key it has no use for is not taken");

        // a second of the right arrow moves it a step to the right
        p.arrow(K::ArrowRight, true);
        assert!(p.step(1.0));
        let (_, s) = p.held().unwrap();
        assert!((s.at.x - (-1.0 + METRES_A_SECOND)).abs() < 1e-4, "{}", s.at.x);
        assert_eq!((s.at.y, s.at.z), (5.2, 1.05), "and nothing else moved");

        // with no key down nothing moves
        p.arrow(K::ArrowRight, false);
        assert!(!p.step(1.0));
    }

    /// (the fork) Tab switches what the arrows do, so there are three keys to learn rather
    /// than a modifier for every axis.
    #[test]
    fn tab_switches_what_the_arrows_do() {
        use winit::keyboard::KeyCode as K;
        let mut p = Placer::default();
        p.take(Some((What::Stander(0), spot())));

        p.next_mode();
        p.arrow(K::ArrowLeft, true);
        p.step(1.0);
        let (_, s) = p.held().unwrap();
        assert!((s.turn - -DEGREES_A_SECOND).abs() < 1e-4, "turn {}", s.turn);
        assert_eq!(s.at.x, -1.0, "and it did not move");

        p.next_mode();
        p.arrow(K::ArrowLeft, false);
        p.arrow(K::ArrowUp, true);
        p.step(1.0);
        let (_, s) = p.held().unwrap();
        assert!(s.scale > 0.45, "it grew: {}", s.scale);
    }

    /// (the fork) A turn never grows out of what the file sensibly holds.
    #[test]
    fn a_turn_comes_back_round() {
        assert_eq!(wrap(0.0), 0.0);
        assert_eq!(wrap(190.0), -170.0);
        assert_eq!(wrap(-190.0), 170.0);
        assert_eq!(wrap(540.0), 180.0);
    }

    /// (the fork) Writing a line back leaves the file the player's own: every comment, the
    /// blank lines and the `mappe` line stay where they were.
    #[test]
    fn saving_leaves_the_file_as_the_player_wrote_it() {
        let text = "\
# mine standere
mappe  Sceneryobjects\\UrbanBacon\\Models\\Rejsekort

*  -1.00  5.20  1.05  0  0.45  0  0   # salgskortlæseren
*  -1.00  1.60  1.05  180
";
        let moved = Spot { at: Vec3::new(-0.90, 5.40, 1.10), ..spot() };
        let out = put_line(text, 0, &moved);

        assert!(out.contains("# mine standere"), "the comment at the top stays");
        assert!(out.contains("mappe  Sceneryobjects"), "and the folder line");
        assert!(out.contains("# salgskortlæseren"), "and the player's own note on the line");
        assert!(out.contains("-0.90  5.40  1.10"), "the line was written back:\n{out}");
        assert!(out.contains("*  -1.00  1.60  1.05  180"), "the other stander is untouched:\n{out}");
        assert_eq!(out.lines().count(), text.lines().count(), "no line gained or lost");

        // the second line is the second stander
        let out2 = put_line(text, 1, &moved);
        assert!(out2.contains("*  -1.00  5.20  1.05  0  0.45  0  0   # salgskortlæseren"), "the first is untouched:\n{out2}");
        assert!(out2.contains("-0.90  5.40  1.10"), "{out2}");

        // a file with no line for it yet gets one
        let fresh = put_line("# intet endnu\n", 0, &moved);
        assert!(fresh.contains("# intet endnu"));
        assert!(fresh.contains("-0.90  5.40  1.10"), "{fresh}");
        assert_eq!(crate::readers::Spot::read(&fresh).len(), 1);
    }

    /// (the fork) A line written back is a line the reader reads the same way again - what
    /// was placed by hand must come back exactly where it was left.
    #[test]
    fn what_is_written_is_read_back_the_same() {
        let s = Spot { bus: Some("MB_O530.bus".into()), at: Vec3::new(-0.93, 5.41, 1.07), turn: -170.0, scale: 0.45, tip: 15.0, lean: -8.0 };
        let read = crate::readers::Spot::read(&line_of(&s));
        assert_eq!(read.len(), 1, "{}", line_of(&s));
        let back = &read[0];
        assert_eq!(back.bus.as_deref(), Some("MB_O530.bus"));
        assert!((back.at - s.at).length() < 0.005, "{:?} vs {:?}", back.at, s.at);
        assert_eq!((back.turn, back.tip, back.lean), (-170.0, 15.0, -8.0));
        assert_eq!(back.scale, 0.45);
    }

    /// (the fork) Saving writes each thing to its own file, and says so.
    #[test]
    fn saving_writes_each_thing_to_its_own_file() {
        let mut p = Placer::default();
        assert_eq!(p.save(|_| None, |_, _| true), "Intet at gemme");

        p.take(Some((What::Stander(1), spot())));
        p.step(1.0);
        p.remember();
        p.take(Some((What::Desk, spot())));
        p.remember();
        assert!(p.unsaved());

        let mut wrote: Vec<(String, String)> = Vec::new();
        let said = p.save(|_| Some("*  0 0 0  0\n*  0 0 0  0\n".into()), |f, t| {
            wrote.push((f.to_string(), t.to_string()));
            true
        });
        assert!(said.contains("2"), "{said}");
        assert!(!p.unsaved(), "nothing is left over once it is written");
        let files: Vec<&str> = wrote.iter().map(|(f, _)| f.as_str()).collect();
        assert_eq!(files, ["standere.txt", "konsol.txt"]);

        // a file that will not take it keeps its change, so it is not quietly lost
        // (the console is still held after saving, as it should be, so it is moved again
        // rather than taken hold of afresh - that would let go of it)
        p.step(1.0);
        p.remember();
        let said = p.save(|_| Some(String::new()), |_, _| false);
        assert!(said.contains("Kunne ikke skrive"), "{said}");
        assert!(p.unsaved(), "and it is still waiting to be written");
    }
}
