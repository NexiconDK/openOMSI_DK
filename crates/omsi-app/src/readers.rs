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
/// The file the standers' places are written in.
pub const SPOTS: &str = "standere.txt";

/// The file the player writes the standers' places in, when there is none yet.
pub const BLANK: &str = "\
# Rejsekort-standere i bussen. En linje pr. stander, i den rækkefølge de skal\n\
# nummereres: den første er stander 1, salgskortlæseren ved chaufføren.\n\
#\n\
#   <bus>  <x> <y> <z>  <drejning>  [skala] [hældning] [kip]\n\
#\n\
# <bus> er bussens .bus-filnavn (uden mappe), eller * for alle busser.\n\
# x er til højre, y er fremad og z er opad, i meter fra bussens nulpunkt.\n\
#\n\
# De tre sidste tal er valgfri. Står de der ikke, er skalaen 1 og standeren\n\
# står lodret, præcis som før de fandtes.\n\
#\n\
# Drejning:  grader om lodret - hvilken vej den vender.\n\
#            0 vender standeren mod bussens højre side.\n\
# Skala:     1 er modellen i sin egen størrelse. checkind.o3d er 45 cm bred og\n\
#            68 cm høj, men en rigtig Rejsekort-stander er cirka 21 x 30 cm, så\n\
#            omkring 0.45 rammer den rigtige størrelse.\n\
# Hældning:  grader om tværaksen - plus vipper toppen fremad i bussen.\n\
# Kip:       grader om længdeaksen - plus hælder toppen mod højre.\n\
#\n\
# Der drejes og skaleres om modellens eget nulpunkt, så ret x, y og z til bagefter.\n\
#\n\
# MODELLEN\n\
# checkind.o3d og de to skind ledes der efter her i mappen. Har du dem liggende\n\
# som et OMSI-sceneryobjekt i forvejen, så lad dem blive, hvor de er, og peg på\n\
# mappen i stedet:\n\
#\n\
#   mappe  Sceneryobjects\\UrbanBacon\\Models\\Rejsekort\n\
#\n\
# Stien er fra din OMSI-mappe. Der ledes i mappen selv og i Model/, Texture/ og\n\
# Textures/ ved siden af, som OMSI plejer at lægge dem.\n\
#\n\
# Linjer, der begynder med #, er kommentarer.\n\
\n\
# *   -1.00   5.20   1.05    0    0.45\n\
# *   -1.00   1.60   1.05    0    0.45   15\n\
";

/// Where one stander stands, in the bus's own frame.
#[derive(Clone, PartialEq, Debug)]
pub struct Spot {
    /// The bus this belongs to (its `.bus` file name), or none for every bus.
    pub bus: Option<String>,
    pub at: Vec3,
    /// Degrees about the upright (z): which way it faces.
    pub turn: f32,
    /// How large the model is drawn, 1 being its own size. The player's own models are
    /// built at whatever size he built them at - the Rejsekort stander is 45 cm across
    /// where a real one is 21 - so this is how he brings one down to size without
    /// opening the model again.
    pub scale: f32,
    /// Degrees about the cross axis (x): a positive one tips the top **forward**, towards
    /// the front of the bus, for a reader held out to the customer rather than standing
    /// straight up.
    pub tip: f32,
    /// Degrees about the length axis (y): a positive one leans the top to the **right**,
    /// for one on a slanted panel or a wall that is not upright.
    pub lean: f32,
}

