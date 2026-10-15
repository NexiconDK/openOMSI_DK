// Sky dome: the envir.cfg gradient textures (day / twilight / night), u = azimuth relative
// to the sun, v = elevation (0 zenith … 1 horizon), blended by the sun altitude.
struct Camera {
    view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    // Kept in lockstep with `CameraUniform` in lib.rs.  The sky does not use the
    // floating-origin reconstruction itself, but omitting this member shifts every
    // following field one vec4 earlier: vanilla then reads the light-grid as `sky`
    // weights (usually all zero) and clears/draws a black sky.
    world_origin: vec4<f32>,
    sun_dir: vec4<f32>,
    ambient: vec4<f32>,
    fog: vec4<f32>,
    sun_color: vec4<f32>,
    sky_color: vec4<f32>,
    light_grid: vec4<f32>,
    sky: vec4<f32>,          // x sun azimuth (rad), y day weight, z twilight weight, w night weight
    cam_right: vec4<f32>,
    cam_up: vec4<f32>,
    clouds: vec4<f32>,       // x density 0..1, yz texture offset (wind drift)
    light_view_proj: mat4x4<f32>,
    light_view_proj_far: mat4x4<f32>,
    shadow: vec4<f32>,
    post: vec4<f32>,         // x enhanced, y time
    inside_a: vec4<f32>,
    inside_b: vec4<f32>,
    inside_c: vec4<f32>,
    flags: vec4<f32>,
    light_view_proj_close: mat4x4<f32>,
    wind: vec4<f32>,
    // Enhanced: the street lamps' shadow maps (the tiles under the far map), and the
    // lights they belong to (-1: none)
    lamp_view_proj: array<mat4x4<f32>, 4>,
    lamp_shadow: vec4<f32>,
    tree_wind: vec4<f32>,
    inside2_a: vec4<f32>,
    inside2_b: vec4<f32>,
    inside2_c: vec4<f32>,
    // (the fork) lightning: xyz towards the flash (unit), w how bright it is now (0 between
    // the strokes); the channel's cone (xyz its centre, w the cosine of its half angle,
    // above 1 for no channel) and the channel's points themselves - directions from the
    // camera, w the thickness in radians, negative on a branch's first point, 0 past the end
    lightning: vec4<f32>,
    bolt_cone: vec4<f32>,
    bolt: array<vec4<f32>, 32>,
};

