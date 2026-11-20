#![forbid(unsafe_code)]

#[path = "bake_study/rooms.rs"]
mod rooms;

use bevy::{
    asset::RenderAssetUsages,
    math::{Vec2, Vec3, vec3},
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use latch::room_lighting::{FOYER_LIT, FOYER_NIGHT, SIGNAL_ATLAS, atlas_height};
use latch::study::{
    ATLAS_WIDTH, BoxGeometry, DESK_LEGS, DESK_LIGHT, LAMP_ATLAS, LIGHTMAP_EXPOSURE, MOON_ATLAS,
    MOON_LIGHT, PADDING, SHELF_LEVELS, SURFACES, Surface, TILE_SIZE, shelf_book, shelf_level,
    shelf_side, tile_origin,
};
use std::{
    error::Error,
    f32::consts::TAU,
    fs,
    path::{Path, PathBuf},
    thread,
};

const SAMPLES: u32 = 64;
const GUTTER: u32 = PADDING;
const SIDE: u32 = TILE_SIZE - GUTTER * 2;
const EPSILON: f32 = 0.002;
const FILL: Vec3 = vec3(49.0, 54.0, 58.0);
const PLASTER: Vec3 = vec3(0.115, 0.211, 0.201);
const OAK: Vec3 = vec3(0.30, 0.18, 0.10);
const WALNUT: Vec3 = vec3(0.17, 0.087, 0.045);
const CEILING: Vec3 = vec3(0.048, 0.083, 0.091);

#[derive(Clone, Copy)]
struct Shape {
    min: Vec3,
    max: Vec3,
    albedo: Vec3,
}

impl Shape {
    /// 建立靜態烘焙幾何，albedo 是線性的漫反射比例。
    fn new(geometry: BoxGeometry, albedo: Vec3) -> Self {
        let half = geometry.size * 0.5;
        Self {
            min: geometry.center - half,
            max: geometry.center + half,
            albedo,
        }
    }
}

#[derive(Clone, Copy)]
struct Ray {
    start: Vec3,
    direction: Vec3,
    inv: Vec3,
}

impl Ray {
    /// 由已正規化的世界方向建立射線；平行軸只用於相交除法的安全替代值。
    fn new(start: Vec3, direction: Vec3) -> Self {
        let safe = |v: f32| v.abs().max(1e-8).copysign(v);
        let inv = Vec3::new(safe(direction.x), safe(direction.y), safe(direction.z)).recip();
        Self {
            start,
            direction,
            inv,
        }
    }

    /// 回傳射線進入方盒的交點距離與朝外法線；方盒內部遮住起點時回傳離開的面。
    fn intersect(self, shape: Shape) -> Option<(f32, Vec3)> {
        let a = (shape.min - self.start) * self.inv;
        let b = (shape.max - self.start) * self.inv;
        let near = a.min(b);
        let far = a.max(b);
        let entry = near.max_element();
        let exit = far.min_element();
        if exit < entry.max(EPSILON) {
            return None;
        }
        let (distance, axis_times, sign) = if entry >= EPSILON {
            (entry, near, -1.0)
        } else {
            (exit, far, 1.0)
        };
        let axis = if distance == axis_times.x {
            Vec3::X
        } else if distance == axis_times.y {
            Vec3::Y
        } else {
            Vec3::Z
        };
        Some((distance, axis * self.direction.signum() * sign))
    }
}

#[derive(Clone, Copy)]
struct Lamp {
    position: Vec3,
    color: Vec3,
    lumens: f32,
}

const MOON: Lamp = Lamp {
    position: MOON_LIGHT,
    color: vec3(0.41, 0.61, 1.0),
    lumens: 1650.0,
};
const DESK: Lamp = Lamp {
    position: DESK_LIGHT,
    color: vec3(1.0, 0.56, 0.24),
    lumens: 1550.0,
};
/// 收集固定表面與近似遮擋的長方體；活動門板、椅子和可移動道具由遊戲照明。
fn scene() -> Vec<Shape> {
    let mut shapes: Vec<_> = SURFACES
        .iter()
        .enumerate()
        .map(|(index, surface)| {
            let albedo = match index {
                0 | 8 => OAK,
                7 => CEILING,
                9 => WALNUT,
                _ => PLASTER,
            };
            Shape::new(surface.geometry, albedo)
        })
        .collect();
    shapes.extend(DESK_LEGS.map(|leg| Shape::new(leg, WALNUT)));
    shapes.extend(SHELF_LEVELS.map(|y| Shape::new(shelf_level(y), WALNUT)));
    shapes.extend([0.02, 2.82].map(|z| Shape::new(shelf_side(z), WALNUT)));
    let colors = [
        vec3(0.08, 0.15, 0.11),
        vec3(0.24, 0.07, 0.032),
        vec3(0.24, 0.17, 0.06),
        vec3(0.053, 0.105, 0.15),
    ];
    for tier in 0..3 {
        for i in 0..13 {
            shapes.push(Shape::new(shelf_book(tier, i), colors[i % 4]));
        }
    }
    for (at, size, albedo) in [
        (
            [-0.9, 2.16, -4.37],
            [2.37, 1.88, 0.04],
            vec3(0.022, 0.058, 0.12),
        ),
        ([-0.9, 1.12, -4.17], [2.9, 0.12, 0.42], WALNUT),
        ([-3.45, 0.28, -3.55], [1.12, 0.52, 0.62], WALNUT),
        (
            [3.68, 0.72, -1.65],
            [0.94, 1.43, 1.10],
            vec3(0.075, 0.085, 0.088),
        ),
        (
            [0.0, 0.045, 0.15],
            [4.2, 0.033, 3.6],
            vec3(0.21, 0.065, 0.031),
        ),
        (
            [-0.65, 1.023, -2.56],
            [0.47, 0.015, 0.34],
            vec3(0.55, 0.43, 0.29),
        ),
    ] {
        shapes.push(Shape::new(BoxGeometry::new(at, size), albedo));
    }
    shapes
}

/// 找出射線在給定公尺距離內的第一個交點。
fn closest(scene: &[Shape], ray: Ray, limit: f32) -> Option<(Shape, f32, Vec3)> {
    let mut hit = None;
    let mut best = limit;
    for &shape in scene {
        if let Some((t, normal)) = ray.intersect(shape)
            && t < best
        {
            best = t;
            hit = Some((shape, t, normal));
        }
    }
    hit
}

/// 計算光源直接照到表面的線性亮度；姿態與直射遮擋皆參與計算。
fn direct(scene: &[Shape], p: Vec3, normal: Vec3, lamp: Lamp) -> Vec3 {
    let to_light = lamp.position - p;
    let distance = to_light.length();
    let direction = to_light / distance.max(EPSILON);
    let incidence = normal.dot(direction).max(0.0);
    if incidence == 0.0 || closest(scene, Ray::new(p, direction), distance - EPSILON).is_some() {
        return Vec3::ZERO;
    }
    lamp.color * (lamp.lumens / (2.0 * TAU) * incidence / distance.powi(2).max(0.18))
}

#[derive(Clone, Copy)]
struct Lighting {
    fill: Vec3,
    fixed: &'static [Lamp],
    switched: &'static [Lamp],
}

const STUDY: Lighting = Lighting {
    fill: FILL,
    fixed: &[MOON],
    switched: &[DESK],
};
const FOYER: Lighting = Lighting {
    fill: rooms::FOYER_FILL,
    fixed: &rooms::FOYER_FIXED,
    switched: &rooms::FOYER_SWITCHED,
};
const SIGNAL_ROOM: Lighting = Lighting {
    fill: rooms::SIGNAL_FILL,
    fixed: &rooms::SIGNAL_LIGHTS,
    switched: &[],
};
/// 取樣單一漫反射反彈，並用衰減距離近似牆角與接地處的環境遮蔽。
fn trace_indirect(scene: &[Shape], p: Vec3, normal: Vec3, lighting: Lighting) -> (Vec3, Vec3) {
    let tangent = if normal.y.abs() > 0.5 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let bitangent = normal.cross(tangent);
    let start = p + normal * EPSILON;
    let mut ambient = Vec3::ZERO;
    let mut warm = Vec3::ZERO;
    for i in 0..SAMPLES {
        let u = (i as f32 + 0.5) / SAMPLES as f32;
        let azimuth = TAU * (i as f32 * 0.618_034).fract();
        let (sin, cos) = azimuth.sin_cos();
        let direction = normal * (1.0 - u).sqrt() + (tangent * cos + bitangent * sin) * u.sqrt();
        if let Some((shape, distance, hit_normal)) =
            closest(scene, Ray::new(start, direction), 14.0)
        {
            let hit = start + direction * distance + hit_normal * EPSILON;
            let openness = 1.0 - 0.85 * (-distance / 0.62).exp();
            let incoming = |lights: &[Lamp]| {
                lights
                    .iter()
                    .map(|&lamp| direct(scene, hit, hit_normal, lamp))
                    .sum::<Vec3>()
                    * shape.albedo
            };
            ambient += lighting.fill * openness + incoming(lighting.fixed);
            warm += incoming(lighting.switched);
        } else {
            ambient += lighting.fill;
        }
    }
    (ambient / SAMPLES as f32, warm / SAMPLES as f32)
}

/// 計算一整個表面的月光和桌燈狀態；同一次追蹤共用樣本。
fn bake_tile(surface: Surface, scene: &[Shape], lighting: Lighting) -> Vec<[Vec3; 2]> {
    (0..SIDE * SIDE)
        .map(|pixel| {
            let uv = Vec2::new((pixel % SIDE) as f32, (pixel / SIDE) as f32) / (SIDE - 1) as f32;
            let (moon, warm) = trace_indirect(scene, surface.point(uv), surface.normal, lighting);
            [moon, moon + warm]
        })
        .collect()
}

/// 對圖塊使用小範圍的可分離濾波核，並將邊界像素延伸到留白。
fn write_tile(atlas: &mut [u8], tile: &[[Vec3; 2]], index: u32, mode: usize) {
    let (x0, y0) = tile_origin(index);
    let weights = [1.0, 2.0, 1.0];
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let sx = x.saturating_sub(PADDING).min(SIDE - 1) as i32;
            let sy = y.saturating_sub(PADDING).min(SIDE - 1) as i32;
            let mut smooth = Vec3::ZERO;
            for (dy, wy) in (-1..=1).zip(weights) {
                for (dx, wx) in (-1..=1).zip(weights) {
                    let tx = (sx + dx).clamp(0, SIDE as i32 - 1) as u32;
                    let ty = (sy + dy).clamp(0, SIDE as i32 - 1) as u32;
                    smooth += tile[(ty * SIDE + tx) as usize][mode] * (wx * wy / 16.0);
                }
            }
            let pixel = (((y0 + y) * ATLAS_WIDTH + x0 + x) * 4) as usize;
            atlas[pixel..pixel + 4].copy_from_slice(&rgba(smooth / LIGHTMAP_EXPOSURE));
        }
    }
}

