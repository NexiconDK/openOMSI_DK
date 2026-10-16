//! (the fork) The Rejsekort standers in the bus: the card readers the customers hold their
//! cards to, drawn where the player put them and wearing the skin the console's **Kortlæser
//! opsætning** says that reader is set to.
//!
//! The model and its two skins are the player's own, in a `Rejsekort` folder beside the
//! Web-IBIS's `WebIBIS` one. The mesh carries one material with no texture of its own,
//! which is what makes the console's setting show on the stander itself: the same mesh is
//! hung twice, once in each skin, and whichever the reader is set to is the one shown.
//!
//! ```text
//! <OMSI folder>/Rejsekort/checkind.o3d     the stander
//! <OMSI folder>/Rejsekort/Checkind.tga     the green Check ind skin
//! <OMSI folder>/Rejsekort/Checkud.tga      the red Check ud skin
//! <OMSI folder>/Rejsekort/standere.txt     where they stand (see `Spot::read`)
//! ```

use crate::scene::World;
use glam::{DVec3, Mat4, Vec3};
use omsi_geometry::mesh_from_o3d;
use omsi_render::{AlphaMode, MaterialId, Renderer, Scene};
use omsi_sim::VehicleInstance;
use std::path::{Path, PathBuf};

/// The folder in the OMSI root the player drops the stander and its skins in.
pub const FOLDER: &str = "Rejsekort";
const MESH: &str = "checkind.o3d";
const SKINS: [&str; 2] = ["Checkud.tga", "Checkind.tga"];
const SPOTS: &str = "standere.txt";

/// The file the player writes the standers' places in, when there is none yet.
pub const BLANK: &str = "\
# Rejsekort-standere i bussen. En linje pr. stander, i den rækkefølge de skal\n\
# nummereres: den første er stander 1, salgskortlæseren ved chaufføren.\n\
#\n\
#   <bus>  <x> <y> <z>  <drejning>\n\
#\n\
# <bus> er bussens .bus-filnavn (uden mappe), eller * for alle busser.\n\
# x er til højre, y er fremad og z er opad, i meter fra bussens nulpunkt.\n\
# Drejningen er grader om lodret: 0 vender standeren mod bussens højre side.\n\
#\n\
# Linjer, der begynder med #, er kommentarer.\n\
\n\
# *   -1.00   5.20   1.05    0\n\
# *   -1.00   1.60   1.05    0\n\
";

/// Where one stander stands, in the bus's own frame.
#[derive(Clone, PartialEq, Debug)]
pub struct Spot {
    /// The bus this belongs to (its `.bus` file name), or none for every bus.
    pub bus: Option<String>,
    pub at: Vec3,
    /// Degrees about the upright.
    pub turn: f32,
}

impl Spot {
    /// Where it sits in the bus, as a matrix. The mesh arrives in the bus's own frame
    /// already (`mesh_from_o3d` swaps the file's Y and Z), so this only moves and turns it.
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.at) * Mat4::from_rotation_z(self.turn.to_radians())
    }

    /// Whether this line belongs to the bus whose `.bus` file is `file`.
    pub fn fits(&self, file: &str) -> bool {
        self.bus.as_ref().is_none_or(|b| b.eq_ignore_ascii_case(file))
    }

    /// The standers a `standere.txt` names, in the order they are written - the first line
    /// is stander 1. A line that cannot be read is passed over with a word in the log
    /// rather than taking the rest of the file with it.
    pub fn read(text: &str) -> Vec<Spot> {
        let mut out = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let w: Vec<&str> = line.split_whitespace().collect();
            let nums: Option<Vec<f32>> = (w.len() >= 5).then(|| w[1..5].iter().map(|v| v.parse().ok()).collect::<Option<Vec<f32>>>()).flatten();
            match nums {
                Some(v) => out.push(Spot {
                    bus: (w[0] != "*").then(|| w[0].to_string()),
                    at: Vec3::new(v[0], v[1], v[2]),
                    turn: v[3],
                }),
                None => log::warn!("{SPOTS} line {}: {line:?} is not <bus> <x> <y> <z> <turn>", n + 1),
            }
        }
        out
    }
}

/// The standers of one bus, drawn.
pub struct Readers {
    /// The bus and the folder they were made for.
    pub made_for: (PathBuf, PathBuf),
    /// Both skins of the mesh: index 0 takes cards out (red), 1 takes them in (green).
    skins: [Option<usize>; 2],
    /// (the two instances of one stander, where it stands)
    standers: Vec<([usize; 2], Mat4)>,
}

impl Readers {
    /// The folder the player's model and skins are in.
    pub fn folder(root: &Path) -> PathBuf {
        omsi_cfg::resolve_path(root, FOLDER)
    }

