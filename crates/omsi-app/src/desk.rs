//! (the fork) The driver console itself in the bus: the desk, its screen and the keys
//! beside it, hung where the player put them and showing the console that
//! [`crate::console`] decides and [`crate::console_screen`] draws.
//!
//! This is the plug-and-play side of it. A bus the fork did not make knows nothing about
//! a Rejsekort console, and its `model.cfg` is not ours to edit, so nothing here touches
//! the bus: the desk is the player's own model, hung in the bus's own frame from a line
//! in a plain file, exactly as the Rejsekort standers are. It works in any bus, whoever
//! built it, and in one whose IBIS is a plain `[texttexture]` line that could not be
//! replaced at all.
//!
//! ```text
//! <OMSI folder>/Rejsekort/Konsol/*.o3d     the desk's parts, all of them hung together
//! <OMSI folder>/Rejsekort/Konsol/*.png     and their textures, beside them
//! <OMSI folder>/Rejsekort/konsol.txt       where the desk stands (see `crate::readers::Spot`)
//! ```
//!
//! The one material whose texture is named [`SCREEN_TEX`] is the screen: it is given the
//! page's own pixels instead of the file's, and a ray that lands on it is a finger on the
//! page at that very spot.

use crate::console_screen::{ConsoleScreen, HEIGHT, WIDTH};
use crate::readers::Spot;
use crate::scene::World;
use glam::{DVec3, Mat4, Vec2, Vec3};
use omsi_geometry::{mesh_from_o3d, MeshData};
use omsi_render::{AlphaMode, MaterialId, Renderer, Scene, TextureId};
use omsi_sim::htmltex::PointerKind;
use omsi_sim::VehicleInstance;
use std::path::{Path, PathBuf};

/// The folder in `Rejsekort` the player drops the console's own model in, when he has not
/// said where it is. A console he already has as an OMSI scenery object stays where it
/// is and is named with a `mappe` line instead (see [`model_dirs`]).
pub const FOLDER: &str = "Konsol";
/// The file that says where the desk stands.
pub const SPOT: &str = "konsol.txt";
/// The texture name that makes a face the console's screen. It is the name the player's
/// own model already uses, so nothing has to be renamed to make the screen live.
pub const SCREEN_TEX: &str = "skærmtexture";

/// What to write in `konsol.txt` when there is none yet.
pub const BLANK: &str = "\
# Rejsekort-chaufførkonsollen i bussen. Én linje, på samme form som standere.txt:\n\
#\n\
#   <bus>  <x> <y> <z>  <drejning>  [skala] [hældning] [kip]\n\
#\n\
# <bus> er bussens .bus-filnavn (uden mappe), eller * for alle busser.\n\
# x er til højre, y er fremad og z er opad, i meter fra bussens nulpunkt.\n\
#\n\
# De tre sidste tal er valgfri. Står de der ikke, er skalaen 1 og konsollen\n\
# står lodret.\n\
#\n\
# Drejning:  grader om lodret - hvilken vej den vender.\n\
#            0 vender konsollen mod bussens højre side.\n\
# Skala:     1 er modellen i sin egen størrelse. Bordet er 40 x 40 cm og\n\
#            skærmen 12,8 x 12,2 cm; passer det ikke i din bus, så skru på tallet.\n\
# Hældning:  grader om tværaksen - plus vipper toppen fremad i bussen.\n\
# Kip:       grader om længdeaksen - plus hælder toppen mod højre.\n\
#\n\
# Der drejes og skaleres om modellens eget nulpunkt, så ret x, y og z til bagefter.\n\
#\n\
# MODELLEN\n\
# Har du konsollen liggende som et OMSI-sceneryobjekt i forvejen, så lad den\n\
# blive, hvor den er, og peg på mappen i stedet for at kopiere filerne ud:\n\
#\n\
#   mappe  Sceneryobjects\\UrbanBacon\\Models\\Rejsekort\n\
#\n\
# Stien er fra din OMSI-mappe. Alle .o3d-filer i mappen — og i en Model-\n\
# undermappe, som OMSI plejer at lægge dem i — bliver hængt op sammen, og\n\
# teksturerne hentes i mappen selv og i Texture/ eller Textures/ ved siden af.\n\
#\n\
# Skriver du ingen mappe, ledes der i undermappen Konsol/ her.\n\
#\n\
# Den flade, hvis tekstur hedder Skærmtexture.png, bliver selve skærmen.\n\
# Vil du ikke have en del med — printeren, kablerne — så lad bare dens\n\
# .o3d-fil være; der hænges kun det op, der ligger der.\n\
\n\
# mappe  Sceneryobjects\\UrbanBacon\\Models\\Rejsekort\n\
# *   -0.95   5.95   0.95   180   1.0\n\
";