/// 將線性顏色編成 PNG 的 sRGB 位元組；Bevy 讀取時會還原為線性。
fn rgba(color: Vec3) -> [u8; 4] {
    let [r, g, b] = color.to_array().map(|x| {
        let value = x.clamp(0.0, 1.0);
        let srgb = if value < 0.003_130_8 {
            12.92 * value
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        };
        (srgb * 255.0).round() as u8
    });
    [r, g, b, 255]
}

/// 產出可供遊戲載入的光照圖集 PNG。
fn save_png(
    destination: &Path,
    width: u32,
    height: u32,
    data: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD,
    )
    .try_into_dynamic()?
    .save(destination)?;
    Ok(())
}

/// 烘焙指定房間的所有圖塊，並依燈具狀態輸出圖集。
fn bake(
    root: &Path,
    surfaces: &[Surface],
    geometry: &[Shape],
    lighting: Lighting,
    outputs: &[&str],
) -> Result<(), Box<dyn Error>> {
    let jobs = thread::scope(|scope| {
        let workers: Vec<_> = surfaces
            .iter()
            .map(|&surface| scope.spawn(move || bake_tile(surface, geometry, lighting)))
            .collect();
        workers
            .into_iter()
            .map(|job| job.join().map_err(|_| "a lightmap worker panicked"))
            .collect::<Result<Vec<_>, _>>()
    })?;
    let height = atlas_height(surfaces.len());
    for (mode, relative) in outputs.iter().enumerate() {
        let mut atlas = vec![0; (ATLAS_WIDTH * height * 4) as usize];
        for (index, tile) in jobs.iter().enumerate() {
            write_tile(&mut atlas, tile, index as u32, mode);
        }
        let path = root.join(relative);
        save_png(&path, ATLAS_WIDTH, height, atlas)?;
        println!("{}", path.display());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    fs::create_dir_all(root.join("lighting"))?;
    bake(&root, &SURFACES, &scene(), STUDY, &[MOON_ATLAS, LAMP_ATLAS])?;
    let (surfaces, geometry) = rooms::foyer();
    bake(
        &root,
        &surfaces,
        &geometry,
        FOYER,
        &[FOYER_NIGHT, FOYER_LIT],
    )?;
    let (surfaces, geometry) = rooms::signal();
    bake(&root, &surfaces, &geometry, SIGNAL_ROOM, &[SIGNAL_ATLAS])
}
#[cfg(test)]
mod tests {
    use super::*;
    use latch::study::FLOOR;

    #[test]
    fn fixed_furniture_casts_indirect_contact_shadows() {
        let scene = scene();
        let floor_height = FLOOR.point(Vec2::splat(0.5)).y;
        let foot = DESK_LEGS[3].center;
        let (contact, _) = trace_indirect(
            &scene,
            vec3(foot.x + 0.08, floor_height, foot.z),
            Vec3::Y,
            STUDY,
        );
        let (open, _) = trace_indirect(&scene, vec3(2.6, floor_height, -2.1), Vec3::Y, STUDY);
        assert!(contact.element_sum() < open.element_sum());
    }

    #[test]
    fn desk_lamp_adds_warm_bounce_to_the_window_wall() {
        let scene = scene();
        let (_, lamp) = trace_indirect(&scene, vec3(-0.1, 1.15, -4.5), Vec3::Z, STUDY);
        assert!(lamp.x > 0.1 && lamp.x > lamp.y && lamp.y > lamp.z);
    }

    #[test]
    fn foyer_night_keeps_cool_light_without_warm_lamp_bounce() {
        let (_, geometry) = rooms::foyer();
        let (night, warm) = trace_indirect(&geometry, vec3(2.9, 0.013, 12.1), Vec3::Y, FOYER);
        assert!(night.z > night.x && night.x > 0.0);
        assert!(warm.x > warm.z && warm.x > 0.1);
    }

    #[test]
    fn signal_room_bakes_only_fixed_lights() {
        let (_, geometry) = rooms::signal();
        let (fixed, switched) =
            trace_indirect(&geometry, vec3(0.5, 0.023, 20.0), Vec3::Y, SIGNAL_ROOM);
        assert!(fixed.element_sum() > 0.0);
        assert_eq!(switched, Vec3::ZERO);
    }
}
