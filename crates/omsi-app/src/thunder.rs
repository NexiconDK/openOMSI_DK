//! Lightning and thunder: the flash, the channel drawn on the sky and the clap that
//! follows it.
//!
//! OMSI 2's weather has no thunderstorm. This one is driven either by the weather - a METAR
//! report with `TS` (see `omsi_content::weather`), or a heavy shower, which more often than
//! not is one - or by `OMSI_THUNDER` as a switch of its own, so it can be turned on and seen
//! at once.
//!
//! A strike is kept here until both its light has died and its thunder has been heard: the
//! sound travels 343 m a second, so a flash four kilometres off is followed twelve seconds
//! later by the clap, which is the whole of what makes a storm feel far away or overhead.
//! The clap itself is made in code (`clap`), so no sound file has to ship with the game.

use glam::Vec3;
use omsi_audio::{AudioEngine, Clip, VoiceParams};
use std::sync::Arc;

/// The speed of sound in air (m/s, near 15 °C): the delay from the flash to the clap.
const SOUND_SPEED: f32 = 343.0;
/// How near and how far a strike can be (m).
const NEAR: f32 = 250.0;
const FAR: f32 = 14_000.0;
/// The points of a channel at most, the shader's array (see `omsi_render::Lightning`).
pub(crate) const BOLT_POINTS: usize = omsi_render::BOLT_POINTS;
/// Return strokes of one flash at most (the leader and up to three more).
const STROKES: usize = 4;
/// The sample rate the claps are made at: thunder holds nothing above a few kHz, and a
/// lower rate halves the samples to filter.
const CLAP_RATE: u32 = 22_050;
/// Claps kept per distance band, picked between so the same roll is not heard twice over.
const CLAP_VARIANTS: usize = 3;

/// The storm of the one session a process runs, as `road_snow` and `terrain_paint` keep
/// theirs: the window and the offscreen pictures both take it, and the frame only hands
/// over the weather.
static STATE: parking_lot::Mutex<Option<Thunder>> = parking_lot::Mutex::new(None);

/// One frame of `dt` weather seconds (see [`Thunder::step`]).
pub(crate) fn step(dt: f32, weather: Option<&omsi_content::weather::Weather>, engine: Option<&AudioEngine>, inside: bool) {
    let mut state = STATE.lock();
    state.get_or_insert_with(Default::default).step(dt, weather, engine, inside);
}

/// What the picture draws the lightning with (nothing while no stroke burns).
///
/// A flash at night is blinding and by day no more than bright, because the eye is adapted
/// to the dark - and the renderers add it in their own exposed terms rather than metering
/// it, so the difference has to be made here, where the daylight is known.
pub(crate) fn light(lighting: &mut omsi_render::Lighting) {
    let night = lighting.night.clamp(0.0, 1.0);
    let Some(mut l) = STATE.lock().as_ref().and_then(Thunder::light) else { return };
    l.strength *= 1.0 + 2.0 * night;
    lighting.lightning = Some(l);
}

/// Lightning and thunder over time: how close the storm is, when it next strikes, and the
/// strikes still lighting the sky or still to be heard.
pub(crate) struct Thunder {
    /// 0 no storm, 1 one overhead: how often it strikes and how near.
    strength: f32,
    /// Seconds until the next strike.
    wait: f32,
    rng: u64,
    /// Strikes whose light still shows or whose thunder has not arrived.
    strikes: Vec<Strike>,
    /// The claps made so far, by band and variant; made at the first strike that needs one
    /// and kept.
    claps: Vec<Option<Arc<Clip>>>,
    /// The claps still rolling and the gain each was given in the open: a rumble runs for
    /// seconds, so one that began while the camera stood in the street is turned down and
    /// muffled when it gets into the bus partway through, rather than staying loud.
    rolling: Vec<(omsi_audio::VoiceId, f32, f32)>,
    /// Whether the camera was in a vehicle last frame (`rolling` is only re-set on a change).
    was_inside: bool,
    /// For `OMSI_THUNDER_LOG`.
    log_t: f32,
    struck: u32,
}