// A hash of a lattice point, from its integer bits.
//
// It was the old `fract(sin(dot(q, k)) * 43758)` trick, and that is what put flat slabs across
// the top of the sky. `cloud_fbm` below walks `q` up by 2.1 an octave from a world coordinate
// that is kilometres wide, and once the argument of the `sin` reaches a few times 1e5 a
// 32-bit float can no longer say where the fractional part of it lands: the hash comes out
// constant over a whole lattice cell, so the noise reads as flat squares with hard edges in a
// grid that turns with the camera - and only where the layer is thin enough to see through,
// which is why it showed as a "seam" in the sky "only when there are clouds": the high thin
// layer is the only thing up there in a clear sky, and it is faint. Integer bits do not lose
// precision with distance.
fn hash2(q: vec2<f32>) -> f32 {
    let x = bitcast<u32>(i32(floor(q.x)));
    let y = bitcast<u32>(i32(floor(q.y)));
    // (integer arithmetic in WGSL wraps, so these multiplications are modular already)
    var h = (x * 0x8da6b343u) ^ (y * 0xd8163841u);
    h = h ^ (h >> 13u);
    h = h * 0x5bd1e995u;
    h = h ^ (h >> 15u);
    return f32(h & 0xffffu) / 65535.0;
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash2(i), hash2(i + vec2<f32>(1.0, 0.0)), u.x), mix(hash2(i + vec2<f32>(0.0, 1.0)), hash2(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

// Cloud density field: the map's cloud texture as the large shape, fractal noise as the
// small one, two layers at different heights drifting at different speeds.
fn cloud_fbm(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        v = v + a * vnoise(q);
        q = q * 2.1 + vec2<f32>(17.0, 9.0);
        a = a * 0.5;
    }
    return v;
}
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var t_day: texture_2d<f32>;
@group(1) @binding(1) var t_twilight: texture_2d<f32>;
@group(1) @binding(2) var t_night: texture_2d<f32>;
@group(1) @binding(3) var s_sky: sampler;
@group(1) @binding(4) var t_clouds: texture_2d<f32>;
@group(1) @binding(5) var s_repeat: sampler;

/// How far the cloud field reaches before it repeats (m): a cumulus is then half a
/// kilometre to two across.
const CLOUD_FIELD_TILE: f32 = 14000.0;

// The ground point under the sky at distance `t` along the view ray `d`, in the clouds' own
// frame (world coordinates modulo 70 km, lib.rs `CLOUD_ORIGIN_PERIOD`), so that the cloud
// field stays where it is when the floating render origin moves on.
// Where the clouds are seen from, relative to the camera (the enhanced sky cube's own eye
// while it is drawn; 0 everywhere else).
var<private> eye_off: vec3<f32> = vec3<f32>(0.0);

fn cloud_ground(d: vec3<f32>, t: f32) -> vec2<f32> {
    return camera.cam_pos.xy + eye_off.xy + camera.world_origin.zw + d.xy * t;
}

// The cloud cover over ground point p (x, 0..1, its edges frayed by the billows), how tall
// the cloud there grows (y, 0..1) and the cover without the billows (z: the enhanced sky
// raises its rounded tops on that, the billows would stand on them as spikes). From the
// cloud field: its equalised shape (G) cut at 1 - the cover, the weather's own picture (R)
// nudging where the clouds gather, billows (B) fraying the edges near by.
fn cloud_cover_at(p: vec2<f32>, lod: f32) -> vec3<f32> {
    let uv = p / CLOUD_FIELD_TILE + camera.clouds.yz * (2500.0 / CLOUD_FIELD_TILE);
    let t = textureSampleLevel(t_clouds, s_repeat, uv, lod);
    let thr = 1.0 - clamp(camera.clouds.x, 0.0, 1.0);
    let smooth_shape = t.g + (t.r - 0.5) * 0.15;
    var shape = smooth_shape;
    // the billows only where they are big enough to see
    let fray = 1.0 - smoothstep(2.0, 5.0, lod);
    if (fray > 0.0) {
        let detail = textureSampleLevel(t_clouds, s_repeat, uv * 3.7 + vec2<f32>(0.31, 0.73), lod + 1.9).b;
        shape = shape + (detail - 0.5) * 0.2 * fray;
    }
    return vec3<f32>(clamp((shape - thr) / 0.14, 0.0, 1.0), t.a, clamp((smooth_shape - thr) / 0.14, 0.0, 1.0));
}

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) dir: vec3<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VsOut {
    var out: VsOut;
    // the dome rides with the camera; pushed to the far plane (0 with reversed Z)
    let wp = camera.cam_pos.xyz + pos * 4000.0;
    var clip = camera.view_proj * vec4<f32>(wp, 1.0);
    clip.z = clip.w * 0.000001;
    out.clip = clip;
    out.dir = pos;
    return out;
}

// The classic sky's weather (lib.rs `VanillaSky`): the cloud type's own texture and
// cloud: x height H (m over the map's zero), y the type's size (m a tile; 0: no clouds),
// zw the wind's drift (m); haze: x the weather's fog range (m), y the visibility (m),
// z 1 for an overcast type, w the render origin's height.
struct VanillaSkyUniform {
    cloud: vec4<f32>,
    haze: vec4<f32>,
    // (the fork) xyz towards the moon (unit), w how much of its disc is lit
    moon: vec4<f32>,
};
@group(1) @binding(9) var t_vclouds: texture_2d<f32>;
@group(1) @binding(10) var<uniform> vsky: VanillaSkyUniform;

