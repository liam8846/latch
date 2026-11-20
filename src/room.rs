mod builder;
mod details;
mod foyer;
mod furniture;
mod materials;
pub mod signal;

use crate::{Session, interaction::Target, physics::PressurePlate};
use bevy::prelude::*;
use builder::Furnisher;
use furniture::{balance_corner, chair, desk_drawer, exit, safe, shelves};
use latch::puzzle::{ClockHand, Lever, Progress};

#[derive(Component)]
pub enum Hinge {
    Safe,
    Exit,
    FrontDoor,
    SignalGate,
}

#[derive(Component)]
pub struct LeverHinge(pub Lever);

#[derive(Component)]
pub struct ClockHandPivot(pub ClockHand);

#[derive(Component)]
pub struct ClockPendulum;

#[derive(Component)]
pub struct Key;

#[derive(Component)]
pub struct MoonMark;

#[derive(Component)]
pub struct BalanceBeam;

#[derive(Component)]
pub struct BalanceCounterPan {
    pub low_y: f32,
    pub high_y: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    DeskLamp,
    Chandelier,
    FoyerLamp,
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
    let cube = meshes.add(
        Cuboid::new(1.0, 1.0, 1.0)
            .mesh()
            .build()
            .with_generated_tangents()
            .unwrap_or_else(|_| Cuboid::new(1.0, 1.0, 1.0).into()),
    );
    let cylinder = meshes.add(
        Cylinder::new(1.0, 1.0)
            .mesh()
            .resolution(16)
            .build()
            .with_generated_tangents()
            .unwrap_or_else(|_| Cylinder::new(1.0, 1.0).into()),
    );
    let sphere = meshes.add(
        Sphere::new(1.0)
            .mesh()
            .uv(12, 8)
            .with_generated_tangents()
            .unwrap_or_else(|_| Sphere::new(1.0).mesh().uv(12, 8)),
    );
    let palette = materials::build(&assets, &mut materials);
    let mut room = Furnisher {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        cube,
        cylinder,
        sphere,
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
    balance_corner(&mut room, &assets, &palette);
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
    foyer::setup(&mut room, &assets, &palette);
    signal::setup(&mut room, &palette);
}

type MechanismAnimateQueries<'w, 's> = (
    Query<
        'w,
        's,
        (&'static PressurePlate, &'static Transform),
        (
            Without<Hinge>,
            Without<BalanceBeam>,
            Without<BalanceCounterPan>,
            Without<LeverHinge>,
            Without<ClockHandPivot>,
            Without<ClockPendulum>,
        ),
    >,
    Query<
        'w,
        's,
        &'static mut Transform,
        (
            With<BalanceBeam>,
            Without<Hinge>,
            Without<BalanceCounterPan>,
            Without<LeverHinge>,
            Without<ClockHandPivot>,
            Without<ClockPendulum>,
        ),
    >,
    Query<
        'w,
        's,
        (&'static BalanceCounterPan, &'static mut Transform),
        (
            Without<Hinge>,
            Without<BalanceBeam>,
            Without<LeverHinge>,
            Without<ClockHandPivot>,
            Without<ClockPendulum>,
        ),
    >,
    Query<
        'w,
        's,
        (&'static LeverHinge, &'static mut Transform),
        (
            Without<Hinge>,
            Without<BalanceBeam>,
            Without<BalanceCounterPan>,
            Without<ClockHandPivot>,
            Without<ClockPendulum>,
        ),
    >,
    Query<
        'w,
        's,
        (&'static ClockHandPivot, &'static mut Transform),
        (
            Without<Hinge>,
            Without<BalanceBeam>,
            Without<BalanceCounterPan>,
            Without<LeverHinge>,
            Without<ClockPendulum>,
        ),
    >,
    Query<
        'w,
        's,
        &'static mut Transform,
        (
            With<ClockPendulum>,
            Without<Hinge>,
            Without<BalanceBeam>,
            Without<BalanceCounterPan>,
            Without<LeverHinge>,
            Without<ClockHandPivot>,
        ),
    >,
);

type VisibilityAnimateQueries<'w, 's> = (
    Query<'w, 's, &'static mut Visibility, (With<Key>, Without<MoonMark>)>,
    Query<'w, 's, &'static mut Visibility, (With<MoonMark>, Without<Key>)>,
    Query<'w, 's, &'static SwitchableLight>,
);

/// 以秒步長逼近門板、天平、掛鐘指針／鐘擺與玄關拉桿，並同步鑰匙與熄燈夜光暗記的可見度。
pub fn animate(
    time: Res<Time>,
    session: Res<Session>,
    mut hinges: Query<(&Hinge, &mut Transform)>,
    mut visibility_queries: VisibilityAnimateQueries,
    mut mechanisms: MechanismAnimateQueries,
) {
    let dt = time.delta_secs().min(0.1);
    for (hinge, mut transform) in &mut hinges {
        let open = match hinge {
            Hinge::Safe => session.progress.safe_is_open(),
            Hinge::Exit => session.progress.door_is_open(),
            Hinge::FrontDoor => session.progress.front_door_is_open(),
            Hinge::SignalGate => session.progress.signal_gate_is_open(),
        };
        let target = Quat::from_rotation_y(if open {
            -std::f32::consts::FRAC_PI_2 * 0.95
        } else {
            0.0
        });
        if transform.rotation.angle_between(target) > 1e-4 {
            transform.rotation = transform.rotation.slerp(target, 1.0 - (-dt * 5.0).exp());
        }
    }
    let ratio = mechanisms
        .0
        .iter()
        .next()
        .map_or(0.0, |(plate, transform)| {
            ((plate.base_y - transform.translation.y) / plate.travel).clamp(0.0, 1.0)
        });
    let tilt = Quat::from_rotation_x(0.18 + (-0.04 - 0.18) * ratio);
    for mut transform in &mut mechanisms.1 {
        if transform.rotation.angle_between(tilt) > 1e-4 {
            transform.rotation = tilt;
        }
    }
    for (pan, mut transform) in &mut mechanisms.2 {
        let target_y = pan.low_y + (pan.high_y - pan.low_y) * ratio;
        if (transform.translation.y - target_y).abs() > 1e-4 {
            transform.translation.y = target_y;
        }
    }
    for (lever, mut transform) in &mut mechanisms.3 {
        let target = Quat::from_rotation_z(if session.progress.lever_is_down(lever.0) {
            0.55
        } else {
            -0.55
        });
        if transform.rotation.angle_between(target) > 1e-4 {
            transform.rotation = transform.rotation.slerp(target, 1.0 - (-dt * 9.0).exp());
        }
    }
    for (hand, mut transform) in &mut mechanisms.4 {
        let angle = match hand.0 {
            ClockHand::Hour => session.clock.hour_angle_rad(),
            ClockHand::Minute => session.clock.minute_angle_rad(),
        };
        let target = Quat::from_rotation_x(angle);
        if transform.rotation.angle_between(target) > 1e-4 {
            transform.rotation = transform.rotation.slerp(target, 1.0 - (-dt * 12.0).exp());
        }
    }
    let swing_target = Quat::from_rotation_x(if session.progress.foyer_clock_synced() {
        (time.elapsed_secs() * 3.0).sin() * 0.16
    } else {
        0.0
    });
    for mut transform in &mut mechanisms.5 {
        if transform.rotation.angle_between(swing_target) > 1e-4 {
            transform.rotation = swing_target;
        }
    }
    for mut visibility in &mut visibility_queries.0 {
        let next = if session.progress == Progress::SafeOpen {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
    let foyer_dark = visibility_queries
        .2
        .iter()
        .any(|light| light.kind == LightKind::FoyerLamp && !light.on);
    let mark_vis = if foyer_dark {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut visibility_queries.1 {
        if *visibility != mark_vis {
            *visibility = mark_vis;
        }
    }
}