impl Default for Thunder {
    fn default() -> Self {
        Thunder {
            strength: 0.0,
            // (not at once: a storm switched on gives its first flash within a few seconds)
            wait: 2.0,
            rng: 0x1234_5678_9ABC_DEF1,
            strikes: Vec::new(),
            claps: vec![None; 3 * CLAP_VARIANTS],
            rolling: Vec::new(),
            was_inside: false,
            log_t: 0.0,
            struck: 0,
        }
    }
}

/// What a clap is heard at: quieter and duller inside a vehicle, as the rain is (its own
/// `regen.wav` on the roof takes over there).
fn heard_at(open_gain: f32, inside: bool) -> (f32, f32) {
    if inside {
        (open_gain * 0.35, 500.0)
    } else {
        (open_gain, 0.0)
    }
}

/// One stroke of lightning.
#[derive(Clone)]
struct Strike {
    /// Towards it from the camera (unit).
    dir: Vec3,
    /// How far away it struck (m).
    distance: f32,
    /// Seconds since it struck.
    age: f32,
    /// The return strokes: when each fired (s) and how bright it was. A flash is not one
    /// stroke but several down the same channel within a tenth of a second, which is what
    /// makes lightning flicker rather than flash once.
    strokes: [(f32, f32); STROKES],
    /// The last stroke's moment: the light is over a little after it.
    last: f32,
    /// What is left of the flash at this distance (the near ones light the street, the far
    /// ones only the sky).
    reach: f32,
    /// The channel drawn on the sky: points as directions from the camera, `w` the
    /// thickness in radians, negative on the first point of a branch. Empty for a strike
    /// behind the cloud, which only lights it from within.
    bolt: Vec<[f32; 4]>,
    /// The cone the channel lies in: its centre (unit) and the cosine of its half angle.
    cone: ([f32; 3], f32),
    /// Its thunder has been played.
    heard: bool,
}

impl Thunder {
    /// One frame of `dt` weather seconds: the storm as the weather or the switch has it,
    /// any strike it makes, and the thunder of the strikes whose sound has arrived.
    ///
    /// `inside` is whether the camera sits in a vehicle, which muffles the clap as it
    /// muffles the rain. Strikes are placed around the camera, so the storm goes where the
    /// bus goes - there is no map of cells to keep, and nothing of it can be seen from
    /// anywhere else.
    fn step(
        &mut self,
        dt: f32,
        weather: Option<&omsi_content::weather::Weather>,
        engine: Option<&AudioEngine>,
        inside: bool,
    ) {
        let strength = storm_strength(weather, omsi_cfg::flags::OMSI_THUNDER.var());
        self.advance(dt, strength, engine, inside);
    }

    /// `step` with the storm's strength already worked out.
    fn advance(&mut self, dt: f32, strength: f32, engine: Option<&AudioEngine>, inside: bool) {
        self.strength = strength;
        for s in &mut self.strikes {
            s.age += dt;
        }
        if self.strength > 0.0 {
            self.wait -= dt;
            if self.wait <= 0.0 {
                self.wait = self.next_wait();
                let s = self.strike();
                self.strikes.push(s);
                self.struck += 1;
            }
        }
        // the thunder of every strike whose sound has had time to get here
        for i in 0..self.strikes.len() {
            if self.strikes[i].heard || self.strikes[i].age < self.strikes[i].distance / SOUND_SPEED {
                continue;
            }
            self.strikes[i].heard = true;
            let (distance, roll) = (self.strikes[i].distance, self.random());
            if let Some(engine) = engine.filter(|e| e.enabled) {
                self.play(engine, distance, roll, inside);
            }
        }
        // (a strike is let go once its light has died and its clap has been heard)
        self.strikes.retain(|s| !s.heard || s.age < s.last + 0.6);
        self.follow_rolling(engine, inside);
        if omsi_cfg::flags::OMSI_THUNDER_LOG.is_set() {
            self.log_t += dt;
            if self.log_t >= 10.0 {
                self.log_t = 0.0;
                let flash = self.light().map(|l| l.strength).unwrap_or(0.0);
                log::info!(
                    "thunder: storm {:.2}, {} strikes so far, {} in the air, next in {:.1} s, flash now {flash:.3}",
                    self.strength,
                    self.struck,
                    self.strikes.len(),
                    self.wait.max(0.0)
                );
            }
        }
    }

