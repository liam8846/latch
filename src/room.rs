mod builder;
mod details;
mod furniture;
mod materials;

use crate::{Session, interaction::Target};
use bevy::prelude::*;
use builder::Furnisher;
use furniture::{exit, safe, shelves};
use latch::puzzle::Progress;

#[derive(Component)]
pub enum Hinge {
    Safe,
    Exit,
}

#[derive(Component)]
pub struct Key;

/// 建立原創程序式午夜書房，Y 向上，房間為九公尺見方，北側為 -Z。
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut room = Furnisher {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        cube,
    };
    let wall = room.material([0.08, 0.19, 0.19], 0.0);
    let trim = room.material([0.043, 0.10, 0.10], 0.0);
    let wood = room.material([0.28, 0.13, 0.073], 0.0);
    let walnut = room.material([0.17, 0.075, 0.04], 0.0);
    let brass = room.material([0.64, 0.42, 0.16], 0.8);
    let cream = room.material([0.78, 0.71, 0.53], 0.0);
    let ink = room.material([0.02, 0.035, 0.04], 0.0);
    let rust = room.material([0.34, 0.13, 0.08], 0.0);
    let leaves = room.material([0.075, 0.23, 0.12], 0.0);
    let steel = room.material([0.12, 0.17, 0.17], 0.6);
    let glow = room.materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.75, 0.38),
        emissive: LinearRgba::rgb(3.0, 1.5, 0.4),
        ..default()
    });
    let moon = room.materials.add(StandardMaterial {
        base_color: Color::srgb(0.025, 0.055, 0.10),
        emissive: LinearRgba::rgb(0.018, 0.045, 0.10),
        ..default()
    });
    materials::apply(
        &mut room,
        &mut images,
        &[
            (&wood, materials::Surface::Wood),
            (&walnut, materials::Surface::Wood),
            (&cream, materials::Surface::Paper),
            (&rust, materials::Surface::Cloth),
            (&trim, materials::Surface::Cloth),
            (&wall, materials::Surface::Plaster),
        ],
    );
    room.block([0.0, -0.12, 0.0], [9.2, 0.2, 9.2], &walnut);
    for row in 0..24 {
        for column in 0..8 {
            let shade = 0.8 + ((row * 7 + column * 3) % 9) as f32 * 0.035;
            let plank = room.material([0.29 * shade, 0.155 * shade, 0.085 * shade], 0.0);
            let x = -3.94 + column as f32 * 1.125;
            let z = -4.31 + row as f32 * 0.375;
            room.block([x, 0.0, z], [1.115, 0.025, 0.365], &plank);
        }
    }
    room.block([0.0, 1.75, -4.6], [9.2, 3.5, 0.2], &wall);
    room.block([-4.6, 1.75, 0.0], [0.2, 3.5, 9.2], &wall);
    room.block([4.6, 1.75, 0.0], [0.2, 3.5, 9.2], &wall);
    room.block([-1.8, 1.75, 4.6], [5.6, 3.5, 0.2], &wall);
    room.block([3.7, 1.75, 4.6], [1.8, 3.5, 0.2], &wall);
    room.block([1.9, 3.0, 4.6], [1.8, 1.0, 0.2], &wall);
    room.block([0.0, 3.55, 0.0], [9.3, 0.1, 9.3], &trim);
    for side in [-1.0, 1.0] {
        for height in [0.14, 1.0, 3.25] {
            room.block([side * 4.47, height, 0.0], [0.08, 0.07, 9.0], &brass);
        }
        room.block([0.0, 0.15, side * 4.47], [9.0, 0.21, 0.09], &trim);
    }
    for x in -4..5 {
        room.block([x as f32, 0.52, -4.47], [0.035, 0.84, 0.05], &trim);
    }
    room.block([0.0, 0.035, 0.15], [4.6, 0.026, 4.0], &rust);
    room.block([0.0, 0.052, 0.15], [4.35, 0.012, 3.75], &cream);
    room.block([0.0, 0.062, 0.15], [4.2, 0.01, 3.6], &rust);
    for z in [-1.5, 1.8] {
        for i in 0..20 {
            room.block(
                [-1.98 + i as f32 * 0.208, 0.07, z],
                [0.10, 0.01, 0.07],
                &brass,
            );
        }
    }
    room.block([-0.9, 2.16, -4.43], [2.6, 2.1, 0.07], &walnut);
    room.block([-0.9, 2.16, -4.37], [2.37, 1.88, 0.04], &moon);
    room.block([-0.9, 2.16, -4.3], [0.06, 1.92, 0.1], &brass);
    room.block([-0.9, 2.16, -4.3], [2.42, 0.06, 0.1], &brass);
    room.block([-0.9, 1.12, -4.17], [2.9, 0.12, 0.42], &wood);
    room.ellipsoid([-0.13, 2.7, -4.28], [0.19, 0.19, 0.04], &cream);
    details::curtains(&mut room, &trim, &brass);
    room.light([-0.8, 2.65, -3.9], [0.48, 0.63, 1.0], 1650.0, true);
    room.block([-0.8, 0.94, -2.85], [2.65, 0.13, 1.12], &wood);
    for x in [-1.93, 0.33] {
        for z in [-3.27, -2.43] {
            room.block([x, 0.46, z], [0.12, 0.92, 0.12], &walnut);
        }
    }
    room.block([-1.45, 0.69, -2.86], [0.69, 0.37, 0.91], &walnut);
    room.block([-1.45, 0.69, -2.39], [0.63, 0.28, 0.03], &wood);
    room.block([-1.45, 0.69, -2.35], [0.21, 0.025, 0.04], &brass);
    room.block([-0.65, 1.023, -2.56], [0.47, 0.015, 0.34], &cream);
    for i in 0..4 {
        room.block(
            [-0.65, 1.033, -2.66 + i as f32 * 0.055],
            [0.31 - i as f32 * 0.035, 0.003, 0.009],
            &ink,
        );
    }
    room.target([-0.65, 1.08, -2.55], Target::Note);
    room.solid([-0.8, 0.5, -2.85], [1.33, 0.5, 0.57]);
    room.cylinder([0.1, 1.06, -3.03], 0.18, 0.07, &brass);
    room.cylinder([0.1, 1.38, -3.03], 0.024, 0.65, &brass);
    room.cylinder([0.1, 1.7, -3.03], 0.31, 0.24, &trim);
    room.cylinder([0.1, 1.57, -3.03], 0.26, 0.016, &glow);
    room.light([0.1, 1.52, -2.90], [1.0, 0.72, 0.42], 1550.0, true);
    room.block([-0.65, 0.53, -1.62], [0.7, 0.15, 0.67], &walnut);
    room.block([-0.65, 0.61, -1.62], [0.64, 0.08, 0.61], &trim);
    room.block([-0.65, 1.02, -1.33], [0.7, 0.82, 0.09], &wood);
    for x in [-0.94, -0.36] {
        for z in [-1.87, -1.37] {
            room.block([x, 0.25, z], [0.07, 0.5, 0.07], &walnut);
        }
    }
    room.solid([-0.65, 0.7, -1.62], [0.36, 0.7, 0.35]);
    shelves(&mut room, &walnut, &wood, &brass);
    safe(&mut room, &steel, &brass, &ink);
    details::photograph(&mut room, &assets, &walnut, &brass, &cream);
    details::keepsakes(&mut room, &cream, &brass, &ink, &wood, &rust);
    details::panelling(&mut room, &walnut, &wood);
    room.cylinder([-3.6, 0.32, -3.6], 0.36, 0.64, &rust);
    room.cylinder([-3.6, 0.66, -3.6], 0.32, 0.04, &ink);
    for i in 0..9 {
        let angle = i as f32 * 2.4;
        let height = 0.95 + (i % 4) as f32 * 0.2;
        let x = -3.6 + angle.cos() * 0.3;
        let z = -3.6 + angle.sin() * 0.3;
        room.cylinder([x, height * 0.5 + 0.4, z], 0.018, height, &wood);
        let leaf = room.ellipsoid([x, height + 0.35, z], [0.28, 0.11, 0.15], &leaves);
        room.commands.entity(leaf).insert(
            Transform::from_xyz(x, height + 0.35, z)
                .with_rotation(Quat::from_rotation_y(angle))
                .with_scale(Vec3::new(0.28, 0.11, 0.15)),
        );
    }
    room.solid([-3.6, 0.7, -3.6], [0.4, 0.7, 0.4]);
    exit(&mut room, &wood, &walnut, &brass);
    room.light([0.0, 2.64, 0.1], [1.0, 0.78, 0.51], 1800.0, true);
    room.cylinder([0.0, 3.13, 0.1], 0.025, 0.66, &brass);
    room.cylinder([0.0, 2.83, 0.1], 0.48, 0.2, &trim);
    room.cylinder([0.0, 2.72, 0.1], 0.43, 0.025, &glow);
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