// The v of the sky textures towards elevation `e` (rad), as Omsi.exe's dome
// (helper\skybox.x) has it: rings at the horizon, at 45 degrees and at the zenith with v
// 0.9833, 0.4868 and 0.0236, flat bands between them along which v runs linearly - along
// the chord, not with the angle - and the horizon's v all the way down below it.
fn dome_v(e: f32) -> f32 {
    if (e <= 0.0) {
        return 0.9833;
    }
    let upper = e > 0.7853982;
    let e0 = select(0.0, 0.7853982, upper);
    let e1 = select(0.7853982, 1.5707963, upper);
    let v0 = select(0.9833, 0.4868, upper);
    let v1 = select(0.4868, 0.0236, upper);
    // (where the ray at e meets the chord from the ring at e0 to the one at e1)
    let t = sin(e - e0) / ((sin(e1) - sin(e0)) * cos(e) - (cos(e1) - cos(e0)) * sin(e));
    return mix(v0, v1, clamp(t, 0.0, 1.0));
}

// The haze Omsi.exe lays over its sky (0x5d8e98): the dome again with Texture\nebel.tga,
// white with an alpha running linearly over its 32 rows from 0 at the zenith to 1 at the
// horizon, in the fog colour, its alpha moved by the fog range f and the cloud height H:
// down by 1 - 4 asin(H/f)/pi where f > H sqrt 2 (no haze at all under a clear sky, H 0),
// up by 1 - 4 acos(H/f)/pi below that, and the whole sky fog where f <= H.
fn vanilla_haze(v: f32) -> f32 {
    let f = vsky.haze.x;
    let h = vsky.cloud.x;
    if (f <= 0.0) {
        return 0.0;
    }
    if (f <= h) {
        return 1.0;
    }
    let tex_a = clamp((v * 32.0 - 0.5) / 31.0, 0.0, 1.0);
    let q = h / f;
    if (q < 0.70710678) {
        return clamp(tex_a - (1.0 - 4.0 * asin(q) / 3.14159265), 0.0, 1.0);
    }
    return clamp(tex_a + (1.0 - 4.0 * acos(q) / 3.14159265), 0.0, 1.0);
}

// The classic cloud layer towards d: rgb (in the sky's own terms, `fog` its fog colour) and
// how much it covers. Omsi.exe (0x754e44) draws helper\clouds.o3d, a flat cone over the
// camera - its apex H over the map's zero, its rim on the zero 9.363 H away - with the cloud
// type's texture laid on the ground plan, one tile every `size` metres, moved by the wind;
// its colour is the texture times the fog colour, its alpha the texture's (an `ovc` deck is
// opaque), and the fixed-function fog takes it into the fog colour by its depth in the
// view over the visibility.
fn vanilla_clouds(d: vec3<f32>, fog: vec3<f32>, tex: vec4<f32>, r: f32, encoded: bool) -> vec4<f32> {
    let hl = max(length(d.xy), 1e-4);
    // (the hit point lies along d, r / hl from the camera; its depth along the view's axis)
    let fwd = normalize(cross(camera.cam_up.xyz, camera.cam_right.xyz));
    let depth = r / hl * dot(d, fwd);
    let vis = max(vsky.haze.y, 1.0);
    let k = clamp(1.0 - depth / vis, 0.0, 1.0);
    var rgb = select(tex.rgb, srgb_encode(tex.rgb), encoded) * fog;
    rgb = mix(fog, rgb, k);
    return vec4<f32>(rgb, select(tex.a, 1.0, vsky.haze.z > 0.5));
}

// How far (over the ground plan) the ray d meets the cloud cone, and whether it does.
fn vanilla_cloud_reach(d: vec3<f32>) -> vec2<f32> {
    let h = vsky.cloud.x;
    let h0 = camera.cam_pos.z + vsky.haze.w;
    let hl = max(length(d.xy), 1e-4);
    let rim = 9.363 * h;
    let den = d.z / hl + 1.0 / 9.363;
    if (vsky.cloud.y <= 0.0 || h <= h0 || h <= 0.0) {
        return vec2<f32>(rim, 0.0);
    }
    // (below the rim the cone's skirt runs down under the horizon, behind the ground: its
    // rim stands in for it)
    var r = rim;
    if (den > 0.0) {
        r = min((h - h0) / den, rim);
    }
    return vec2<f32>(r, 1.0);
}

