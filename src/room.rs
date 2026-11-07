use crate::{Session, interaction::Target, player::Solid};
use bevy::prelude::*;
use latch::puzzle::Progress;

#[derive(Component)]
pub enum Hinge {
    Safe,
    Exit,
}

#[derive(Component)]
pub struct Key;

struct Furnisher<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    cube: Handle<Mesh>,
}

impl Furnisher<'_, '_, '_> {
    /// 建立可重用 PBR 材質；sRGB 色值為 0 至 1，粗糙度與金屬度由用途決定。
    fn material(&mut self, rgb: [f32; 3], metallic: f32) -> Handle<StandardMaterial> {
        self.materials.add(StandardMaterial {
            base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
            metallic,
            perceptual_roughness: if metallic > 0.5 { 0.28 } else { 0.8 },
            ..default()
        })
    }

    /// 以公尺制世界座標建立長方體，縮放共用單位立方體以減少網格資源。
    fn block(
        &mut self,
        at: [f32; 3],
        size: [f32; 3],
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        self.commands
            .spawn((
                Mesh3d(self.cube.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)).with_scale(Vec3::from(size)),
            ))
            .id()
    }

    /// 建立垂直圓柱，半徑與高度均以公尺表示。
    fn cylinder(
        &mut self,
        at: [f32; 3],
        radius: f32,
        height: f32,
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.meshes.add(Cylinder::new(radius, height));
        self.commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)),
            ))
            .id()
    }

    /// 建立不透明橢球裝飾，以公尺制三軸縮放球體。
    fn ellipsoid(
        &mut self,
        at: [f32; 3],
        size: [f32; 3],
        material: &Handle<StandardMaterial>,
    ) -> Entity {
        let mesh = self.meshes.add(Sphere::new(1.0).mesh().uv(12, 8));
        self.commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(Vec3::from(at)).with_scale(Vec3::from(size)),
            ))
            .id()
    }

    /// 加入玩家水平碰撞與互動視線共用的世界空間包圍盒。
    fn solid(&mut self, at: [f32; 3], half: [f32; 3]) {
        self.commands.spawn(Solid {
            center: Vec3::from(at),
            half: Vec3::from(half),
        });
    }

    /// 以獨立節點定義互動焦點，和可視網格、碰撞體分開。
    fn target(&mut self, at: [f32; 3], target: Target) {
        self.commands
            .spawn((Transform::from_translation(Vec3::from(at)), target));
    }

    /// 建立暖色或月光點光源；光通量為流明，只有主要燈源產生陰影。
    fn light(&mut self, at: [f32; 3], rgb: [f32; 3], lumens: f32, shadows: bool) {
        self.commands.spawn((
            PointLight {
                color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                intensity: lumens,
                range: 16.0,
                radius: 0.25,
                shadow_maps_enabled: shadows,
                ..default()
            },
            Transform::from_translation(Vec3::from(at)),
        ));
    }
}

/// 建立原創程序式午夜書房，Y 向上，房間為九公尺見方，北側為 -Z。
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        base_color: Color::srgb(0.2, 0.33, 0.42),
        emissive: LinearRgba::rgb(0.22, 0.42, 0.6),
        ..default()
    });
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
    for x in [-2.45, 0.65] {
        room.block([x, 2.1, -4.19], [0.46, 2.45, 0.16], &trim);
        for dx in [-0.16, 0.0, 0.16] {
            room.block([x + dx, 2.1, -4.08], [0.045, 2.4, 0.1], &wall);
        }
    }
    room.light([-0.8, 2.65, -3.9], [0.42, 0.67, 1.0], 1000.0, false);
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
    room.light([0.1, 1.52, -3.03], [1.0, 0.68, 0.32], 620.0, true);
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
    room.block([-4.38, 2.0, -1.0], [0.12, 1.52, 1.16], &brass);
    room.block([-4.3, 2.0, -1.0], [0.06, 1.36, 1.0], &cream);
    room.block([-4.25, 1.93, -1.0], [0.02, 0.76, 0.88], &trim);
    room.ellipsoid([-4.22, 2.25, -1.15], [0.035, 0.23, 0.23], &rust);
    room.target([-4.1, 2.0, -1.0], Target::Painting);
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
    exit(&mut room, &wood, &walnut, &brass, &glow);
    room.light([0.0, 2.9, 0.1], [1.0, 0.71, 0.4], 1100.0, false);
    room.cylinder([0.0, 3.13, 0.1], 0.025, 0.66, &brass);
    room.cylinder([0.0, 2.83, 0.1], 0.48, 0.2, &trim);
    room.cylinder([0.0, 2.72, 0.1], 0.43, 0.025, &glow);
}

