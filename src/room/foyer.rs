use super::{
    ClockHandPivot, ClockPendulum, Hinge, LeverHinge, LightKind, MoonMark, SwitchableLight,
    builder::Furnisher, materials::Palette,
};
use crate::{
    interaction::Target,
    physics::{Pushable, furniture_layers, world_layers},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use latch::puzzle::{ClockHand, Lever};
use std::f32::consts::{FRAC_PI_2, TAU};

/// 建立與午夜書房南側出口無縫銜接的長廊與老宅玄關門廳，複用共用 PBR 材質與輕量模型。
pub(super) fn setup(room: &mut Furnisher, assets: &AssetServer, palette: &Palette) {
    corridor(room, palette);
    shell(room, palette);
    blocking_crate_and_timetable(room, assets, palette);
    wall_clock_and_levers(room, palette);
    foyer_props(room, assets, palette);
    front_door_and_moon_mark(room, palette);
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

/// 在玄關西牆建立被可推動木箱遮住的舊鐵道月台時刻表，推開木箱即可細讀。
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
        FRAC_PI_2,
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

/// 在玄關東牆建立停在 12:00 的復古黃銅掛鐘與連動四道拉桿（I–IV），校時至 22:35 觸發鐘擺與拉桿解鎖。
fn wall_clock_and_levers(room: &mut Furnisher, palette: &Palette) {
    let clock_z = 8.20_f32;
    let face_y = 1.86_f32;
    room.block([3.44, 1.62, clock_z], [0.10, 0.96, 0.64], &palette.walnut);
    room.block([3.39, 1.36, clock_z], [0.04, 0.34, 0.36], &palette.ink);
    let rim = room.cylinder([3.39, face_y, clock_z], 0.28, 0.04, &palette.brass);
    room.commands.entity(rim).insert(
        Transform::from_xyz(3.39, face_y, clock_z)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
            .with_scale(Vec3::new(0.28, 0.04, 0.28)),
    );
    let dial = room.cylinder([3.37, face_y, clock_z], 0.25, 0.02, &palette.cream);
    room.commands.entity(dial).insert(
        Transform::from_xyz(3.37, face_y, clock_z)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
            .with_scale(Vec3::new(0.25, 0.02, 0.25)),
    );
    for tick in 0..12 {
        let angle = tick as f32 * (TAU / 12.0);
        let major = tick % 3 == 0;
        let radius = if major { 0.205 } else { 0.215 };
        let y = face_y + angle.cos() * radius;
        let z = clock_z + angle.sin() * radius;
        let length = if major { 0.048 } else { 0.024 };
        let width = if major { 0.018 } else { 0.010 };
        let mark = room.block([3.356, y, z], [0.008, length, width], &palette.ink);
        room.commands.entity(mark).insert(
            Transform::from_xyz(3.356, y, z)
                .with_rotation(Quat::from_rotation_x(angle))
                .with_scale(Vec3::new(0.008, length, width)),
        );
    }
    room.ellipsoid(
        [3.342, face_y, clock_z],
        [0.016, 0.016, 0.016],
        &palette.brass,
    );

    let hour_pivot = room
        .commands
        .spawn((
            Transform::from_xyz(3.350, face_y, clock_z),
            Visibility::default(),
            ClockHandPivot(ClockHand::Hour),
        ))
        .id();
    let hour_bar = room.block([0.0, 0.068, 0.0], [0.010, 0.145, 0.022], &palette.ink);
    room.commands.entity(hour_pivot).add_child(hour_bar);

    let minute_pivot = room
        .commands
        .spawn((
            Transform::from_xyz(3.344, face_y, clock_z),
            Visibility::default(),
            ClockHandPivot(ClockHand::Minute),
        ))
        .id();
    let minute_bar = room.block([0.0, 0.098, 0.0], [0.008, 0.205, 0.014], &palette.brass);
    room.commands.entity(minute_pivot).add_child(minute_bar);

    let pendulum = room
        .commands
        .spawn((
            Transform::from_xyz(3.36, 1.54, clock_z),
            Visibility::default(),
            ClockPendulum,
        ))
        .id();
    let rod = room.block([0.0, -0.14, 0.0], [0.010, 0.26, 0.010], &palette.brass);
    let bob = room.ellipsoid([0.0, -0.27, 0.0], [0.018, 0.052, 0.052], &palette.brass);
    room.commands.entity(pendulum).add_children(&[rod, bob]);

    for (hand, dz) in [(ClockHand::Hour, -0.22_f32), (ClockHand::Minute, 0.22)] {
        let knob = room.cylinder([3.36, 1.48, clock_z + dz], 0.036, 0.05, &palette.brass);
        room.commands.entity(knob).insert(
            Transform::from_xyz(3.36, 1.48, clock_z + dz)
                .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
                .with_scale(Vec3::new(0.036, 0.05, 0.036)),
        );
        room.sensor(
            [3.32, 1.48, clock_z + dz],
            [0.18, 0.22, 0.22],
            Target::ClockDial(hand),
        );
    }

    room.block([3.44, 1.48, 8.92], [0.05, 0.05, 0.85], &palette.brass);
    room.block([3.46, 1.48, 10.2], [0.08, 0.70, 1.84], &palette.walnut);
    room.block([3.41, 1.48, 10.2], [0.03, 0.58, 1.72], &palette.steel);
    room.block([3.44, 1.48, 12.25], [0.05, 0.05, 2.45], &palette.brass);
    room.block([2.45, 1.48, 13.45], [2.05, 0.05, 0.05], &palette.brass);

    for (lever, z, ticks) in [
        (Lever::One, 9.54, 1_usize),
        (Lever::Two, 9.98, 2),
        (Lever::Three, 10.42, 3),
        (Lever::Four, 10.86, 4),
    ] {
        let span = (ticks - 1) as f32 * 0.032;
        for t in 0..ticks {
            let tz = z - span * 0.5 + t as f32 * 0.032;
            room.block([3.39, 1.68, tz], [0.014, 0.075, 0.014], &palette.brass);
        }
        room.block([3.39, 1.46, z], [0.02, 0.30, 0.06], &palette.ink);
        let pivot = room
            .commands
            .spawn((
                Transform::from_xyz(3.38, 1.46, z).with_rotation(Quat::from_rotation_z(-0.55)),
                Visibility::default(),
                LeverHinge(lever),
            ))
            .id();
        let shaft = room.block([-0.11, 0.0, 0.0], [0.22, 0.032, 0.032], &palette.brass);
        let knob = room.ellipsoid([-0.22, 0.0, 0.0], [0.04, 0.04, 0.04], &palette.wood);
        room.commands.entity(pivot).add_children(&[shaft, knob]);
        room.sensor([3.26, 1.46, z], [0.30, 0.44, 0.32], Target::Lever(lever));
    }
}

/// 建立玄關信箱、衣帽架與黑傘、邊桌盆栽與連動的玄關燈具。
fn foyer_props(room: &mut Furnisher, assets: &AssetServer, palette: &Palette) {
    room.block([-1.18, 1.28, 13.44], [0.34, 0.42, 0.14], &palette.steel);
    room.block([-1.18, 1.40, 13.36], [0.26, 0.04, 0.03], &palette.brass);
    room.sensor([-1.18, 1.28, 13.40], [0.40, 0.48, 0.22], Target::Mailbox);

    room.cylinder([-1.95, 0.03, 12.85], 0.26, 0.05, &palette.walnut);
    room.cylinder([-1.95, 0.96, 12.85], 0.036, 1.86, &palette.walnut);
    for i in 0..4 {
        let angle = FRAC_PI_2 * i as f32;
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
        FRAC_PI_2,
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

    let lantern = room.light([0.5, 2.55, 10.3], [0.85, 0.82, 0.72], 1250.0, false);
    room.commands.entity(lantern).insert(SwitchableLight {
        kind: LightKind::FoyerLamp,
        on: true,
        intensity: 1250.0,
    });
    room.model(
        assets,
        "models/lantern_chandelier_01/lantern_chandelier_01_1k.gltf",
        [0.5, 3.50, 10.3],
        0.0,
        0.80,
    );
}

/// 建立南側臨街大門、夜光暗記及通往潮汐信號室的門廊。
fn front_door_and_moon_mark(room: &mut Furnisher, palette: &Palette) {
    let glow_paint = room.materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.76, 1.0),
        emissive: LinearRgba::rgb(0.42, 0.92, 1.95),
        ..default()
    });
    for (dy, width) in [
        (0.11_f32, 0.26_f32),
        (0.03, 0.22),
        (-0.05, 0.28),
        (-0.13, 0.18),
    ] {
        let stroke = room.block(
            [-0.74, 1.44 + dy, 13.48],
            [width, 0.022, 0.008],
            &glow_paint,
        );
        room.commands
            .entity(stroke)
            .insert((MoonMark, Visibility::Hidden));
    }
    room.sensor([-0.74, 1.44, 13.44], [0.36, 0.40, 0.16], Target::MoonMark);

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
    room.solid_block([-0.6, 1.65, 15.2], [0.2, 3.4, 3.2], &palette.wall);
    room.solid_block([1.6, 1.65, 15.2], [0.2, 3.4, 3.2], &palette.wall);
    room.solid_block([0.5, 3.4, 15.2], [2.4, 0.12, 3.2], &palette.trim);
    room.light([0.5, 2.2, 15.3], [0.46, 0.62, 1.0], 720.0, false);
}