// (the fork) The star sky: how it stands now, and the catalogue of the brightest stars it
// is drawn from (lib.rs `StarSkyUniform`, `crate::stars`).
struct StarSky {
    // the columns of the turn from the ground's frame (x east, y north, z up) into the
    // J2000 equatorial frame the catalogue is in
    east: vec4<f32>,
    north: vec4<f32>,
    up: vec4<f32>,
    // x the seconds the twinkling and the meteors run on, y how bright the stars are
    // drawn, z how often a meteor falls (s), w unused
    params: vec4<f32>,
    // where each band of declination starts, four to a vector
    bands: array<vec4<f32>, 9>,
    // xyz the star's unit vector (equatorial), w its colour and magnitude packed as
    // tint * 8 + (magnitude + 2)
    list: array<vec4<f32>, 518>,
};
@group(1) @binding(11) var<uniform> sky_stars: StarSky;

// (the fork) Bands of declination the catalogue is cut into (stars.rs `STAR_BANDS_N`).
const STAR_BANDS: i32 = 32;

// (the fork) The eight colours the catalogue's temperatures are kept in, from the reddest
// giants (Betelgeuse, 3350 K) to the hottest blue-white (Rigel, 14000 K). Stars are far
// less coloured to the eye than a photograph makes them: at night the colour-blind rods
// do most of the seeing, and only the brightest show any tint at all.
fn star_tint(code: i32) -> vec3<f32> {
    switch code {
        case 0: { return vec3<f32>(1.0, 0.72, 0.48); }
        case 1: { return vec3<f32>(1.0, 0.80, 0.60); }
        case 2: { return vec3<f32>(1.0, 0.87, 0.72); }
        case 3: { return vec3<f32>(1.0, 0.93, 0.84); }
        case 4: { return vec3<f32>(1.0, 0.98, 0.95); }
        case 5: { return vec3<f32>(0.95, 0.96, 1.0); }
        case 6: { return vec3<f32>(0.87, 0.91, 1.0); }
        default: { return vec3<f32>(0.80, 0.86, 1.0); }
    }
}

// (the fork) How hard a star twinkles and in what colour, towards `d`.
//
// Scintillation is the air's doing, not the star's: pockets of air of slightly different
// temperature act as weak lenses drifting across the line of sight, and the light they
// bend arrives a little brighter or fainter many times a second. The more air the ray
// crosses the worse it is, so a star at the zenith is nearly steady while one near the
// horizon - six or seven times the air - flickers hard and visibly in colour, because the
// air bends blue a little more than red and the two arrive out of step. Planets hardly
// twinkle at all, being discs rather than points, and the catalogue holds none.
//
// `seed` keeps every star on its own rhythm; three frequencies from one to twenty hertz
// are enough that no beat is heard in it.
fn star_twinkle(up: f32, seed: f32, t: f32) -> vec2<f32> {
    let airmass = 1.0 / max(up, 0.055);
    let amp = clamp(0.028 * pow(airmass, 1.7), 0.0, 0.55);
    let a = sin(t * 11.3 + seed * 6.283) * 0.5
        + sin(t * 7.1 + seed * 14.7) * 0.3
        + sin(t * 19.7 + seed * 31.1) * 0.2;
    // the colour runs on its own, a little slower, and only tells near the horizon
    let c = sin(t * 5.3 + seed * 21.7) * 0.6 + sin(t * 13.1 + seed * 9.3) * 0.4;
    return vec2<f32>(1.0 + amp * a, amp * c * 0.8);
}

