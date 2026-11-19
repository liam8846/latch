use super::{
    Hinge, LeverHinge, LightKind, SwitchableLight, builder::Furnisher, materials::Palette,
};
use crate::{
    interaction::Target,
    physics::{Pushable, furniture_layers, world_layers},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use latch::puzzle::Lever;

/// 建立與午夜書房南側出口無縫銜接的長廊與老宅玄關門廳，複用共用 PBR 材質與輕量模型。
pub(super) fn setup(room: &mut Furnisher, assets: &AssetServer, palette: &Palette) {
    corridor(room, palette);
    shell(room, palette);
    blocking_crate_and_timetable(room, assets, palette);
    lever_panel(room, palette);
    foyer_props(room, assets, palette);
    front_door(room, palette);
}

/// 建立連接書房與玄關的月光木廊（Z 4.6 至 7.0 公尺）。
fn corridor(room: &mut Furnisher, palette: &Palette) {
    room.block([1.9, 0.0, 5.8], [2.2, 0.025, 2.5], &palette.floor);
    room.solid_block([1.9, 3.55, 5.8], [2.4, 0.1, 2.5], &palette.trim);
    room.solid_block([0.85, 1.75, 5.8], [0.2, 3.5, 2.5], &palette.wall);
    room.solid_block([2.95, 1.75, 5.8], [0.2, 3.5, 2.5], &palette.wall);
    for x in [0.97, 2.83] {
        room.block([x, 0.15, 5.8], [0.06, 0.22, 2.4], &palette.walnut);
        room.block([x, 1.0, 5.8], [0.05, 0.06, 2.4], &palette.brass);
    }
    room.light([1.9, 2.35, 5.8], [0.75, 0.80, 1.0], 620.0, false);
}

/// 建立玄關地板、四面牆、天花板、地毯與踢腳板（X -2.6 至 3.6，Z 7.0 至 13.6）。
fn shell(room: &mut Furnisher, palette: &Palette) {
    room.block([0.5, -0.12, 10.3], [6.4, 0.2, 6.8], &palette.walnut);
    room.static_box([0.5, -0.0875, 12.0], [7.0, 0.2, 10.5]);
    room.block([0.5, 0.0, 10.3], [6.4, 0.025, 6.8], &palette.floor);
    room.solid_block([0.5, 3.55, 10.3], [6.6, 0.1, 6.8], &palette.trim);

    room.solid_block([-0.85, 1.75, 7.0], [3.5, 3.5, 0.2], &palette.wall);
    room.solid_block([3.25, 1.75, 7.0], [0.7, 3.5, 0.2], &palette.wall);
    room.solid_block([1.9, 3.0, 7.0], [2.0, 1.0, 0.2], &palette.wall);
    room.solid_block([-2.6, 1.75, 10.3], [0.2, 3.5, 6.8], &palette.wall);
    room.solid_block([3.6, 1.75, 10.3], [0.2, 3.5, 6.8], &palette.wall);
    room.solid_block([-1.5, 1.75, 13.6], [2.2, 3.5, 0.2], &palette.wall);
    room.solid_block([2.5, 1.75, 13.6], [2.2, 3.5, 0.2], &palette.wall);
    room.solid_block([0.5, 3.0, 13.6], [1.8, 1.0, 0.2], &palette.wall);

    for x in [-2.47, 3.47] {
        room.block([x, 0.52, 10.3], [0.05, 0.88, 6.4], &palette.walnut);
        room.block([x, 0.98, 10.3], [0.06, 0.06, 6.4], &palette.brass);
    }
    room.block([0.5, 0.032, 10.4], [2.4, 0.022, 4.4], &palette.rust);
    room.block([0.5, 0.046, 10.4], [2.16, 0.01, 4.16], &palette.cream);
    room.block([0.5, 0.054, 10.4], [2.04, 0.008, 4.04], &palette.rust);
}

/// 在玄關西牆建立被可推動木箱遮住的舊鐵道停靠時刻表，推開木箱即可細讀。
fn blocking_crate_and_timetable(room: &mut Furnisher, assets: &AssetServer, palette: &Palette) {
    room.block([-2.46, 0.72, 10.1], [0.04, 0.56, 0.72], &palette.walnut);
    room.block([-2.435, 0.72, 10.1], [0.02, 0.48, 0.64], &palette.cream);
    for row in 0..4 {
        room.block(
            [-2.42, 0.84 - row as f32 * 0.08, 10.1],
            [0.006, 0.012, 0.46 - row as f32 * 0.03],
            &palette.ink,
        );
    }
    room.sensor([-2.41, 0.72, 10.1], [0.16, 0.58, 0.74], Target::Timetable);

    let home = Transform::from_xyz(-1.98, 0.015, 10.1);
    let crate_root = room
        .commands
        .spawn((
            home,
            Visibility::default(),
            RigidBody::Dynamic,
            SleepingDisabled,
            Mass(5.5),
            Friction::new(0.15),
            LinearDamping(3.2),
            AngularDamping(4.5),
            LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            furniture_layers(),
            Pushable { home },
        ))
        .id();
    let lower_mesh = room.model(
        assets,
        "models/wooden_crate_01/wooden_crate_01_1k.gltf",
        [0.0, 0.0, 0.0],
        std::f32::consts::FRAC_PI_2,
        1.25,
    );
    let upper_chest = room.block([0.0, 0.68, 0.0], [0.56, 0.46, 0.78], &palette.wood);
    let upper_band = room.block([0.0, 0.68, 0.0], [0.58, 0.05, 0.80], &palette.brass);
    let collider = room
        .commands
        .spawn((
            Transform::from_xyz(0.0, 0.50, 0.0),
            Collider::cuboid(0.76, 1.0, 1.06),
            furniture_layers(),
        ))
        .id();
    room.commands
        .entity(crate_root)
        .add_children(&[lower_mesh, upper_chest, upper_band, collider]);
}

/// 在玄關東牆建立三道黃銅門栓拉桿（港灣、站台、海岸），附連通至臨街大門的銅導管。
fn lever_panel(room: &mut Furnisher, palette: &Palette) {
    room.block([3.46, 1.38, 10.2], [0.08, 0.68, 1.42], &palette.walnut);
    room.block([3.41, 1.38, 10.2], [0.03, 0.56, 1.30], &palette.steel);
    room.block([3.44, 1.38, 12.15], [0.05, 0.05, 2.65], &palette.brass);
    room.block([2.45, 1.38, 13.45], [2.05, 0.05, 0.05], &palette.brass);

    for (lever, z, bar_width) in [
        (Lever::Harbor, 9.76, 0.14),
        (Lever::Station, 10.20, 0.22),
        (Lever::Coast, 10.64, 0.08),
    ] {
        room.block([3.39, 1.58, z], [0.015, 0.05, bar_width], &palette.brass);
        room.block([3.39, 1.38, z], [0.02, 0.30, 0.06], &palette.ink);
        let pivot = room
            .commands
            .spawn((
                Transform::from_xyz(3.38, 1.38, z).with_rotation(Quat::from_rotation_z(-0.55)),
                Visibility::default(),
                LeverHinge(lever),
            ))
            .id();
        let shaft = room.block([-0.11, 0.0, 0.0], [0.22, 0.032, 0.032], &palette.brass);
        let knob = room.ellipsoid([-0.22, 0.0, 0.0], [0.042, 0.042, 0.042], &palette.wood);
        room.commands.entity(pivot).add_children(&[shaft, knob]);
        room.sensor([3.26, 1.38, z], [0.30, 0.44, 0.34], Target::Lever(lever));
    }
}

/// 建立玄關信箱、衣帽架與黑傘、邊桌盆栽與可切換的壁燈與主廊燈。
fn foyer_props(room: &mut Furnisher, assets: &AssetServer, palette: &Palette) {
    room.block([-1.05, 1.28, 13.44], [0.34, 0.42, 0.14], &palette.steel);
    room.block([-1.05, 1.40, 13.36], [0.26, 0.04, 0.03], &palette.brass);
    room.sensor([-1.05, 1.28, 13.40], [0.40, 0.48, 0.22], Target::Mailbox);

    room.cylinder([-1.95, 0.03, 12.85], 0.26, 0.05, &palette.walnut);
    room.cylinder([-1.95, 0.96, 12.85], 0.036, 1.86, &palette.walnut);
    for i in 0..4 {
        let angle = std::f32::consts::FRAC_PI_2 * i as f32;
        let hook = room.block(
            [-1.95 + angle.cos() * 0.12, 1.76, 12.85 + angle.sin() * 0.12],
            [0.16, 0.025, 0.025],
            &palette.brass,
        );
        room.commands.entity(hook).insert(
            Transform::from_xyz(-1.95 + angle.cos() * 0.10, 1.76, 12.85 + angle.sin() * 0.10)
                .with_rotation(Quat::from_rotation_y(-angle))
                .with_scale(Vec3::new(0.16, 0.025, 0.025)),
        );
    }
    room.cylinder([-1.75, 0.45, 12.88], 0.026, 0.86, &palette.ink);
    room.static_box([-1.95, 0.95, 12.85], [0.54, 1.90, 0.54]);
    room.sensor([-1.95, 1.05, 12.85], [0.64, 1.90, 0.64], Target::CoatRack);

    room.model(
        assets,
        "models/small_wooden_table_01/small_wooden_table_01_1k.gltf",
        [3.12, 0.015, 8.2],
        std::f32::consts::FRAC_PI_2,
        1.35,
    );
    room.static_box([3.12, 0.36, 8.2], [0.62, 0.72, 1.24]);
    room.model(
        assets,
        "models/potted_plant_04/potted_plant_04_1k.gltf",
        [3.12, 0.735, 8.2],
        -0.5,
        1.35,
    );

    room.block([3.44, 1.95, 12.2], [0.10, 0.24, 0.14], &palette.brass);
    room.sensor([3.40, 1.95, 12.2], [0.24, 0.36, 0.26], Target::FoyerLight);
    let sconce = room.light([3.05, 1.98, 12.2], [1.0, 0.74, 0.44], 950.0, false);
    room.commands.entity(sconce).insert(SwitchableLight {
        kind: LightKind::FoyerLamp,
        on: true,
        intensity: 950.0,
    });

    room.light([0.5, 2.55, 10.3], [0.85, 0.82, 0.72], 1250.0, false);
    room.model(
        assets,
        "models/lantern_chandelier_01/lantern_chandelier_01_1k.gltf",
        [0.5, 3.50, 10.3],
        0.0,
        0.80,
    );
}

/// 建立南側臨街大門（隨鉸鏈旋轉的碰撞體）與門外星夜門廊。
fn front_door(room: &mut Furnisher, palette: &Palette) {
    for x in [-0.46, 1.46] {
        room.block([x, 1.28, 13.48], [0.14, 2.56, 0.22], &palette.walnut);
    }
    room.block([0.5, 2.58, 13.48], [2.06, 0.16, 0.22], &palette.walnut);
    let pivot = room
        .commands
        .spawn((
            Transform::from_xyz(-0.40, 0.0, 13.56),
            Visibility::default(),
            RigidBody::Kinematic,
            Hinge::FrontDoor,
            Target::FrontDoor,
        ))
        .id();
    let door = room.block([0.90, 1.25, 0.0], [1.80, 2.50, 0.14], &palette.walnut);
    room.commands
        .entity(door)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let panel = room.block([0.90, 1.45, -0.08], [1.46, 1.62, 0.04], &palette.wood);
    let handle = room.block([1.58, 1.12, -0.14], [0.22, 0.055, 0.08], &palette.brass);
    let bolt_bar = room.block([1.56, 1.38, -0.10], [0.28, 0.06, 0.04], &palette.brass);
    room.commands
        .entity(pivot)
        .add_children(&[door, panel, handle, bolt_bar]);

    room.block([0.5, 0.0, 15.1], [2.4, 0.06, 3.2], &palette.wood);
    room.solid_block([0.5, 1.65, 16.7], [2.8, 3.4, 0.14], &palette.moon);
    room.static_box([-0.6, 1.65, 15.2], [0.2, 3.4, 3.2]);
    room.static_box([1.6, 1.65, 15.2], [0.2, 3.4, 3.2]);
    room.light([0.5, 2.2, 15.3], [0.46, 0.62, 1.0], 720.0, false);
}
