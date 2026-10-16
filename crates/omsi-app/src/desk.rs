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

/// The folder in `Rejsekort` the player drops the console's own model in.
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
#   <bus>  <x> <y> <z>  <drejning>  [skala]\n\
#\n\
# <bus> er bussens .bus-filnavn (uden mappe), eller * for alle busser.\n\
# x er til højre, y er fremad og z er opad, i meter fra bussens nulpunkt.\n\
# Drejningen er grader om lodret: 0 vender konsollen mod bussens højre side.\n\
#\n\
# Skalaen er valgfri og 1 som standard. Bordet er 40 x 40 cm i modellen og\n\
# skærmen 12,8 x 12,2 cm; passer det ikke i din bus, så skru på tallet.\n\
# Den skaleres om modellens eget nulpunkt, så ret x, y og z til bagefter.\n\
#\n\
# Selve modellen ligger i undermappen Konsol/ — alle .o3d-filer der bliver hængt op\n\
# sammen, og den flade, hvis tekstur hedder Skærmtexture.png, bliver skærmen.\n\
\n\
# *   -0.95   5.95   0.95   180   1.0\n\
";

/// Whether a console screen is standing in a bus, so the game keeps its state up to date
/// for it even when the tablet is switched off.
static WANTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the console's screen is being drawn in a bus just now.
pub fn wanted() -> bool {
    WANTED.load(std::sync::atomic::Ordering::Relaxed)
}

/// The console desk of one bus, hung in it.
pub struct Desk {
    /// The bus and the folder it was made for.
    pub made_for: (PathBuf, PathBuf),
    /// Every part of the desk, each with where it hangs in the bus.
    parts: Vec<(usize, Mat4)>,
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

struct Screen {
    tex: TextureId,
    mesh: MeshData,
    local: Mat4,
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
        let Some(spot) = Spot::read(&text).into_iter().find(|s| s.fits(&name)) else {
            return desk;
        };
        let local = spot.matrix();

        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("o3d")))
            .collect();
        // (a folder lists its files in whatever order it likes; the same order every time
        // keeps the log and any report of it steady)
        files.sort();
        if files.is_empty() {
            log::info!("{}: no console model there yet", dir.display());
            return desk;
        }

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
                            desk.screen = Some(Screen { tex: t, mesh: data.clone(), local });
                            is_screen = true;
                            t
                        }
                    };
                    Some(t)
                } else {
                    omsi_texture::find_texture(&m.texture, &[dir.as_path()])
                        .and_then(|p| world.textures.get_gpu_fast(&p))
                        .map(|(img, _)| renderer.add_texture_data(scene, &img))
                };
                mats.push(renderer.add_material(scene, tex, AlphaMode::Opaque, m.diffuse, false));
            }
            let inst = renderer.add_instance(scene, id, DVec3::ZERO, Mat4::IDENTITY, mats);
            desk.parts.push((inst, local));
            if is_screen {
                log::info!("rejsekort: the console's screen is {}", file.file_name().unwrap_or_default().to_string_lossy());
            }
        }
        log::info!("rejsekort: a console of {} parts in {name}", desk.parts.len());
        WANTED.store(desk.screen.is_some(), std::sync::atomic::Ordering::Relaxed);
        desk
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Take the desk down again, before another is hung in its place.
    pub fn take_down(&self, renderer: &Renderer, scene: &mut Scene) {
        for (inst, _) in &self.parts {
            renderer.remove_instance(scene, *inst);
        }
    }

    /// Show the desk where it stands in the bus now.
    pub fn sync(&self, renderer: &Renderer, scene: &mut Scene, bus: &VehicleInstance) {
        let rot = bus.body_rotation();
        for (inst, local) in &self.parts {
            renderer.set_transform(scene, *inst, bus.position, rot * *local);
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
        let hit = omsi_geometry::ray_mesh_hit(o, dir, &screen.mesh, &(bus.body_rotation() * screen.local))?;
        // (a face of the desk that is not the screen has its own mapping; only the
        // screen's own quad covers the whole picture, and that is the one we aim at)
        (hit.uv.x >= 0.0 && hit.uv.x <= 1.0 && hit.uv.y >= 0.0 && hit.uv.y <= 1.0).then_some(hit.uv)
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