// (the fork) A shooting star towards `d`, or 0.
//
// One falls every `sky_stars.params.z` seconds: which one it is numbers the period, and
// everything about it - where it comes from, which way it goes, how bright and how long it
// burns - is hashed from that number, so it needs nothing kept from frame to frame. It
// burns for a few tenths of a second along a great circle, head first with the trail
// stretched out behind it, and fades as it goes.
fn meteor_light(d: vec3<f32>, t: f32) -> f32 {
    let every = max(sky_stars.params.z, 1.0);
    let k = floor(t / every);
    let u = (t / every - k) * every;
    let h = star_hash4(k);
    // it burns for between a third of a second and a second
    let burn = 0.35 + 0.65 * h.w;
    if (u > burn) {
        return 0.0;
    }
    // where it starts: anywhere in the upper half of the sky, and which way it runs
    let a = h.x * 6.2831853;
    let z = 0.12 + 0.85 * h.y;
    let r = sqrt(max(1.0 - z * z, 0.0));
    let start = vec3<f32>(r * cos(a), r * sin(a), z);
    let b = h.z * 6.2831853;
    let side = normalize(cross(start, vec3<f32>(0.0, 0.0, 1.0)) + vec3<f32>(1e-4, 0.0, 0.0));
    let other = cross(start, side);
    let way = normalize(side * cos(b) + other * sin(b));
    // how far along its path the head has got (10 to 25 degrees in all), and the tail
    let f = u / burn;
    let span = 0.17 + 0.26 * h.x;
    let head = f * span;
    let tail = max(head - span * 0.45, 0.0);
    // the ray's place against the great circle it runs on
    let along = atan2(dot(d, way), dot(d, start));
    let off = asin(clamp(dot(d, normalize(cross(start, way))), -1.0, 1.0));
    if (along < tail - 0.02 || along > head + 0.01) {
        return 0.0;
    }
    // a thin streak, brightest at the head and thinning out along the tail
    let back = clamp((head - along) / max(head - tail, 1e-4), 0.0, 1.0);
    let width = mix(0.0016, 0.0055, back);
    let core = exp(-(off * off) / (width * width));
    let fade = (1.0 - back) * (1.0 - back) * 0.85 + 0.15 * (1.0 - back);
    // it lights up as it comes in and is gone before the period is over
    let life = smoothstep(0.0, 0.1, f) * (1.0 - smoothstep(0.65, 1.0, f));
    return core * fade * life * (2.0 + 6.0 * h.z);
}

// (the fork) Four numbers from one, for the meteors.
fn star_hash4(n: f32) -> vec4<f32> {
    let i = bitcast<u32>(i32(n)) * 0x9e3779b9u;
    var a = i ^ (i >> 15u);
    a = a * 0x85ebca6bu;
    var b = a ^ (a >> 13u);
    b = b * 0xc2b2ae35u;
    var c = b ^ (b >> 16u);
    c = c * 0x27d4eb2fu;
    var e = c ^ (c >> 15u);
    e = e * 0x165667b1u;
    return vec4<f32>(vec4<u32>(a, b, c, e) >> vec4<u32>(8u)) / 16777216.0;
}

// (the fork) The stars towards `d`, catalogue and veil together, in the picture's own
// terms. `shows` is how much of them the sky behind lets through: 1 on a dark country
// night, little under a city's glow, none in the twilight.
//
// Only the stars of the ray's own band of declination are walked - the catalogue is sorted
// by it - so a pixel tests some sixteen of the 518 rather than all of them. The margin
// takes in the next band where the ray lies near an edge, and because the stars are sorted
// the two bands are one unbroken run.
fn star_light(d: vec3<f32>, shows: f32, pixel: f32) -> vec3<f32> {
    if (shows <= 0.0 || d.z < 0.0) {
        return vec3<f32>(0.0);
    }
    let t = sky_stars.params.x;
    // the ray in the frame the catalogue is in
    let s = sky_stars.east.xyz * d.x + sky_stars.north.xyz * d.y + sky_stars.up.xyz * d.z;
    let dec = asin(clamp(s.z, -1.0, 1.0));
    let margin = 0.006;
    let band = f32(STAR_BANDS) / 3.14159265;
    let b0 = clamp(i32(floor((dec - margin + 1.5707963) * band)), 0, STAR_BANDS - 1);
    let b1 = clamp(i32(floor((dec + margin + 1.5707963) * band)), 0, STAR_BANDS - 1);
    let first = i32(sky_stars.bands[b0 / 4][b0 % 4]);
    let last = i32(sky_stars.bands[(b1 + 1) / 4][(b1 + 1) % 4]);
    // a star is a point: it is drawn the size of a pixel, no smaller, or it would flicker
    // as the camera turns rather than twinkle
    let size = clamp(pixel * 0.7, 0.0006, 0.004);
    var col = vec3<f32>(0.0);
    for (var i = first; i < last; i = i + 1) {
        let star = sky_stars.list[i];
        let c = dot(s, star.xyz);
        // (the chord, which stands for the angle this close in)
        let d2 = max(2.0 - 2.0 * c, 0.0);
        if (d2 > size * size * 16.0) {
            continue;
        }
        let code = floor(star.w / 8.0);
        let mag = star.w - code * 8.0 - 2.0;
        // a magnitude is a ratio of light: five of them are a hundred times
        let flux = exp2(-mag * 1.3287712);
        let tw = star_twinkle(d.z, fract(star.w * 0.618), t);
        let spot = exp(-d2 / (size * size));
        var tint = star_tint(i32(code));
        // the colour the air pulls off it, which only tells near the horizon
        tint = clamp(tint + vec3<f32>(tw.y, 0.0, -tw.y) * 0.5, vec3<f32>(0.0), vec3<f32>(1.4));
        col = col + tint * (flux * max(tw.x, 0.0) * spot);
    }
    // the veil of the fainter ones, which no catalogue need hold: a hash over cells of the
    // sky, most of them empty, with the power law of the magnitudes that the real sky has
    col = col + star_veil(s, size, t, d.z);
    // the meteor rides over the lot
    let m = meteor_light(d, t);
    return (col * 0.55 + vec3<f32>(0.95, 0.97, 1.0) * m) * shows * sky_stars.params.y;
}