    /// The standers of this bus, or none at all when the folder holds no model. A missing
    /// `standere.txt` is written out blank, so the player has something to fill in.
    pub fn new(world: &World, renderer: &Renderer, scene: &mut Scene, bus: &VehicleInstance) -> Readers {
        let dir = Self::folder(&world.root);
        let made_for = (bus.ty.def.path.clone(), dir.clone());
        let file = dir.join(MESH);
        let Ok(mesh) = omsi_o3d::load_mesh(&file).map_err(|e| log::warn!("{}: {e}", file.display())) else {
            return Readers { made_for, skins: [None; 2], standers: Vec::new() };
        };
        let spots = dir.join(SPOTS);
        if !spots.is_file() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&spots, BLANK);
            log::info!("{}: written blank, waiting for the standers' places", spots.display());
        }
        let text = std::fs::read_to_string(&spots).unwrap_or_default();
        let name = bus.ty.def.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let mine: Vec<Spot> = Spot::read(&text).into_iter().filter(|s| s.fits(&name)).collect();
        if mine.is_empty() {
            return Readers { made_for, skins: [None; 2], standers: Vec::new() };
        }

        let data = mesh_from_o3d(&mesh);
        let id = renderer.add_mesh(scene, &data);
        let mut skin = |which: usize| -> Option<MaterialId> {
            let p = omsi_texture::find_texture(SKINS[which], &[dir.as_path()])?;
            let (img, _) = world.textures.get_gpu_fast(&p)?;
            let tex = renderer.add_texture_data(scene, &img);
            Some(renderer.add_material(scene, Some(tex), AlphaMode::Opaque, [1.0; 4], false))
        };
        let mats = [skin(0), skin(1)];
        if mats.iter().any(Option::is_none) {
            log::warn!("{}: {} or {} is missing", dir.display(), SKINS[0], SKINS[1]);
        }
        let standers: Vec<([usize; 2], Mat4)> = mine
            .iter()
            .map(|s| {
                let pair = [0, 1].map(|k| {
                    let m = mats[k].into_iter().collect();
                    renderer.add_instance(scene, id, DVec3::ZERO, Mat4::IDENTITY, m)
                });
                (pair, s.matrix())
            })
            .collect();
        log::info!("rejsekort: {} standers in {}", standers.len(), name);
        Readers { made_for, skins: [Some(0), Some(1)], standers }
    }

    pub fn is_empty(&self) -> bool {
        self.standers.is_empty() || self.skins.iter().any(Option::is_none)
    }

    /// Show the standers where they stand in the bus now, each in the skin the console says
    /// that reader is set to: green **Check ind** or red **Check ud**.
    pub fn sync(&self, renderer: &Renderer, scene: &mut Scene, bus: &VehicleInstance, takes_in: &[bool]) {
        let rot = bus.body_rotation();
        for (k, (pair, local)) in self.standers.iter().enumerate() {
            // a stander the console knows nothing about takes cards in, as the yard leaves
            // the front reader
            let want = usize::from(takes_in.get(k).copied().unwrap_or(k == 0));
            for (which, inst) in pair.iter().enumerate() {
                renderer.set_transform(scene, *inst, bus.position, rot * *local);
                renderer.set_params(scene, *inst, &[], which == want, &[]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (the fork) The player writes the standers' places in a plain file, and a line he has
    /// mistyped is passed over rather than taking the rest of the file with it.
    #[test]
    fn the_standers_are_read_from_their_file() {
        let text = "\
# the sales reader by the driver
*       -1.00   5.20   1.05     0
MB_O530.bus  -1.02  1.60  1.05  180

  not a line at all
*  -1.00  bad  1.05  0
*  -1.00  3.00  1.05  90   # with a comment after it
";
        let got = Spot::read(text);
        assert_eq!(got.len(), 3, "the two bad lines are passed over: {got:?}");
        assert_eq!(got[0].bus, None);
        assert_eq!(got[0].at, Vec3::new(-1.00, 5.20, 1.05));
        assert_eq!(got[1].bus.as_deref(), Some("MB_O530.bus"));
        assert_eq!(got[1].turn, 180.0);
        assert_eq!(got[2].at.y, 3.00, "a comment after the numbers is still a line");

        // a line for one bus belongs to that bus alone; a * belongs to every one
        assert!(got[0].fits("anything.bus"));
        assert!(got[1].fits("mb_o530.bus"), "the file name is matched whatever its case");
        assert!(!got[1].fits("Urbino_18.bus"));
    }

    /// (the fork) The place is the stander's own, in the bus's frame: moved and turned
    /// about the upright, never scaled or tipped.
    #[test]
    fn a_stander_stands_where_it_is_put() {
        let s = Spot { bus: None, at: Vec3::new(-1.0, 5.2, 1.05), turn: 90.0 };
        let m = s.matrix();
        // its own origin lands where the line says
        assert!((m.transform_point3(Vec3::ZERO) - s.at).length() < 1e-5);
        // turned a quarter round, what pointed right points forward
        let face = m.transform_vector3(Vec3::X);
        assert!((face - Vec3::Y).length() < 1e-5, "{face:?}");
        // and nothing is stretched on the way
        assert!((m.transform_vector3(Vec3::Z).length() - 1.0).abs() < 1e-5);
    }

    /// (the fork) The blank file says what to write in it, and says nothing the reader
    /// would choke on.
    #[test]
    fn the_blank_file_holds_no_standers() {
        assert!(Spot::read(BLANK).is_empty(), "every line of it is a comment");
        assert!(BLANK.contains("<bus>") && BLANK.contains("drejning"));
    }
}