    /// The brightest flash of the moment and its channel where one is drawn, as the picture
    /// takes it. None while nothing is lit, which is nearly always.
    fn light(&self) -> Option<omsi_render::Lightning> {
        let mut best: Option<(f32, &Strike)> = None;
        for s in &self.strikes {
            let f = s.flash();
            if f > 0.0 && best.as_ref().map(|(b, _)| f > *b).unwrap_or(true) {
                best = Some((f, s));
            }
        }
        let (strength, s) = best?;
        Some(omsi_render::Lightning {
            dir: s.dir,
            strength,
            // (the channel only while a stroke is actually burning: between two strokes it
            // is dark, and a channel left standing in a dark sky read as a painted crack)
            bolt: if strength > 0.12 * s.reach { s.bolt.clone() } else { Vec::new() },
            cone: s.cone,
        })
    }

    /// Seconds to the next strike: a storm overhead gives one every few seconds, a distant
    /// one every half minute, each waited out with a good spread so they do not come to a
    /// beat.
    fn next_wait(&mut self) -> f32 {
        let mean = 3.0 + 40.0 * (1.0 - self.strength).powi(2);
        mean * (0.35 + 1.3 * self.random())
    }

    /// A strike around the camera.
    fn strike(&mut self) -> Strike {
        // How far off: log-uniform between the near edge and the far one, drawn towards the
        // near edge as the storm comes over. A weak storm is all distance.
        let close = NEAR + (1.0 - self.strength).powf(1.5) * 6000.0;
        let u = self.random().powf(1.0 + 2.0 * self.strength);
        let distance = close * (FAR / close).powf(u);
        let azimuth = self.random() * std::f32::consts::TAU;
        let (sa, ca) = azimuth.sin_cos();
        // the cloud base it comes out of, and where it reaches the ground
        let base = 700.0 + self.random() * 1800.0;
        let strokes = self.strokes();
        let last = strokes.iter().map(|s| s.0).fold(0.0, f32::max);
        // Behind the cloud more often than not: then there is no channel to see, only the
        // whole cloud base lighting up from within - which is what most lightning looks
        // like, and what keeps a drawn bolt from becoming wallpaper.
        let drawn = distance < 9000.0 && self.random() < 0.55;
        let ground = Vec3::new(ca * distance, sa * distance, 0.0);
        let bolt = if drawn { self.channel(ground, base) } else { Vec::new() };
        Strike {
            dir: (ground + Vec3::Z * base * 0.55).normalize_or_zero(),
            distance,
            age: 0.0,
            strokes,
            last,
            reach: (2500.0 / (distance + 2500.0)).powf(0.8),
            bolt,
            cone: ([0.0, 0.0, 1.0], 2.0),
            heard: false,
        }
        .to_directions()
    }

    /// The return strokes of one flash: the leader, and one to three more down the same
    /// channel within a tenth of a second each.
    fn strokes(&mut self) -> [(f32, f32); STROKES] {
        let mut out = [(0.0, 0.0); STROKES];
        out[0] = (0.0, 1.0);
        let n = 1 + (self.random() * (STROKES as f32 - 0.01)) as usize;
        let mut t = 0.0;
        for i in 1..n {
            t += 0.03 + self.random() * 0.09;
            out[i] = (t, 0.45 + self.random() * 0.5);
        }
        out
    }