// (the fork) The fainter stars, which are too many to keep and too faint to know apart: a
// point in some of the cells of a grid laid over the sky, with the same power law of
// brightness the catalogue's own stars follow.
fn star_veil(s: vec3<f32>, size: f32, t: f32, up: f32) -> vec3<f32> {
    let cells = 150.0;
    let g = s * cells;
    let c = vec3<i32>(floor(g));
    let h = star_hash3(c);
    let density = 0.05;
    if (h.x >= density) {
        return vec3<f32>(0.0);
    }
    let p = normalize(vec3<f32>(c) + vec3<f32>(0.5) + (h.yzw - 0.5) * 0.9);
    let d2 = max(2.0 - 2.0 * dot(s, p), 0.0);
    if (d2 > size * size * 9.0) {
        return vec3<f32>(0.0);
    }
    // they run from the fifth magnitude down to the seventh, where the eye gives out
    let mag = 4.6 + 2.4 * pow(h.x / density, 0.6);
    let flux = exp2(-mag * 1.3287712);
    let tw = star_twinkle(up, h.w, t);
    return star_tint(i32(h.z * 7.99)) * (flux * max(tw.x, 0.0) * exp(-d2 / (size * size)));
}

// (the fork) Four numbers from a cell of the sky.
fn star_hash3(c: vec3<i32>) -> vec4<f32> {
    var x = bitcast<u32>(c.x) * 0x8da6b343u;
    var y = bitcast<u32>(c.y) * 0xd8163841u;
    var z = bitcast<u32>(c.z) * 0xcb1ab31fu;
    var h = x ^ y ^ z;
    h = h ^ (h >> 13u);
    h = h * 0x5bd1e995u;
    h = h ^ (h >> 15u);
    let a = h * 0x27d4eb2fu;
    let b = a ^ (a >> 15u);
    let e = b * 0x165667b1u;
    return vec4<f32>(vec4<u32>(h, a, b, e) >> vec4<u32>(8u)) / 16777216.0;
}

// (the fork) The moon's angular radius (rad): a quarter of a degree, as it is seen.
const MOON_R: f32 = 0.00465;