/// Whether a console screen is standing in a bus, so the game keeps its state up to date
/// for it even when the tablet is switched off.
static WANTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the console's screen is being drawn in a bus just now.
pub fn wanted() -> bool {
    WANTED.load(std::sync::atomic::Ordering::Relaxed)
}

/// What came of the last try at hanging the console, and when that was. A console that
/// does not turn up in the bus looks exactly like one standing somewhere out of sight, so
/// the game says which of the two it is rather than leaving the player to guess.
static NOTE: std::sync::Mutex<Option<Note>> = std::sync::Mutex::new(None);

struct Note {
    what: String,
    at: std::time::Instant,
    /// Whether it goes away by itself. A note that it worked does; a note that it did
    /// not stays, since it will not mend itself.
    fades: bool,
}

/// How long a note that it worked stays on the screen: long enough to read on the way
/// into the bus, short enough not to sit there for the rest of the drive.
const SAY_FOR: std::time::Duration = std::time::Duration::from_secs(15);

/// Say nothing at all: the player has not asked for a console, so a missing one is not
/// worth a word.
fn say_nothing() {
    *NOTE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

fn say(what: String, fades: bool) {
    if fades {
        log::info!("rejsekort: {what}");
    } else {
        log::warn!("rejsekort: {what}");
    }
    *NOTE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Note { what, at: std::time::Instant::now(), fades });
}

/// The HUD's line about the console, while there is something to say.
pub fn hud_line() -> Option<String> {
    let guard = NOTE.lock().unwrap_or_else(|e| e.into_inner());
    let n = guard.as_ref()?;
    (!n.fades || n.at.elapsed() < SAY_FOR).then(|| format!("Rejsekort: {}", n.what))
}

/// The console desk of one bus, hung in it.
pub struct Desk {
    /// The bus and the folder it was made for.
    pub made_for: (PathBuf, PathBuf),
    /// Every part of the desk. They all hang on the one line, which is kept rather than
    /// the matrix it makes, so the placer can move the console about while the game runs.
    parts: Vec<usize>,
    /// The line that puts the console in the bus.
    spot: Spot,
    /// The screen: the face the page is drawn on, its texture, and the mesh the ray is
    /// cast against to find where a finger landed on it.
    screen: Option<Screen>,
    /// The page itself, while the desk is in a bus.
    page: ConsoleScreen,
    /// Whether the pointer is down, so a press and its release make one tap.
    held: bool,
    /// When `konsol.txt` was last written, and when that was last looked at. The player
    /// puts the console in its place by trying a line and looking, so the file is read
    /// again when he saves it rather than making him start the game over.
    wrote: Option<std::time::SystemTime>,
    looked: std::time::Instant,
}

/// How often `konsol.txt` is looked at for a change. Often enough that saving it and
/// looking up is one movement; rarely enough that it is not a question asked of the disk
/// sixty times a second.
const LOOK_EVERY: std::time::Duration = std::time::Duration::from_millis(500);