    /// The channel from the cloud base at `ground` (relative to the camera, level) down to
    /// the ground, with one or two branches: world points, turned into directions by `at`.
    fn channel(&mut self, ground: Vec3, base: f32) -> Vec<[f32; 4]> {
        const STEPS: usize = 11;
        let width = 2.5; // the luminous channel, metres across
        let mut out: Vec<[f32; 4]> = Vec::with_capacity(BOLT_POINTS);
        // the main channel, walked down from the cloud with a lateral stagger and a drift
        let drift = Vec3::new(self.random() - 0.5, self.random() - 0.5, 0.0) * base * 0.25;
        let mut p = ground + Vec3::Z * base;
        let mut trunk: Vec<Vec3> = vec![p];
        for i in 0..STEPS {
            let f = (i + 1) as f32 / STEPS as f32;
            let jitter = Vec3::new(self.random() - 0.5, self.random() - 0.5, 0.0) * base * 0.07;
            p = ground + drift * f + Vec3::Z * base * (1.0 - f) + jitter * (1.0 - f * 0.5);
            trunk.push(p);
        }
        for q in &trunk {
            out.push(point(*q, width));
        }
        // one or two branches, forking partway down and ending in mid-air
        let branches = 1 + (self.random() * 1.99) as usize;
        for _ in 0..branches {
            let k = 2 + (self.random() * (STEPS as f32 - 4.0)) as usize;
            let from = trunk[k.min(trunk.len() - 2)];
            let legs = 2 + (self.random() * 2.99) as usize;
            if out.len() + legs + 1 > BOLT_POINTS {
                break;
            }
            let away = Vec3::new(self.random() - 0.5, self.random() - 0.5, 0.0).normalize_or_zero();
            let mut q = from;
            // (the first point of a branch is marked by a negative thickness: no segment is
            // drawn from the end of what came before into it)
            out.push(point(q, -width * 0.5));
            for _ in 0..legs {
                let reach = base * (0.06 + self.random() * 0.1);
                q += away * reach + Vec3::Z * -reach * (0.7 + self.random() * 0.8)
                    + Vec3::new(self.random() - 0.5, self.random() - 0.5, 0.0) * reach * 0.5;
                out.push(point(q, width * 0.5));
            }
        }
        out
    }

    /// The clap of a strike `distance` away, `roll` picking between the claps of its band.
    fn play(&mut self, engine: &AudioEngine, distance: f32, roll: f32, inside: bool) {
        let band = clap_band(distance);
        let i = band * CLAP_VARIANTS + ((roll * CLAP_VARIANTS as f32) as usize).min(CLAP_VARIANTS - 1);
        if self.claps[i].is_none() {
            // (made at the first strike that needs it, not at startup: a session without a
            // storm makes none at all)
            self.claps[i] = Some(Arc::new(clap(band, 0x9E37_79B9_7F4A_7C15 ^ i as u64 * 0x2545_F491)));
        }
        let Some(clip) = self.claps[i].clone() else { return };
        // It is not heard from a point: the clap rolls off every wall in the street, so it
        // comes from everywhere at once.
        let open = (2000.0 / (distance + 2000.0)).powf(0.6) * (0.7 + 0.3 * roll);
        let pitch = 0.92 + 0.16 * roll;
        let (gain, lowpass_hz) = heard_at(open, inside);
        let id = engine.play(
            clip,
            VoiceParams { gain, pitch, looping: false, position: None, doppler: false, range: 1.0, lowpass_hz, important: true },
        );
        self.rolling.push((id, open, pitch));
    }

    /// The claps still rolling, heard as the camera now stands: one that began in the street
    /// is turned down and muffled when the camera gets into a bus partway through its roll.
    fn follow_rolling(&mut self, engine: Option<&AudioEngine>, inside: bool) {
        let Some(engine) = engine.filter(|e| e.enabled) else {
            self.rolling.clear();
            return;
        };
        let changed = inside != self.was_inside;
        self.was_inside = inside;
        self.rolling.retain(|&(id, open, pitch)| {
            if !engine.is_playing(id) {
                return false;
            }
            if changed {
                let (gain, lowpass_hz) = heard_at(open, inside);
                engine.set_params(id, VoiceParams { gain, pitch, looping: false, position: None, doppler: false, range: 1.0, lowpass_hz, important: true });
            }
            true
        });
    }

    /// A number in 0..1.
    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        ((self.rng >> 40) as f32) / 16_777_216.0
    }
}

