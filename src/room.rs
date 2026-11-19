mod builder;
mod details;
mod furniture;
mod materials;

use crate::{Session, interaction::Target};
use bevy::prelude::*;
use builder::Furnisher;
use furniture::{chair, desk_drawer, exit, pressure_plate, safe, shelves};
use latch::puzzle::Progress;

#[derive(Component)]
pub enum Hinge {
    Safe,
    Exit,
}

#[derive(Component)]
pub struct Key;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    DeskLamp,
    Chandelier,
}

#[derive(Component)]
pub struct SwitchableLight {
    pub kind: LightKind,
    pub on: bool,
    pub intensity: f32,
}

/// 建立午夜書房，Y 向上，房間為九公尺見方，北側為 -Z，全表面套用 CC0 PBR 貼圖。
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let cube_mesh = Cuboid::new(1.0, 1.0, 1.0)
        .mesh()
        .build()
        .with_generated_tangents()
        .unwrap_or_else(|_| Cuboid::new(1.0, 1.0, 1.0).into());
    let cube = meshes.add(cube_mesh);
    let palette = materials::build(&assets, &mut materials);
    let mut room = Furnisher {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        cube,
    };
    room.block([0.0, -0.12, 0.0], [9.2, 0.2, 9.2], &palette.walnut);
    room.static_box([0.0, -0.0875, 1.0], [10.0, 0.2, 14.0]);
    room.block([0.0, 0.0, 0.0], [9.2, 0.025, 9.2], &palette.floor);
    room.solid_block([0.0, 1.75, -4.6], [9.2, 3.5, 0.2], &palette.wall);
    room.solid_block([-4.6, 1.75, 0.0], [0.2, 3.5, 9.2], &palette.wall);
    room.solid_block([4.6, 1.75, 0.0], [0.2, 3.5, 9.2], &palette.wall);
    room.solid_block([-1.8, 1.75, 4.6], [5.6, 3.5, 0.2], &palette.wall);
    room.solid_block([3.7, 1.75, 4.6], [1.8, 3.5, 0.2], &palette.wall);
    room.solid_block([1.9, 3.0, 4.6], [1.8, 1.0, 0.2], &palette.wall);
    room.solid_block([0.0, 3.55, 0.0], [9.3, 0.1, 9.3], &palette.trim);
    for side in [-1.0, 1.0] {
        for height in [0.14, 1.0, 3.25] {
            room.block(
                [side * 4.47, height, 0.0],
                [0.08, 0.07, 9.0],
                &palette.brass,
            );
        }
        room.block([0.0, 0.15, side * 4.47], [9.0, 0.21, 0.09], &palette.trim);
    }
    for x in -4..5 {
        room.block([x as f32, 0.52, -4.47], [0.035, 0.84, 0.05], &palette.trim);
    }
    room.block([0.0, 0.035, 0.15], [4.6, 0.026, 4.0], &palette.rust);
    room.block([0.0, 0.052, 0.15], [4.35, 0.012, 3.75], &palette.cream);
    room.block([0.0, 0.062, 0.15], [4.2, 0.01, 3.6], &palette.rust);
    for z in [-1.5, 1.8] {
        for i in 0..20 {
            room.block(
                [-1.98 + i as f32 * 0.208, 0.07, z],
                [0.10, 0.01, 0.07],
                &palette.brass,
            );
        }
    }
    room.block([-0.9, 2.16, -4.43], [2.6, 2.1, 0.07], &palette.walnut);
    room.block([-0.9, 2.16, -4.37], [2.37, 1.88, 0.04], &palette.moon);
    room.block([-0.9, 2.16, -4.3], [0.06, 1.92, 0.1], &palette.brass);
    room.block([-0.9, 2.16, -4.3], [2.42, 0.06, 0.1], &palette.brass);
    room.solid_block([-0.9, 1.12, -4.17], [2.9, 0.12, 0.42], &palette.wood);
    room.ellipsoid([-0.13, 2.7, -4.28], [0.19, 0.19, 0.04], &palette.cream);
    details::curtains(&mut room, &palette.trim, &palette.brass);
    room.light([-0.8, 2.65, -3.9], [0.48, 0.63, 1.0], 1650.0, false);
    room.solid_block([-0.8, 0.94, -2.85], [2.65, 0.13, 1.12], &palette.wood);
    for x in [-1.93, 0.33] {
        for z in [-3.27, -2.43] {
            room.solid_block([x, 0.44, z], [0.12, 0.87, 0.12], &palette.walnut);
        }
    }
    desk_drawer(&mut room, &palette.walnut, &palette.wood, &palette.brass);
    room.block([-0.65, 1.023, -2.56], [0.47, 0.015, 0.34], &palette.cream);
    for i in 0..4 {
        room.block(
            [-0.65, 1.033, -2.66 + i as f32 * 0.055],
            [0.31 - i as f32 * 0.035, 0.003, 0.009],
            &palette.ink,
        );
    }
    room.sensor([-0.65, 1.05, -2.56], [0.50, 0.08, 0.38], Target::Note);
    room.model(
        &assets,
        "models/vintage_oil_lamp/vintage_oil_lamp_1k.gltf",
        [0.18, 1.005, -3.05],
        -0.35,
        0.72,
    );
    room.sensor([0.18, 1.32, -3.05], [0.30, 0.62, 0.30], Target::DeskLamp);
    let desk_light = room.light([0.18, 1.42, -2.92], [1.0, 0.72, 0.42], 1550.0, true);
    room.commands.entity(desk_light).insert(SwitchableLight {
        kind: LightKind::DeskLamp,
        on: true,
        intensity: 1550.0,
    });
    chair(&mut room, &assets);
    shelves(&mut room, &palette.walnut, &palette.wood, &palette.brass);
    safe(&mut room, &palette.steel, &palette.brass, &palette.ink);
    pressure_plate(&mut room, &assets, &palette.walnut, &palette.brass);
    details::photograph(
        &mut room,
        &assets,
        &palette.walnut,
        &palette.brass,
        &palette.cream,
    );
    details::keepsakes(
        &mut room,
        &assets,
        &palette.cream,
        &palette.brass,
        &palette.ink,
        &palette.wood,
        &palette.rust,
    );
    details::panelling(&mut room, &palette.walnut, &palette.wood);
    room.model(
        &assets,
        "models/wooden_crate_01/wooden_crate_01_1k.gltf",
        [-3.45, 0.02, -3.55],
        0.25,
        1.35,
    );
    room.static_box([-3.45, 0.28, -3.55], [1.12, 0.52, 0.62]);
    room.sensor([-3.45, 0.32, -3.55], [1.14, 0.56, 0.64], Target::Crate);
    exit(&mut room, &palette.wood, &palette.walnut, &palette.brass);
    room.block([0.52, 1.32, 4.46], [0.11, 0.16, 0.04], &palette.brass);
    room.block([0.52, 1.32, 4.43], [0.03, 0.05, 0.03], &palette.walnut);
    room.sensor([0.52, 1.32, 4.44], [0.22, 0.24, 0.12], Target::WallSwitch);
    let main_light = room.light([0.0, 2.55, 0.1], [1.0, 0.78, 0.51], 1800.0, true);
    room.commands.entity(main_light).insert(SwitchableLight {
        kind: LightKind::Chandelier,
        on: true,
        intensity: 1800.0,
    });
    room.model(
        &assets,
        "models/lantern_chandelier_01/lantern_chandelier_01_1k.gltf",
        [0.0, 3.50, 0.1],
        0.0,
        0.85,
    );
}

/// 以秒步長逼近目標角度；門與鑰匙的呈現完全由唯一謎題進度導出。
pub fn animate(
    time: Res<Time>,
    session: Res<Session>,
    mut hinges: Query<(&Hinge, &mut Transform)>,
    mut keys: Query<&mut Visibility, With<Key>>,
) {
    for (hinge, mut transform) in &mut hinges {
        let open = match hinge {
            Hinge::Safe => session.progress.safe_is_open(),
            Hinge::Exit => session.progress.door_is_open(),
        };
        let target = if open {
            -std::f32::consts::FRAC_PI_2 * 0.95
        } else {
            0.0
        };
        transform.rotation = transform.rotation.slerp(
            Quat::from_rotation_y(target),
            1.0 - (-time.delta_secs() * 5.0).exp(),
        );
    }
    for mut visibility in &mut keys {
        *visibility = if session.progress == Progress::SafeOpen {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