// (the fork) The moon in the classic sky, which has no model of the night to put it in -
// the Enhanced sky draws its own from the atmosphere (sky_enhanced.wgsl `night_sky`).
//
// It is a lit sphere, not a disc with a bite out of it: the point of the sphere seen in
// each direction is worked out, and how much of it is lit is the angle between that point's
// own normal and the sun. That is what makes the terminator a smooth curve rather than a
// straight edge, and it gives the phase for free from where the sun stands.
//
// `dark` is how far into the night the sky has gone (the dome's twilight and night weights):
// the moon is up by day as often as not, and then it is barely a pale mark.
fn vanilla_moon(d: vec3<f32>, dark: f32) -> vec3<f32> {
    let m = vsky.moon.xyz;
    let lit = vsky.moon.w;
    // (below the horizon, or new: nothing to draw)
    if (lit <= 0.0 || m.z < -0.03 || dot(m, m) < 0.5) {
        return vec3<f32>(0.0);
    }
    let cm = dot(d, m);
    if (cm < cos(MOON_R * 9.0)) {
        return vec3<f32>(0.0);
    }
    let s = normalize(camera.sun_dir.xyz);
    let right = normalize(cross(m, vec3<f32>(0.0, 0.0, 1.0)) + vec3<f32>(1e-5, 0.0, 0.0));
    let up = cross(right, m);
    let q = vec2<f32>(dot(d, right), dot(d, up)) / MOON_R;
    let q2 = dot(q, q);
    // the glow the air spreads round it, which is most of what one sees of a bright moon
    let halo = 0.09 * exp(-q2 * 0.06) * dark;
    var face = 0.0;
    if (q2 < 1.2) {
        // the point of the sphere this ray meets, and its normal there
        let nz = sqrt(max(1.0 - q2, 0.0));
        let n = normalize(right * q.x + up * q.y - m * nz);
        // the sun on that point: a soft terminator, and a trace of earthshine on the rest
        // (the dark limb of a crescent, lit by the earth, which the eye does pick up)
        let sun_side = smoothstep(-0.05, 0.1, dot(n, s));
        let rim = 1.0 - smoothstep(0.93, 1.03, sqrt(q2));
        // the seas: the broad grey patches that make it a face and not a lamp
        let sea = smoothstep(0.45, 0.7, cloud_fbm(q * 1.7 + vec2<f32>(3.1, 7.7)));
        face = rim * (sun_side + 0.02 * lit) * (1.0 - 0.33 * sea);
    }
    // a full moon is the brightest thing in a night sky by far; a crescent is a sliver
    return MOON_COLOUR * (face * mix(0.22, 1.0, dark) + halo * lit);
}

// (the fork) The moon's own colour: a touch warm of white (the blue of moonlight is the
// eye's, not the moon's - at that little light the rods see no colour at all).
const MOON_COLOUR = vec3<f32>(1.04, 1.0, 0.93);

// (the fork) Lightning runs at some 25 000 K: a touch blue of white.
const LIGHTNING_TINT = vec3<f32>(0.82, 0.88, 1.0);

// (the fork) How much of a lightning channel the ray `d` passes through: 1 along its core,
// falling away into the halo around it.
//
// The channel is held as a run of directions from the camera (see `omsi_render::Lightning`),
// so this is the distance from the ray to the nearest of its segments, measured in the
// channel's own thickness. Nothing is walked outside the cone the whole channel lies in,
// which is all but a sliver of the sky: a pixel there pays one dot product. Nothing is
// walked at all while no channel burns, which is nearly always.
fn bolt_glow(d: vec3<f32>) -> f32 {
    if (camera.bolt_cone.w > 1.0 || dot(d, camera.bolt_cone.xyz) < camera.bolt_cone.w) {
        return 0.0;
    }
    var near = 1e9;
    for (var i = 1; i < 32; i = i + 1) {
        let b = camera.bolt[i];
        if (b.w == 0.0) {
            break;
        }
        // (a branch's first point: nothing is drawn into it from the end of what came before)
        if (b.w < 0.0) {
            continue;
        }
        let a = camera.bolt[i - 1];
        let ab = b.xyz - a.xyz;
        let t = clamp(dot(d - a.xyz, ab) / max(dot(ab, ab), 1e-12), 0.0, 1.0);
        // (the points are unit directions, so their chord stands for the angle between them
        // to within a part in a thousand over the degree or two a channel spans)
        near = min(near, length(d - (a.xyz + ab * t)) / mix(abs(a.w), b.w, t));
    }
    // the core white hot, with a halo that reaches a dozen times its width
    return exp(-near * near * 0.8) + 0.3 * exp(-near * 0.22);
}