impl Spot {
    /// Where it sits in the bus, as a matrix. The mesh arrives in the bus's own frame
    /// already (`mesh_from_o3d` swaps the file's Y and Z), so this only moves, turns and
    /// sizes it. All three are about the model's own origin.
    ///
    /// The three turns are taken in the order a person sets them: first which way it
    /// faces, then how far it is tipped forward, then how far it leans to one side.
    /// The tip is negated so that a positive one tips the top forward rather than back,
    /// which is the way round anybody would expect who has not thought about axes.
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_translation(self.at)
            * Mat4::from_euler(glam::EulerRot::ZXY, self.turn.to_radians(), -self.tip.to_radians(), self.lean.to_radians())
            * Mat4::from_scale(Vec3::splat(self.scale))
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
            // the last three columns may each be left out, so a line written before they
            // existed still means what it meant: the model in its own size, standing
            // straight up
            let opt = |i: usize, stock: f32| match w.get(i) {
                None => Some(stock),
                Some(s) => s.parse::<f32>().ok(),
            };
            let scale = opt(5, 1.0).filter(|v| *v > 0.0);
            match (nums, scale, opt(6, 0.0), opt(7, 0.0)) {
                (Some(v), Some(scale), Some(tip), Some(lean)) => out.push(Spot {
                    bus: (w[0] != "*").then(|| w[0].to_string()),
                    at: Vec3::new(v[0], v[1], v[2]),
                    turn: v[3],
                    scale,
                    tip,
                    lean,
                }),
                _ => log::warn!("{SPOTS} line {}: {line:?} is not <bus> <x> <y> <z> <turn> [scale] [tip] [lean]", n + 1),
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
    /// (the two instances of one stander, and the line that puts it there). The line is
    /// kept rather than the matrix it makes, so the placer can move a stander about while
    /// the game runs and the same line can be written back to the file.
    standers: Vec<([usize; 2], Spot)>,
    /// When `standere.txt` was last written, and when that was last looked at: the player
    /// puts the standers in their places by trying a line and looking, so the file is read
    /// again when he saves it rather than making him start the game over.
    wrote: Option<std::time::SystemTime>,
    looked: std::time::Instant,
    /// The stander's own shape, for working out which one a player is looking at. One
    /// copy serves every stander, since they are all the same model.
    mesh: Option<omsi_geometry::MeshData>,
}

/// How often `standere.txt` is looked at for a change.
const LOOK_EVERY: std::time::Duration = std::time::Duration::from_millis(500);

/// When the file was last written, or `None` when there is none.
fn written(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// The word that names the folder a model is in, when it is not the stock one.
pub const FOLDER_WORD: &str = "mappe";

/// The folder a placement file names its model in, if it names one: a `mappe` line
/// holding a path from the OMSI folder. A model the player already has as an OMSI
/// scenery object lives under `Sceneryobjects\...`, and there is no sense in making him
/// copy it out of there into a folder of ours.
pub fn named_folder(text: &str) -> Option<&str> {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .find_map(|l| l.strip_prefix(FOLDER_WORD).map(str::trim).filter(|r| !r.is_empty()))
}

/// Where a model's own pieces are looked for, in order: the folder the file names, else
/// `stock`. OMSI keeps a scenery object's meshes in a `Model` folder beside it and its
/// pictures in `Texture` (or `Textures`, as the player's own console has it), so those
/// are searched too and the first that holds a thing wins.
pub fn model_dirs(root: &Path, text: &str, stock: &Path) -> Vec<PathBuf> {
    let base = match named_folder(text) {
        Some(rel) => omsi_cfg::resolve_path(root, rel),
        None => stock.to_path_buf(),
    };
    let mut dirs = vec![base.clone()];
    for sub in ["Model", "Texture", "Textures"] {
        let d = omsi_cfg::resolve_path(&base, sub);
        if d != base {
            dirs.push(d);
        }
    }
    dirs
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
        let spots = dir.join(SPOTS);
        let blank =
            || Readers { made_for: made_for.clone(), skins: [None; 2], standers: Vec::new(), wrote: written(&spots), looked: std::time::Instant::now(), mesh: None };
        if !spots.is_file() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&spots, BLANK);
            log::info!("{}: written blank, waiting for the standers' places", spots.display());
        }
        // the file is read before the model, since it is the file that may say where the
        // model is: a stander the player already keeps as a scenery object stays there
        let text = std::fs::read_to_string(&spots).unwrap_or_default();
        let dirs = model_dirs(&world.root, &text, &dir);
        let looks_in: Vec<&Path> = dirs.iter().map(PathBuf::as_path).collect();
        let Some(file) = looks_in.iter().map(|d| omsi_cfg::resolve_path(d, MESH)).find(|p| p.is_file()) else {
            log::warn!("{}: no {MESH} there", dirs[0].display());
            return blank();
        };
        let Ok(mesh) = omsi_o3d::load_mesh(&file).map_err(|e| log::warn!("{}: {e}", file.display())) else {
            return blank();
        };
        let name = bus.ty.def.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let mine: Vec<Spot> = Spot::read(&text).into_iter().filter(|s| s.fits(&name)).collect();
        if mine.is_empty() {
            return blank();
        }

