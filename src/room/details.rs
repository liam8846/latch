use super::builder::Furnisher;
use crate::interaction::Target;
use bevy::prelude::*;

/// 西牆 3:2 原創照片朝 +X，照片與 DOM 使用同一 JPEG；尺寸使用公尺。
pub(super) fn photograph(
    room: &mut Furnisher,
    assets: &AssetServer,
    walnut: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
    paper: &Handle<StandardMaterial>,
) {
    room.block([-4.38, 2.0, -1.0], [0.13, 1.26, 1.82], walnut);
    room.block([-4.29, 2.0, -1.0], [0.055, 1.15, 1.71], brass);
    room.block([-4.25, 2.0, -1.0], [0.04, 1.09, 1.65], paper);
    let photo = room.materials.add(StandardMaterial {
        base_color_texture: Some(assets.load("artwork/seaside-station-memory.jpg")),
        perceptual_roughness: 0.95,
        ..default()
    });
    let mesh = room.meshes.add(Rectangle::new(1.44, 0.96));
    room.commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(photo),
        Transform::from_xyz(-4.22, 2.02, -1.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
    ));
    room.target([-4.1, 2.0, -1.0], Target::Photograph);
    room.block([-4.13, 2.77, -1.0], [0.13, 0.055, 0.67], brass);
    room.light([-3.88, 2.66, -1.0], [1.0, 0.78, 0.50], 550.0, false);
}

/// 兩只杯子、筆、薄信與略轉的書本以公尺放置在既有桌面，不新增碰撞路線。
pub(super) fn keepsakes(
    room: &mut Furnisher,
    paper: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
    ink: &Handle<StandardMaterial>,
    wood: &Handle<StandardMaterial>,
    rust: &Handle<StandardMaterial>,
) {
    let ceramic = room.material([0.73, 0.77, 0.67], 0.05);
    let tea = room.material([0.09, 0.044, 0.023], 0.0);
    for (x, z) in [(-1.52, -2.64), (-1.22, -3.12)] {
        room.cylinder([x, 1.025, z], 0.13, 0.017, &ceramic);
        room.cylinder([x, 1.105, z], 0.075, 0.15, &ceramic);
        room.cylinder([x, 1.182, z], 0.061, 0.006, &tea);
        let handle = room.meshes.add(Torus::new(0.033, 0.05));
        room.commands.spawn((
            Mesh3d(handle),
            MeshMaterial3d(ceramic.clone()),
            Transform::from_xyz(x + 0.085, 1.11, z)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ));
    }
    room.block([-0.28, 1.023, -2.59], [0.018, 0.017, 0.32], ink);
    room.block([-0.28, 1.023, -2.42], [0.022, 0.021, 0.07], brass);
    for (at, size, material, angle) in [
        ([-1.65, 1.058, -3.1], [0.46, 0.09, 0.30], rust, 0.16),
        ([-1.65, 1.062, -3.1], [0.435, 0.063, 0.28], paper, 0.16),
        ([-1.65, 1.111, -3.1], [0.46, 0.014, 0.30], rust, 0.16),
        ([-0.66, 1.021, -3.18], [0.42, 0.008, 0.23], paper, -0.15),
    ] {
        let entity = room.block(at, size, material);
        room.commands.entity(entity).insert(
            Transform::from_translation(Vec3::from(at))
                .with_scale(Vec3::from(size))
                .with_rotation(Quat::from_rotation_y(angle)),
        );
    }
    room.block([3.16, 0.91, -1.65], [0.035, 0.22, 0.43], paper);
    let seal = room.ellipsoid([3.132, 0.94, -1.65], [0.012, 0.044, 0.044], rust);
    room.commands
        .entity(seal)
        .insert(Name::new("Unsent letter wax seal"));
    room.block([-2.65, 0.26, -3.62], [0.7, 0.52, 0.6], wood);
    room.block([-2.65, 0.535, -3.62], [0.72, 0.03, 0.62], paper);
    room.block([-2.65, 0.554, -3.62], [0.04, 0.012, 0.65], brass);
}

/// 連續窗簾摺面沿世界 Y 軸垂落，交錯深度形成柔和的立體陰影。
pub(super) fn curtains(
    room: &mut Furnisher,
    cloth: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
) {
    room.block([-0.9, 3.33, -4.12], [3.85, 0.035, 0.055], brass);
    for center in [-2.47, 0.68] {
        for fold in 0..9 {
            let x = center - 0.27 + fold as f32 * 0.068;
            let z = -4.13 + if fold % 2 == 0 { 0.085 } else { 0.0 };
            room.cylinder([x, 2.08, z], 0.054, 2.42, cloth);
        }
        room.block([center, 1.32, -4.0], [0.66, 0.045, 0.045], brass);
    }
}

/// 下半牆的凹板、細框與門套保持原房間邊界與出口寬度。
pub(super) fn panelling(
    room: &mut Furnisher,
    walnut: &Handle<StandardMaterial>,
    wood: &Handle<StandardMaterial>,
) {
    for side in [-1.0, 1.0] {
        room.block([side * 4.46, 0.54, 0.0], [0.04, 0.88, 8.8], walnut);
        for panel in 0..9 {
            let z = -4.0 + panel as f32;
            room.block([side * 4.42, 0.54, z], [0.035, 0.75, 0.035], wood);
        }
        for y in [0.20, 0.88] {
            room.block([side * 4.41, y, 0.0], [0.035, 0.028, 8.8], wood);
        }
    }
    for x in [0.81, 2.99] {
        room.block([x, 1.32, 4.36], [0.11, 2.64, 0.12], walnut);
        room.block([x, 0.20, 4.32], [0.17, 0.37, 0.15], wood);
    }
    room.block([1.9, 2.73, 4.37], [2.42, 0.16, 0.19], walnut);
}