impl Strike {
    /// The channel's points turned from places relative to the camera into the directions
    /// and angular thicknesses the shader draws, and the cone they lie in.
    fn to_directions(mut self) -> Strike {
        for p in &mut self.bolt {
            let v = Vec3::new(p[0], p[1], p[2]);
            let d = v.length().max(1.0);
            let dir = v / d;
            // the thickness in radians: a channel of `w` metres at `d` metres off
            let w = p[3] / d;
            *p = [dir.x, dir.y, dir.z, w];
        }
        self.cone = cone_of(&self.bolt);
        self
    }

    /// How bright the flash is now: the return strokes still burning, dimmed by the
    /// distance. Each stroke rises in a few thousandths of a second and falls away in a
    /// twentieth, which is why lightning reads as a flicker and not a lamp.
    fn flash(&self) -> f32 {
        const RISE: f32 = 0.004;
        const FALL: f32 = 0.045;
        let mut sum = 0.0;
        for &(t, a) in &self.strokes {
            if a <= 0.0 {
                continue;
            }
            let u = self.age - t;
            if u < 0.0 {
                continue;
            }
            sum += a * if u < RISE { u / RISE } else { (-(u - RISE) / FALL).exp() };
        }
        (sum * self.reach).min(1.6)
    }
}

/// A channel point: the place relative to the camera, and the channel's width in metres
/// (negative to start a branch), turned into a direction and a thickness by `Strike::at`.
fn point(p: Vec3, width: f32) -> [f32; 4] {
    [p.x, p.y, p.z, width]
}

/// The cone a channel lies in: its centre (unit) and the cosine of its half angle, widened
/// for the glow around it. `w` above 1 where there is no channel, so nothing is drawn.
fn cone_of(bolt: &[[f32; 4]]) -> ([f32; 3], f32) {
    if bolt.is_empty() {
        return ([0.0, 0.0, 1.0], 2.0);
    }
    let mut mid = Vec3::ZERO;
    for p in bolt {
        mid += Vec3::new(p[0], p[1], p[2]);
    }
    let mid = mid.normalize_or_zero();
    let mut cos = 1.0f32;
    for p in bolt {
        cos = cos.min(mid.dot(Vec3::new(p[0], p[1], p[2]).normalize_or_zero()));
    }
    // (room for the halo around the channel and for the points' own thickness)
    ([mid.x, mid.y, mid.z], (cos - 0.03).max(-1.0))
}

/// The distance band a clap is made for: a near strike cracks, a far one only rolls.
fn clap_band(distance: f32) -> usize {
    if distance < 1200.0 {
        0
    } else if distance < 4500.0 {
        1
    } else {
        2
    }
}

/// How strong a storm the weather or the switch asks for (0 none, 1 one overhead).
///
/// `switch` is `OMSI_THUNDER`: `auto` (the default) takes it from the weather, `off` has
/// none, `on` or a number 0..1 lays one over whatever the weather says.
fn storm_strength(weather: Option<&omsi_content::weather::Weather>, switch: Option<&str>) -> f32 {
    match switch {
        Some("off") | Some("0") | Some("false") => return 0.0,
        Some("on") | Some("true") => return 1.0,
        Some(s) => {
            if let Ok(v) = s.parse::<f32>() {
                return v.clamp(0.0, 1.0);
            }
        }
        None => {}
    }
    let Some(w) = weather else { return 0.0 };
    if w.thunder {
        return 1.0;
    }
    // A heavy shower is a thunderstorm more often than not, and a weather file has no way
    // to say so: a distant storm, a flash now and then.
    let rate = if w.precip.first().copied().unwrap_or(0.0) == 1.0 { w.precip.get(1).copied().unwrap_or(0.0) / 255.0 } else { 0.0 };
    if rate >= 0.55 && w.temp.0 > 3.0 {
        ((rate - 0.55) / 0.45 * 0.35).clamp(0.0, 0.35)
    } else {
        0.0
    }
}