        let data = mesh_from_o3d(&mesh);
        let id = renderer.add_mesh(scene, &data);
        let mut skin = |which: usize| -> Option<MaterialId> {
            let p = omsi_texture::find_texture(SKINS[which], &looks_in)?;
            let (img, _) = world.textures.get_gpu_fast(&p)?;
            let tex = renderer.add_texture_data(scene, &img);
            Some(renderer.add_material(scene, Some(tex), AlphaMode::Opaque, [1.0; 4], false))
        };
        let mats = [skin(0), skin(1)];
        if mats.iter().any(Option::is_none) {
            log::warn!("{}: {} or {} is missing", dirs[0].display(), SKINS[0], SKINS[1]);
        }
        let standers: Vec<([usize; 2], Spot)> = mine
            .iter()
            .map(|s| {
                let pair = [0, 1].map(|k| {
                    let m = mats[k].into_iter().collect();
                    renderer.add_instance(scene, id, DVec3::ZERO, Mat4::IDENTITY, m)
                });
                (pair, s.clone())
            })
            .collect();
        log::info!("rejsekort: {} standers in {}", standers.len(), name);
        Readers { made_for, skins: [Some(0), Some(1)], standers, wrote: written(&spots), looked: std::time::Instant::now(), mesh: Some(data) }
    }

    /// Whether `standere.txt` has been saved again since the standers were put up, so
    /// they are to be put up afresh. The disk is asked at most twice a second.
    pub fn moved(&mut self, root: &Path) -> bool {
        if self.looked.elapsed() < LOOK_EVERY {
            return false;
        }
        self.looked = std::time::Instant::now();
        let now = written(&Self::folder(root).join(SPOTS));
        if now == self.wrote {
            return false;
        }
        self.wrote = now;
        log::info!("{SPOTS}: saved again, putting the standers up afresh");
        true
    }

    /// Which stander a ray lands on, nearest first: the one the player is looking at.
    pub fn hit(&self, bus: &VehicleInstance, origin: DVec3, dir: Vec3) -> Option<usize> {
        let data = self.mesh.as_ref()?;
        let o = (origin - bus.position).as_vec3();
        let rot = bus.body_rotation();
        self.standers
            .iter()
            .enumerate()
            .filter_map(|(i, (_, spot))| omsi_geometry::ray_mesh(o, dir, data, &(rot * spot.matrix())).map(|t| (t, i)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, i)| i)
    }

    /// Where stander `i` stands.
    pub fn spot(&self, i: usize) -> Option<&Spot> {
        self.standers.get(i).map(|(_, s)| s)
    }

    /// Put stander `i` somewhere else, while the game runs.
    pub fn set(&mut self, i: usize, spot: Spot) {
        if let Some((_, s)) = self.standers.get_mut(i) {
            *s = spot;
        }
    }

    /// Take the standers down again, before others are put up in their place.
    pub fn take_down(&self, renderer: &Renderer, scene: &mut Scene) {
        for (pair, _) in &self.standers {
            for inst in pair {
                renderer.remove_instance(scene, *inst);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.standers.is_empty() || self.skins.iter().any(Option::is_none)
    }

    /// Show the standers where they stand in the bus now, each in the skin the console says
    /// that reader is set to: green **Check ind** or red **Check ud**.
    pub fn sync(&self, renderer: &Renderer, scene: &mut Scene, bus: &VehicleInstance, takes_in: &[bool]) {
        let rot = bus.body_rotation();
        for (k, (pair, spot)) in self.standers.iter().enumerate() {
            let local = spot.matrix();
            // a stander the console knows nothing about takes cards in, as the yard leaves
            // the front reader
            let want = usize::from(takes_in.get(k).copied().unwrap_or(k == 0));
            for (which, inst) in pair.iter().enumerate() {
                renderer.set_transform(scene, *inst, bus.position, rot * local);
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
MB_O530.bus  -1.02  1.60  1.05  180   0.45

  not a line at all
*  -1.00  bad  1.05  0
*  -1.00  3.00  1.05  90   # with a comment after it
*  -1.00  4.00  1.05  0   0
*  -1.00  4.00  1.05  0   huge
";
        let got = Spot::read(text);
        assert_eq!(got.len(), 3, "the four bad lines are passed over: {got:?}");
        assert_eq!(got[0].bus, None);
        assert_eq!(got[0].at, Vec3::new(-1.00, 5.20, 1.05));
        // a line written before the last three columns existed means what it always meant
        assert_eq!(got[0].scale, 1.0, "the model's own size");
        assert_eq!((got[0].tip, got[0].lean), (0.0, 0.0), "standing straight up");
        assert_eq!(got[1].bus.as_deref(), Some("MB_O530.bus"));
        assert_eq!(got[1].turn, 180.0);
        assert_eq!(got[1].scale, 0.45);
        assert_eq!(got[2].at.y, 3.00, "a comment after the numbers is still a line");

        // and the two turns beyond the upright one, each on its own or together
        let more = Spot::read("*  0 0 0  90  0.45  15\n*  0 0 0  0  1  -8  20\n");
        assert_eq!(more.len(), 2, "{more:?}");
        assert_eq!((more[0].tip, more[0].lean), (15.0, 0.0));
        assert_eq!((more[1].tip, more[1].lean), (-8.0, 20.0));

        // a line for one bus belongs to that bus alone; a * belongs to every one
        assert!(got[0].fits("anything.bus"));
        assert!(got[1].fits("mb_o530.bus"), "the file name is matched whatever its case");
        assert!(!got[1].fits("Urbino_18.bus"));
    }

    /// (the fork) The place is the stander's own, in the bus's frame: moved and turned
    /// about the upright, never scaled or tipped.
    #[test]
    fn a_stander_stands_where_it_is_put() {
        let s = Spot { bus: None, at: Vec3::new(-1.0, 5.2, 1.05), turn: 90.0, scale: 1.0, tip: 0.0, lean: 0.0 };
        let m = s.matrix();
        // its own origin lands where the line says
        assert!((m.transform_point3(Vec3::ZERO) - s.at).length() < 1e-5);
        // turned a quarter round, what pointed right points forward
        let face = m.transform_vector3(Vec3::X);
        assert!((face - Vec3::Y).length() < 1e-5, "{face:?}");
        // and nothing is stretched on the way
        assert!((m.transform_vector3(Vec3::Z).length() - 1.0).abs() < 1e-5);
    }

    /// (the fork) The player found the stander towering over the seats: the model is 45 cm
    /// across where a real Rejsekort stander is 21. The scale brings it down to size
    /// without opening the model, and it keeps its shape - no stretching in one direction.
    #[test]
    fn a_stander_is_brought_down_to_size() {
        let s = Spot { bus: None, at: Vec3::new(-1.0, 5.2, 1.05), turn: 0.0, scale: 0.45, tip: 0.0, lean: 0.0 };
        let m = s.matrix();
        // where it stands is where the line says, whatever size it is drawn at
        assert!((m.transform_point3(Vec3::ZERO) - s.at).length() < 1e-5);
        // and every direction is brought down by the same amount
        for v in [Vec3::X, Vec3::Y, Vec3::Z] {
            assert!((m.transform_vector3(v).length() - 0.45).abs() < 1e-5, "{v:?}");
        }
        // checkind.o3d is 0.451 m across and 0.682 m tall; drawn at this scale it is the
        // size of a real Rejsekort stander, about 21 by 30 cm
        let across = m.transform_vector3(Vec3::new(0.451, 0.0, 0.0)).length();
        let tall = m.transform_vector3(Vec3::new(0.0, 0.0, 0.682)).length();
        assert!((across - 0.203).abs() < 0.01, "about 20 cm across, not {across}");
        assert!((tall - 0.307).abs() < 0.01, "about 31 cm tall, not {tall}");
    }

    /// (the fork) A stander can be tipped and leaned as well as turned, for one held out
    /// to the customer or sitting on a panel that is not upright. Each turn is about the
    /// model's own origin, and none of them changes where it stands or how big it is.
    #[test]
    fn a_stander_can_be_tipped_and_leaned() {
        let flat = Spot { bus: None, at: Vec3::new(-1.0, 5.2, 1.05), turn: 0.0, scale: 1.0, tip: 0.0, lean: 0.0 };

        // tipped a quarter round forward, what pointed up points forward
        let s = Spot { tip: 90.0, ..flat.clone() };
        let up = s.matrix().transform_vector3(Vec3::Z);
        assert!((up - Vec3::Y).length() < 1e-5, "a positive tip goes forward, not {up:?}");

        // leaned a quarter round, what pointed up points to the right
        let s = Spot { lean: 90.0, ..flat.clone() };
        let up = s.matrix().transform_vector3(Vec3::Z);
        assert!((up - Vec3::X).length() < 1e-5, "a positive lean goes right, not {up:?}");

        // however it is turned, it stands where the line says and keeps its size
        for s in [Spot { tip: 20.0, lean: -35.0, turn: 110.0, ..flat.clone() }, Spot { tip: -70.0, ..flat.clone() }] {
            let m = s.matrix();
            assert!((m.transform_point3(Vec3::ZERO) - s.at).length() < 1e-5, "{s:?}");
            for v in [Vec3::X, Vec3::Y, Vec3::Z] {
                assert!((m.transform_vector3(v).length() - 1.0).abs() < 1e-5, "{s:?} along {v:?}");
            }
        }
    }

    /// (the fork) The blank file says what to write in it, and says nothing the reader
    /// would choke on.
    #[test]
    fn the_blank_file_holds_no_standers() {
        assert!(Spot::read(BLANK).is_empty(), "every line of it is a comment");
        assert!(BLANK.contains("<bus>") && BLANK.contains("drejning"));
    }
}