/// When the file was last written, or `None` when there is none.
fn written(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// The folder `konsol.txt` names its model in, if it names one (see
/// [`crate::readers::named_folder`]).
pub fn named_folder(text: &str) -> Option<&str> {
    crate::readers::named_folder(text)
}

/// Where the console's own pieces are looked for, in order.
pub fn model_dirs(root: &Path, text: &str) -> Vec<PathBuf> {
    crate::readers::model_dirs(root, text, &crate::readers::Readers::folder(root).join(FOLDER))
}

struct Screen {
    tex: TextureId,
    mesh: MeshData,
}

impl Desk {
    /// The folder the player's console model is in.
    pub fn folder(root: &Path) -> PathBuf {
        crate::readers::Readers::folder(root).join(FOLDER)
    }

    /// The file that says where the desk stands.
    pub fn spot_file(root: &Path) -> PathBuf {
        crate::readers::Readers::folder(root).join(SPOT)
    }

    /// Whether `konsol.txt` has been saved again since the desk was hung, so it is to be
    /// hung afresh. The disk is asked at most twice a second.
    pub fn moved(&mut self, root: &Path) -> bool {
        if self.looked.elapsed() < LOOK_EVERY {
            return false;
        }
        self.looked = std::time::Instant::now();
        let now = written(&Self::spot_file(root));
        if now == self.wrote {
            return false;
        }
        self.wrote = now;
        log::info!("{}: saved again, hanging the console afresh", SPOT);
        true
    }

    /// The console desk of this bus, or none when the player has put none there. A
    /// missing `konsol.txt` is written out blank, so there is something to fill in.
    pub fn new(world: &World, renderer: &Renderer, scene: &mut Scene, bus: &VehicleInstance) -> Desk {
        let dir = Self::folder(&world.root);
        let made_for = (bus.ty.def.path.clone(), dir.clone());
        let spots = Self::spot_file(&world.root);
        let mut desk = Desk {
            made_for,
            parts: Vec::new(),
            screen: None,
            // (nowhere yet; the line out of the file takes its place below)
            spot: Spot { bus: None, at: Vec3::ZERO, turn: 0.0, scale: 1.0, tip: 0.0, lean: 0.0 },
            page: ConsoleScreen::new(),
            held: false,
            wrote: written(&spots),
            looked: std::time::Instant::now(),
        };
        // nothing is standing in the bus until the screen is hung, whichever way out of
        // this we take: a bus with no console must not keep the game writing state for one
        WANTED.store(false, std::sync::atomic::Ordering::Relaxed);

        if !spots.is_file() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&spots, BLANK);
            log::info!("{}: written blank, waiting for the console's place", spots.display());
        }
        let text = std::fs::read_to_string(&spots).unwrap_or_default();
        let name = bus.ty.def.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let places = Spot::read(&text);
        // nothing written in the file yet: the player has not asked for a console, so a
        // missing one is not worth a word on his screen
        if places.is_empty() && named_folder(&text).is_none() {
            say_nothing();
            return desk;
        }
        let Some(spot) = places.iter().find(|s| s.fits(&name)) else {
            say(format!("{SPOT} har ingen linje til {name}"), false);
            return desk;
        };
        desk.spot = spot.clone();

        // where the model lies: the folder the file names, or the stock one beside it
        let dirs = model_dirs(&world.root, &text);
        let mut files: Vec<PathBuf> = dirs
            .iter()
            .flat_map(|d| std::fs::read_dir(d).into_iter().flatten().flatten())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("o3d")))
            .collect();
        // (a folder lists its files in whatever order it likes; the same order every time
        // keeps the log and any report of it steady)
        files.sort();
        if files.is_empty() {
            say(format!("ingen .o3d-filer i {}", dirs[0].display()), false);
            return desk;
        }
        let looks_in: Vec<&Path> = dirs.iter().map(PathBuf::as_path).collect();
        // pictures the model names that the folders do not hold
        let mut missing: Vec<String> = Vec::new();

        for file in &files {
            let Ok(mesh) = omsi_o3d::load_mesh(file).map_err(|e| log::warn!("{}: {e}", file.display())) else {
                continue;
            };
            let data = mesh_from_o3d(&mesh);
            let id = renderer.add_mesh(scene, &data);
            let mut mats: Vec<MaterialId> = Vec::new();
            let mut is_screen = false;
            for m in &mesh.materials {
                let screen = m.texture.to_lowercase().contains(SCREEN_TEX);
                let tex = if screen {
                    // the page's own pixels, blank until the first frame is drawn. A
                    // second face wearing the same texture shows the same page rather
                    // than a black one of its own.
                    let t = match desk.screen.as_ref() {
                        Some(s) => s.tex,
                        None => {
                            let blank =
                                omsi_texture::Image { width: WIDTH, height: HEIGHT, rgba: vec![0; (WIDTH * HEIGHT * 4) as usize], has_alpha: false };
                            let t = renderer.add_texture(scene, &blank, false);
                            desk.screen = Some(Screen { tex: t, mesh: data.clone() });
                            is_screen = true;
                            t
                        }
                    };
                    Some(t)
                } else {
                    let found = omsi_texture::find_texture(&m.texture, &looks_in)
                        .and_then(|p| world.textures.get_gpu_fast(&p))
                        .map(|(img, _)| renderer.add_texture_data(scene, &img));
                    // a picture the model names and the folder does not hold leaves that
                    // face plain, and a plain face looks like a part of the model rather
                    // than like a file the player forgot to copy, so it is said out loud
                    if found.is_none() && !m.texture.trim().is_empty() && !missing.contains(&m.texture) {
                        missing.push(m.texture.clone());
                    }
                    found
                };
                // the screen lights itself, as a screen does: the page's own pixels, not
                // shaded by the sun and not dark at night. Lit like the plastic around
                // it, it washed out on the sunny side and went grey on the other, which
                // is how the player found it.
                let colour = if screen { [1.0; 4] } else { m.diffuse };
                mats.push(renderer.add_material(scene, tex, AlphaMode::Opaque, colour, screen));
            }
            let inst = renderer.add_instance(scene, id, DVec3::ZERO, Mat4::IDENTITY, mats);
            desk.parts.push(inst);
            if is_screen {
                log::info!("rejsekort: the console's screen is {}", file.file_name().unwrap_or_default().to_string_lossy());
            }
        }
        match (desk.screen.is_some(), missing.as_slice()) {
            // a missing picture is the player's to put right, so it stays on the screen
            (_, [_, ..]) => say(format!("{} mangler i {}", missing.join(", "), dirs[0].display()), false),
            (true, []) => say(format!("konsol med {} dele og skærm", desk.parts.len()), true),
            // every part hung but no face wearing the screen texture: the desk is there
            // and the driver is looking at a blank panel, which is worth saying out loud
            (false, []) => say(format!("konsol med {} dele, men ingen skærm - ingen flade med {SCREEN_TEX}", desk.parts.len()), false),
        }
        WANTED.store(desk.screen.is_some(), std::sync::atomic::Ordering::Relaxed);
        desk
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Take the desk down again, before another is hung in its place.
    pub fn take_down(&self, renderer: &Renderer, scene: &mut Scene) {
        for inst in &self.parts {
            renderer.remove_instance(scene, *inst);
        }
    }

    /// Show the desk where it stands in the bus now.
    pub fn sync(&self, renderer: &Renderer, scene: &mut Scene, bus: &VehicleInstance) {
        let rot = bus.body_rotation();
        let local = self.spot.matrix();
        for inst in &self.parts {
            renderer.set_transform(scene, *inst, bus.position, rot * local);
        }
    }

    /// Hand the game down to the page, its presses on to the console, and the page's new
    /// pixels on to the screen. Called every frame; the page is drawn again only when it
    /// has something new to show.
    pub fn draw(&mut self, renderer: &Renderer, scene: &Scene, state: &serde_json::Value) {
        let keys = self.page.update(state);
        if !keys.is_empty() {
            for key in &keys {
                // a key that asks something of the game is passed on to it. The tablet
                // posts these over the network; the screen in the bus is drawn on the
                // game's own thread, so it leaves the question and does not wait.
                match crate::console::press(key) {
                    crate::console::Did::Sold => crate::tablet::ask_later(crate::tablet::Ask::Sell),
                    crate::console::Did::Ended => crate::tablet::ask_later(crate::tablet::Ask::EndTrip),
                    _ => {}
                }
            }
            // the console has moved under the page's own hands, so it is shown at once
            // rather than at the next time the game writes its state out
            let mut now = state.clone();
            now["con"] = crate::console::with(|c| c.value());
            self.page.update(&now);
        }
        let Some(screen) = self.screen.as_ref() else { return };
        if let Some(px) = self.page.frame() {
            let img = omsi_texture::Image { width: WIDTH, height: HEIGHT, rgba: px, has_alpha: false };
            renderer.update_texture(scene, screen.tex, &img);
        }
    }

    /// Where a ray lands on the screen, as the place on the page: `None` when it misses.
    /// `origin` and `dir` are in the world, as a look or a click is.
    pub fn aim(&self, bus: &VehicleInstance, origin: DVec3, dir: Vec3) -> Option<Vec2> {
        let screen = self.screen.as_ref()?;
        // (the mesh stands in the bus's own frame, so the ray is brought into it rather
        // than the desk pushed out into the world)
        let o = (origin - bus.position).as_vec3();
        let hit = omsi_geometry::ray_mesh_hit(o, dir, &screen.mesh, &(bus.body_rotation() * self.spot.matrix()))?;
        // (a face of the desk that is not the screen has its own mapping; only the
        // screen's own quad covers the whole picture, and that is the one we aim at)
        (hit.uv.x >= 0.0 && hit.uv.x <= 1.0 && hit.uv.y >= 0.0 && hit.uv.y <= 1.0).then_some(hit.uv)
    }

    /// Whether the player is looking at the console: its screen is the face he would
    /// look at, and the one already aimed at for pressing its keys.
    pub fn under(&self, bus: &VehicleInstance, origin: DVec3, dir: Vec3) -> bool {
        self.aim(bus, origin, dir).is_some()
    }

    /// Where the console stands.
    pub fn spot(&self) -> &Spot {
        &self.spot
    }

    /// Put the console somewhere else, while the game runs.
    pub fn set(&mut self, spot: Spot) {
        self.spot = spot;
    }

    /// A finger on the screen, where [`Self::aim`] says it landed.
    pub fn touch(&mut self, at: Vec2, down: bool) {
        let kind = if down { PointerKind::Down } else { PointerKind::Up };
        if down {
            self.held = true;
        } else if !std::mem::take(&mut self.held) {
            return;
        }
        self.page.touch(at.x, at.y, kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (the fork) The blank file says what to write in it, and says nothing the reader of
    /// `standere.txt` would choke on - it is the same reader.
    #[test]
    fn the_blank_file_holds_no_console() {
        assert!(Spot::read(BLANK).is_empty(), "every line of it is a comment");
        assert!(BLANK.contains("<bus>") && BLANK.contains("Konsol/"));
        assert_eq!(named_folder(BLANK), None, "and names no folder either");
    }

    /// (the fork) The player already had the console as an OMSI scenery object, under
    /// `Sceneryobjects\...`. Making him copy sixteen files out of there would be silly,
    /// so the file names the folder and the model stays where it is.
    #[test]
    fn the_model_can_stay_where_the_player_keeps_it() {
        let named = "# the console I already have\nmappe  Sceneryobjects\\UrbanBacon\\Models\\Rejsekort\n*  -0.95 5.95 0.95  180\n";
        assert_eq!(named_folder(named), Some("Sceneryobjects\\UrbanBacon\\Models\\Rejsekort"));
        // the line that places it is still read as it always was
        assert_eq!(Spot::read(named).len(), 1, "the mappe line is not a place");

        let root = Path::new("/omsi");
        let dirs = model_dirs(root, named);
        assert!(dirs[0].ends_with("Rejsekort"), "{:?}", dirs[0]);
        // OMSI keeps a scenery object's meshes in Model and its pictures in Texture, and
        // this one's are in Textures, so all of them are looked in
        let ends: Vec<String> = dirs.iter().map(|d| d.file_name().unwrap_or_default().to_string_lossy().into_owned()).collect();
        assert_eq!(ends, ["Rejsekort", "Model", "Texture", "Textures"]);

        // with no mappe line it is the stock folder beside standere.txt
        let stock = model_dirs(root, "*  0 0 0  0\n");
        assert!(stock[0].ends_with(FOLDER), "{:?}", stock[0]);

        // a mappe line with nothing after it names no folder
        for text in ["mappe\n", "mappe   \n", "# mappe Sceneryobjects\n"] {
            assert_eq!(named_folder(text), None, "{text:?}");
        }
    }

    /// (the fork) A console that does not turn up in the bus looks exactly like one
    /// standing somewhere out of sight, so the game says which of the two it is - but
    /// only once the player has asked for one at all.
    #[test]
    fn the_game_says_what_came_of_the_console() {
        say_nothing();
        assert_eq!(hud_line(), None, "nothing is said until the player asks for a console");

        say("konsol.txt har ingen linje til MB_O530.bus".into(), false);
        let said = hud_line().expect("a console that is not there is worth a word");
        assert!(said.starts_with("Rejsekort: "), "{said}");
        assert!(said.contains("ingen linje"), "{said}");

        // one that worked says so and then goes; one that did not stays put
        say("konsol med 16 dele og skærm".into(), true);
        assert!(hud_line().is_some_and(|s| s.contains("16 dele")));
        NOTE.lock().unwrap().as_mut().unwrap().at -= SAY_FOR * 2;
        assert_eq!(hud_line(), None, "a note that it worked does not sit there all drive");

        say("ingen .o3d-filer i C:\\OMSI\\Rejsekort\\Konsol".into(), false);
        NOTE.lock().unwrap().as_mut().unwrap().at -= SAY_FOR * 4;
        assert!(hud_line().is_some(), "a note that it did not work stays until it is put right");
        say_nothing();
    }

    /// (the fork) The screen is found by the texture the player's own model already names,
    /// whatever case he wrote it in, so nothing has to be renamed to make it live.
    #[test]
    fn the_screen_is_the_face_wearing_the_screen_texture() {
        for name in ["Skærmtexture.png", "SKÆRMTEXTURE.PNG", "skærmtexture.dds"] {
            assert!(name.to_lowercase().contains(SCREEN_TEX), "{name}");
        }
        for name in ["Rkpiltaster.png", "", "bord.png"] {
            assert!(!name.to_lowercase().contains(SCREEN_TEX), "{name}");
        }
    }
}