/// The clap of a strike in distance band `band`, made from filtered noise.
///
/// Thunder is the air along the whole channel blowing apart at once: the near end reaches
/// the ear first and the far end a second or more later, so one strike is heard as a crack
/// followed by a roll. It is built that way - a burst for the near end and a handful more
/// spread out behind it - and run through a low pass, since the air carries the high notes
/// only a short way: a strike overhead cracks, one eight kilometres off is a rumble with
/// nothing above a few hundred hertz left in it.
fn clap(band: usize, seed: u64) -> Clip {
    let (secs, attack, tau, cutoff) = match band {
        0 => (2.6f32, 0.004f32, 0.55f32, 3200.0f32),
        1 => (4.6, 0.09, 1.5, 950.0),
        _ => (7.2, 0.38, 2.8, 330.0),
    };
    let n = (secs * CLAP_RATE as f32) as usize;
    let mut rng = seed | 1;
    let mut rand = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        ((rng >> 40) as f32) / 16_777_216.0
    };
    // the bursts: the first the near end of the channel, then the rest of it rolling in
    let mut bursts: Vec<(f32, f32, f32)> = vec![(0.0, 1.0, tau)];
    for i in 0..(2 + band * 2) {
        let t = 0.08 + rand() * secs * 0.75;
        let a = (0.55 - 0.07 * i as f32).max(0.15) * (0.5 + rand());
        bursts.push((t, a, tau * (1.0 + 2.0 * rand())));
    }
    // two one-pole low passes in series, their corner falling as the clap rolls out: the
    // last of it has come the farthest through the air and has least left of its edge
    let mut lp1 = 0.0f32;
    let mut lp2 = 0.0f32;
    let mut out = vec![0.0f32; n];
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / CLAP_RATE as f32;
        let mut env = 0.0;
        for &(bt, a, btau) in &bursts {
            let u = t - bt;
            if u < 0.0 {
                continue;
            }
            // (the first burst is the one that cracks: the later ones swell instead)
            let rise = if bt == 0.0 { attack } else { attack.max(0.06) };
            env += a * if u < rise { u / rise } else { (-(u - rise) / btau).exp() };
        }
        let noise = rand() * 2.0 - 1.0;
        let hz = cutoff * (0.25 + 0.75 * (-t / (secs * 0.4)).exp());
        let k = (1.0 - (-std::f32::consts::TAU * hz / CLAP_RATE as f32).exp()).clamp(0.001, 1.0);
        lp1 += (noise - lp1) * k;
        lp2 += (lp1 - lp2) * k;
        *s = lp2 * env;
    }
    // the corner takes most of the noise's level with it, so the clap is scaled by what it
    // actually came out at rather than by what it was built from
    let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-6);
    let scale = 0.85 / peak;
    Clip {
        sample_rate: CLAP_RATE,
        channels: 1,
        samples: out.iter().map(|v| (v * scale * 32767.0).clamp(-32768.0, 32767.0) as i16).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weather(thunder: bool, precip: Vec<f32>, temp: f32) -> omsi_content::weather::Weather {
        omsi_content::weather::Weather { thunder, precip, temp: (temp, 8.0), ..Default::default() }
    }

    /// The switch wins over the weather, and the weather drives it on its own.
    #[test]
    fn the_storm_comes_from_the_switch_or_the_weather() {
        let storm = weather(true, vec![1.0, 200.0], 14.0);
        let dry = weather(false, vec![0.0, 0.0], 14.0);
        let shower = weather(false, vec![1.0, 230.0], 14.0);
        let sleet = weather(false, vec![1.0, 230.0], 1.0);
        // `auto` (and an unreadable value) takes the weather
        for auto in [Some("auto"), Some("yes please")] {
            assert_eq!(storm_strength(Some(&storm), auto), 1.0);
            assert_eq!(storm_strength(Some(&dry), auto), 0.0);
            assert_eq!(storm_strength(None, auto), 0.0);
            // a heavy shower makes a distant storm, a cold one none (that is sleet)
            let s = storm_strength(Some(&shower), auto);
            assert!(s > 0.1 && s < 0.4, "{s}");
            assert_eq!(storm_strength(Some(&sleet), auto), 0.0);
        }
        // the switch wins either way
        assert_eq!(storm_strength(Some(&storm), Some("off")), 0.0);
        assert_eq!(storm_strength(Some(&dry), Some("on")), 1.0);
        assert_eq!(storm_strength(Some(&dry), Some("0.5")), 0.5);
        assert_eq!(storm_strength(Some(&storm), Some("2")), 1.0);
    }

    /// A flash flickers: it rises in thousandths of a second, dies away in a twentieth, and
    /// is over well within a second.
    #[test]
    fn a_flash_rises_fast_and_dies_away() {
        let mut t = Thunder::default();
        let s = t.strike();
        let at = |age: f32| Strike { age, ..s.clone() }.flash();
        // dark before, up within a few thousandths, and all but gone a third of a second
        // after the last stroke
        assert_eq!(at(-0.001), 0.0);
        assert!(at(0.0) < at(0.004) && at(0.004) > 0.0);
        assert!(at(s.last + 0.35) < at(0.004) * 0.05, "{} vs {}", at(s.last + 0.35), at(0.004));
        // the strokes all fall within a few tenths of the leader
        assert!(s.last < 0.4, "{}", s.last);
    }

    /// A storm overhead strikes often and near; a distant one seldom and far.
    #[test]
    fn a_near_storm_strikes_more_often_than_a_far_one() {
        let mut near = Thunder { strength: 1.0, ..Default::default() };
        let mut far = Thunder { strength: 0.15, ..Default::default() };
        let (mut wn, mut wf, mut dn, mut df) = (0.0, 0.0, 0.0, 0.0);
        for _ in 0..200 {
            wn += near.next_wait();
            wf += far.next_wait();
            dn += near.strike().distance;
            df += far.strike().distance;
        }
        assert!(wn / 200.0 < 8.0, "near waits {}", wn / 200.0);
        assert!(wf / 200.0 > 20.0, "far waits {}", wf / 200.0);
        assert!(dn / 200.0 < df / 200.0 * 0.6, "near {} far {}", dn / 200.0, df / 200.0);
        // and every strike stays inside the bounds the sky is drawn for
        for _ in 0..200 {
            let d = near.strike().distance;
            assert!((NEAR..=FAR).contains(&d), "{d}");
        }
    }

    /// The channel is a run of unit directions with one or two branches, and it fits the
    /// shader's array.
    #[test]
    fn the_channel_fits_the_shader_and_lies_in_its_cone() {
        let mut t = Thunder { strength: 1.0, ..Default::default() };
        let mut drawn = 0;
        for _ in 0..300 {
            let s = t.strike();
            if s.bolt.is_empty() {
                continue;
            }
            drawn += 1;
            assert!(s.bolt.len() <= BOLT_POINTS, "{} points", s.bolt.len());
            // no point may carry a zero thickness: the shader reads that as the end
            assert!(s.bolt.iter().all(|p| p[3] != 0.0));
            let mid = Vec3::new(s.cone.0[0], s.cone.0[1], s.cone.0[2]);
            for p in &s.bolt {
                let d = Vec3::new(p[0], p[1], p[2]);
                assert!((d.length() - 1.0).abs() < 1e-4, "{d:?}");
                assert!(mid.dot(d) >= s.cone.1, "outside the cone");
            }
            // a branch starts somewhere in it, marked by a negative thickness
            assert!(s.bolt.iter().skip(1).any(|p| p[3] < 0.0));
        }
        // most strikes are behind the cloud, but a fair share are drawn
        assert!(drawn > 60 && drawn < 260, "{drawn} drawn of 300");
    }

    /// A clap is a crack that rolls away: it peaks early, lasts seconds, and a far one is
    /// longer and duller than a near one.
    #[test]
    fn a_clap_cracks_and_then_rolls() {
        let near = clap(0, 1);
        let far = clap(2, 1);
        assert_eq!((near.channels, near.sample_rate), (1, CLAP_RATE));
        // a near clap is over in a few seconds, a far one rolls for the best part of ten
        let secs = |c: &Clip| c.frames() as f32 / c.sample_rate as f32;
        assert!((secs(&near) - 2.6).abs() < 0.1, "{}", secs(&near));
        assert!(secs(&far) > 6.5, "{}", secs(&far));
        // both are scaled to the same peak, and neither clips
        for c in [&near, &far] {
            let peak = c.samples.iter().map(|s| s.unsigned_abs() as u32).max().unwrap_or(0);
            assert!((27_000..=28_500).contains(&peak), "peak {peak}");
        }
        // the near one cracks: its loudest tenth of a second is at the very start, while
        // the far one builds
        let loud = |c: &Clip, from: f32, to: f32| {
            let (a, b) = ((from * c.sample_rate as f32) as usize, (to * c.sample_rate as f32) as usize);
            c.samples[a..b.min(c.samples.len())].iter().map(|s| (*s as f32 / 32768.0).abs()).fold(0.0, f32::max)
        };
        assert!(loud(&near, 0.0, 0.1) > 0.7, "{}", loud(&near, 0.0, 0.1));
        assert!(loud(&far, 0.0, 0.1) < 0.35, "{}", loud(&far, 0.0, 0.1));
        // and the far one keeps going long after the near one has finished
        assert!(loud(&far, 4.0, 6.0) > 0.02, "{}", loud(&far, 4.0, 6.0));
    }

    /// The clap waits for the sound to travel: a flash four kilometres off is heard twelve
    /// seconds later, and the strike is kept until then.
    #[test]
    fn the_thunder_follows_the_flash_by_the_distance() {
        let mut t = Thunder::default();
        // the first strike comes within a few seconds of a storm overhead
        let mut waited = 0.0;
        while t.strikes.is_empty() && waited < 30.0 {
            t.advance(0.1, 1.0, None, false);
            waited += 0.1;
        }
        assert!(!t.strikes.is_empty(), "no strike in {waited} s");
        let d = t.strikes[0].distance;
        let due = d / SOUND_SPEED;
        // nothing is heard before its sound could have got here (the clock is moved on in
        // one step, so no second strike can slip in)
        t.advance((due - 0.5).max(0.0), 0.0, None, false);
        assert!(!t.strikes.iter().any(|s| s.heard), "heard after {:.1} of {due:.1} s", due - 0.5);
        // and it is heard once it has; a far one is still waiting to be heard, so it is
        // still kept, while a near one may have been let go already
        t.advance(1.0, 0.0, None, false);
        assert!(t.strikes.iter().all(|s| s.heard), "a strike still unheard past {due:.1} s");
        assert_eq!(t.struck, 1);
    }

    /// Nothing is lit, drawn or kept while the storm is off.
    #[test]
    fn a_switched_off_storm_does_nothing() {
        let mut t = Thunder::default();
        for _ in 0..6000 {
            t.advance(0.1, 0.0, None, false);
        }
        assert!(t.strikes.is_empty() && t.struck == 0);
        assert!(t.light().is_none());
    }

    /// A strike lights the picture while it burns and goes out again, and its channel is
    /// only drawn while a stroke is actually burning.
    #[test]
    fn the_picture_gets_the_flash_only_while_it_burns() {
        let mut t = Thunder::default();
        // (a storm overhead, then switched off, so only the one strike is in the air)
        while t.strikes.is_empty() {
            t.advance(0.1, 1.0, None, false);
        }
        t.strikes[0].age = 0.004;
        let l = t.light().expect("lit while it burns");
        assert!(l.strength > 0.0 && (l.dir.length() - 1.0).abs() < 1e-4);
        // a drawn channel comes with a cone that holds it; a strike behind the cloud has
        // neither, and is only a glow
        if t.strikes[0].bolt.is_empty() {
            assert!(l.bolt.is_empty() && l.cone.1 > 1.0);
        } else {
            assert!(!l.bolt.is_empty() && l.cone.1 <= 1.0);
        }
        // dark between the strokes and after the last of them
        t.strikes[0].age = t.strikes[0].last + 0.5;
        assert!(t.light().map(|l| l.strength).unwrap_or(0.0) < 0.01);
    }
}