/// 建立西側書架與有節奏的書脊；不依賴任何外部模型或貼圖。
fn shelves(
    room: &mut Furnisher,
    walnut: &Handle<StandardMaterial>,
    wood: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
) {
    room.block([-4.06, 1.24, 1.42], [0.64, 2.48, 2.8], walnut);
    room.block([-3.71, 1.32, 1.42], [0.08, 2.33, 2.63], wood);
    for y in [0.22, 0.83, 1.44, 2.05, 2.52] {
        room.block([-3.68, y, 1.42], [0.85, 0.07, 2.87], walnut);
    }
    for tier in 0..3 {
        for i in 0..13 {
            let color = match i % 4 {
                0 => [0.22, 0.31, 0.26],
                1 => [0.38, 0.15, 0.09],
                2 => [0.45, 0.34, 0.19],
                _ => [0.12, 0.2, 0.26],
            };
            let cover = room.material(color, 0.0);
            let height = 0.31 + ((i * 3 + tier) % 5) as f32 * 0.04;
            let z = 0.18 + i as f32 * 0.2;
            let y = 0.27 + tier as f32 * 0.61 + height * 0.5;
            room.block([-3.6, y, z], [0.38, height, 0.14], &cover);
            room.block([-3.4, y + height * 0.22, z], [0.012, 0.016, 0.13], brass);
        }
    }
    room.solid([-3.94, 1.3, 1.42], [0.5, 1.3, 1.45]);
}

/// 建立面向 -X 的保險箱與沿 Y 軸旋轉的獨立門鉸鏈。
fn safe(
    room: &mut Furnisher,
    steel: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
    ink: &Handle<StandardMaterial>,
) {
    room.block([3.8, 0.41, -1.65], [1.04, 0.82, 1.1], steel);
    room.block([3.8, 1.12, -1.65], [1.12, 0.62, 1.14], steel);
    room.block([3.19, 1.12, -1.65], [0.035, 0.47, 0.97], ink);
    let pivot = room
        .commands
        .spawn((
            Transform::from_xyz(3.13, 1.12, -2.19),
            Visibility::default(),
            Hinge::Safe,
        ))
        .id();
    let door = room.block([0.0, 0.0, 0.54], [0.1, 0.6, 1.08], steel);
    let dial = room.cylinder([-0.085, 0.0, 0.55], 0.11, 0.05, brass);
    room.commands.entity(dial).insert(
        Transform::from_xyz(-0.085, 0.0, 0.55)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    );
    room.commands.entity(pivot).add_children(&[door, dial]);
    let key = room.block([3.15, 1.0, -1.65], [0.06, 0.06, 0.36], brass);
    room.commands.entity(key).insert((Key, Visibility::Hidden));
    let head = room.ellipsoid([3.15, 1.0, -1.83], [0.05, 0.11, 0.11], brass);
    room.commands.entity(head).insert((Key, Visibility::Hidden));
    room.target([3.04, 1.2, -1.65], Target::Safe);
    room.solid([3.8, 0.72, -1.65], [0.55, 0.72, 0.58]);
    room.light([3.1, 2.65, -1.55], [1.0, 0.65, 0.28], 500.0, false);
    room.block([4.36, 2.45, -1.65], [0.12, 0.5, 0.22], brass);
}

/// 建立南側出口，房門開向走廊，門外光線引導玩家實際走出房間。
fn exit(
    room: &mut Furnisher,
    wood: &Handle<StandardMaterial>,
    walnut: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
    glow: &Handle<StandardMaterial>,
) {
    for x in [0.91, 2.89] {
        room.block([x, 1.28, 4.43], [0.13, 2.56, 0.2], brass);
    }
    room.block([1.9, 2.58, 4.43], [2.1, 0.14, 0.2], brass);
    let pivot = room
        .commands
        .spawn((
            Transform::from_xyz(0.98, 0.0, 4.48),
            Visibility::default(),
            Hinge::Exit,
        ))
        .id();
    let door = room.block([0.92, 1.25, 0.0], [1.84, 2.5, 0.12], wood);
    let panel = room.block([0.92, 1.45, -0.073], [1.52, 1.62, 0.04], walnut);
    let handle = room.block([1.6, 1.12, -0.14], [0.23, 0.055, 0.08], brass);
    room.commands
        .entity(pivot)
        .add_children(&[door, panel, handle]);
    room.target([1.9, 1.35, 4.29], Target::Door);
    room.block([1.9, 0.0, 5.65], [2.0, 0.07, 2.4], wood);
    room.block([1.9, 1.45, 7.0], [2.0, 3.0, 0.1], glow);
    room.light([1.9, 2.0, 5.9], [1.0, 0.79, 0.5], 1800.0, false);
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