// (the fork) The lightning on the sky: the cloud base lit from within on the side it struck,
// and the channel burning across it where one can be seen.
//
// `cloud` is how much cloud the ray meets. A stroke is inside the cloud, so the cloud lights
// up and the clear sky beside it only a little - which is what makes a flash read as a storm
// overhead rather than as the whole dome being turned up.
//
// In the picture's own exposed terms (see shader.wgsl `lightning_light`): the Vanilla sky
// takes it as it is and clips to white, which is what a flash does, and the Enhanced one
// adds it beside its own pre-exposed terms.
fn lightning_sky(d: vec3<f32>, cloud: f32) -> vec3<f32> {
    let f = camera.lightning.w;
    if (f <= 0.0) {
        return vec3<f32>(0.0);
    }
    let side = max(dot(d, camera.lightning.xyz), 0.0);
    let glow = (0.12 + 1.4 * side * side) * mix(0.3, 1.0, clamp(cloud, 0.0, 1.0));
    return LIGHTNING_TINT * f * (glow + 25.0 * bolt_glow(d));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let d = normalize(in.dir);
    // the dome turned so that the texture's column 0.5036 faces the sun, u growing
    // anticlockwise (from above) away from it (0x5d92a0: the dome rotated by the sun's
    // azimuth)
    let az = atan2(d.x, d.y);
    let u = 0.503645 + (camera.sky.x - az) / 6.2831853;
    let elev = asin(clamp(d.z, -1.0, 1.0));
    let v = dome_v(elev);
    let uv = vec2<f32>(u, v);
    // (Omsi.exe blends two of its pictures by the sun's height, see `Daylight::sky_weights`;
    // the sky textures have no mip levels, so the seam of u behind the sun is not seen)
    let sky = textureSample(t_day, s_sky, uv).rgb * camera.sky.y + textureSample(t_twilight, s_sky, uv).rgb * camera.sky.z + textureSample(t_night, s_sky, uv).rgb * camera.sky.w;
    // the cloud texture on the ground plan where the ray meets the cone, sampled outside
    // any branch (its mip levels by the screen's derivatives)
    let reach = vanilla_cloud_reach(d);
    let hl = max(length(d.xy), 1e-4);
    let ground = camera.cam_pos.xy + camera.world_origin.zw + d.xy / hl * reach.x;
    let size = max(vsky.cloud.y, 1.0);
    let cuv = (ground + vsky.cloud.zw) / size + vec2<f32>(0.5, -0.5) * (vsky.cloud.x / size);
    let ctex = textureSample(t_vclouds, s_repeat, vec2<f32>(cuv.x, cuv.y));
    // Vanilla blends in the 8-bit sRGB terms of Omsi.exe's fixed-function stages,
    // Vanilla+ in linear light
    let encoded = camera.sky_color.w > 0.5;
    let fog = camera.fog.xyz;
    var col = select(sky, srgb_encode(sky), encoded);
    col = mix(col, fog, vanilla_haze(v));
    var cloud = 0.0;
    if (reach.y > 0.5) {
        let c = vanilla_clouds(d, fog, ctex, reach.x, encoded);
        col = mix(col, c.rgb, c.a);
        cloud = c.a;
    }
    var out = col;
    if (encoded) {
        out = srgb_decode(col);
    }
    // (the fork) the moon and the stars, behind whatever cloud lies in front of them, and
    // the lightning: all added in linear light whichever of the two terms the sky itself
    // was blended in
    let dark = clamp(camera.sky.z * 0.5 + camera.sky.w, 0.0, 1.0);
    let clear = 1.0 - clamp(cloud, 0.0, 1.0);
    out = out + vanilla_moon(d, dark) * clear;
    // the stars show as far as the sky behind them is dark: a night sky lets them all
    // through, the dusk none, and the moon's own glow washes the faintest out
    let washed = 1.0 - 0.45 * clamp(vsky.moon.w, 0.0, 1.0) * smoothstep(-0.05, 0.25, vsky.moon.z);
    out = out + star_light(d, clamp(camera.sky.w, 0.0, 1.0) * clear * washed, fwidth(d.z) + 1e-5);
    return vec4<f32>(out + lightning_sky(d, cloud), 1.0);
}
