use super::{Hinge, Key, builder::Furnisher};
use crate::{
    interaction::Target,
    physics::{
        Drawer, Grabbable, LatchBolt, PressurePlate, PropKind, Pushable, furniture_layers,
        prop_layers, world_layers,
    },
};
use avian3d::prelude::*;
use bevy::prelude::*;

/// 建立西側書架與有節奏的書脊，維持低面數以兼顧網頁 WebGPU 效能。
pub(super) fn shelves(
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
    let shelf_box = room.static_box([-3.94, 1.3, 1.42], [1.0, 2.6, 2.9]);
    room.commands.entity(shelf_box).insert(Target::Bookshelf);
}

/// 建立可沿 +Z 滑動的書桌抽屜，並在槽內放置可拿取的 2.2 公斤黃銅鎮紙。
pub(super) fn desk_drawer(
    room: &mut Furnisher,
    walnut: &Handle<StandardMaterial>,
    wood: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
) {
    let closed = Vec3::new(-1.45, 0.69, -2.86);
    let drawer = room
        .commands
        .spawn((
            Transform::from_translation(closed),
            Visibility::default(),
            RigidBody::Kinematic,
            Drawer {
                open: false,
                closed_position: closed,
                open_offset: Vec3::new(0.0, 0.0, 0.58),
                progress: 0.0,
            },
        ))
        .id();
    let bottom = room.block([0.0, -0.14, 0.0], [0.69, 0.04, 0.88], walnut);
    room.commands
        .entity(bottom)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let front = room.block([0.0, 0.0, 0.455], [0.71, 0.30, 0.03], wood);
    room.commands
        .entity(front)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let back = room.block([0.0, -0.02, -0.425], [0.69, 0.24, 0.03], walnut);
    room.commands
        .entity(back)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let left = room.block([-0.33, -0.02, 0.0], [0.03, 0.24, 0.88], walnut);
    room.commands
        .entity(left)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let right = room.block([0.33, -0.02, 0.0], [0.03, 0.24, 0.88], walnut);
    room.commands
        .entity(right)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let handle = room.block([0.0, 0.0, 0.495], [0.21, 0.025, 0.04], brass);
    room.commands
        .entity(drawer)
        .add_children(&[bottom, front, back, left, right, handle]);

    let weight_home = Transform::from_xyz(-1.45, 0.625, -2.62);
    let weight = room
        .commands
        .spawn((
            weight_home,
            Visibility::default(),
            RigidBody::Dynamic,
            SleepingDisabled,
            Collider::cuboid(0.18, 0.10, 0.14),
            Mass(PropKind::Paperweight.mass_kg()),
            LinearDamping(0.7),
            AngularDamping(1.4),
            prop_layers(),
            Grabbable {
                kind: PropKind::Paperweight,
                home: weight_home,
            },
        ))
        .id();
    let base = room.block([0.0, 0.0, 0.0], [0.18, 0.10, 0.14], brass);
    let ridge = room.block([0.0, 0.058, 0.0], [0.12, 0.02, 0.085], brass);
    let knob = room.cylinder([0.0, 0.085, 0.0], 0.026, 0.038, brass);
    room.commands
        .entity(weight)
        .add_children(&[base, ridge, knob]);
}

/// 建立可推動的復古木質書桌椅；鎖定水平翻倒軸讓椅子能平穩在地板上滑動與旋轉。
pub(super) fn chair(room: &mut Furnisher, assets: &AssetServer) {
    let home = Transform::from_xyz(-0.65, 0.015, -1.62);
    let root = room
        .commands
        .spawn((
            home,
            Visibility::default(),
            RigidBody::Dynamic,
            SleepingDisabled,
            Mass(5.0),
            Friction::new(0.15),
            LinearDamping(3.0),
            AngularDamping(4.0),
            LockedAxes::new().lock_rotation_x().lock_rotation_z(),
            furniture_layers(),
            Pushable { home },
        ))
        .id();
    let mesh = room.model(
        assets,
        "models/painted_wooden_chair_01/painted_wooden_chair_01_1k.gltf",
        [0.0, 0.0, 0.0],
        std::f32::consts::PI,
        1.22,
    );
    let seat_collider = room
        .commands
        .spawn((
            Transform::from_xyz(0.0, 0.30, 0.0),
            Collider::cuboid(0.58, 0.60, 0.68),
            furniture_layers(),
        ))
        .id();
    let back_collider = room
        .commands
        .spawn((
            Transform::from_xyz(0.0, 0.90, 0.26),
            Collider::cuboid(0.58, 0.65, 0.14),
            furniture_layers(),
        ))
        .id();
    room.commands
        .entity(root)
        .add_children(&[mesh, seat_collider, back_collider]);
}

/// 建立面向 -X 的保險箱、沿 Y 軸旋轉的碰撞門鉸鏈與連動向東側配重托盤的暗扣栓。
pub(super) fn safe(
    room: &mut Furnisher,
    steel: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
    ink: &Handle<StandardMaterial>,
) {
    room.solid_block([3.8, 0.41, -1.65], [1.04, 0.82, 1.1], steel);
    let upper = room.solid_block([3.8, 1.12, -1.65], [1.12, 0.62, 1.14], steel);
    room.commands.entity(upper).insert(Target::Safe);
    room.block([3.19, 1.12, -1.65], [0.035, 0.47, 0.97], ink);
    let pivot = room
        .commands
        .spawn((
            Transform::from_xyz(3.13, 1.12, -2.19),
            Visibility::default(),
            RigidBody::Kinematic,
            Hinge::Safe,
            Target::Safe,
        ))
        .id();
    let door = room.block([0.0, 0.0, 0.54], [0.1, 0.6, 1.08], steel);
    room.commands
        .entity(door)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let dial = room.cylinder([-0.085, 0.0, 0.55], 0.11, 0.05, brass);
    room.commands.entity(dial).insert(
        Transform::from_xyz(-0.085, 0.0, 0.55)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
    );
    room.commands.entity(pivot).add_children(&[door, dial]);
    let bolt = room.block([3.07, 1.12, -1.12], [0.07, 0.34, 0.07], brass);
    room.commands.entity(bolt).insert(LatchBolt {
        locked_y: 1.12,
        released_y: 1.50,
    });
    let key = room.block([3.15, 1.0, -1.65], [0.06, 0.06, 0.36], brass);
    room.commands.entity(key).insert((Key, Visibility::Hidden));
    let head = room.ellipsoid([3.15, 1.0, -1.83], [0.05, 0.11, 0.11], brass);
    room.commands.entity(head).insert((Key, Visibility::Hidden));
    room.light([3.1, 2.65, -1.55], [1.0, 0.65, 0.28], 500.0, false);
    room.block([4.36, 2.45, -1.65], [0.12, 0.5, 0.22], brass);
}

/// 建立東側復古木桌與黃銅配重托盤，透過牆面黃銅連桿視覺連結保險箱暗扣。
pub(super) fn pressure_plate(
    room: &mut Furnisher,
    assets: &AssetServer,
    walnut: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
) {
    room.model(
        assets,
        "models/small_wooden_table_01/small_wooden_table_01_1k.gltf",
        [3.8, 0.015, 0.25],
        std::f32::consts::FRAC_PI_2,
        1.48,
    );
    room.static_box([3.8, 0.40, 0.25], [0.70, 0.80, 1.32]);
    room.block([3.8, 0.815, 0.25], [0.58, 0.02, 0.58], walnut);
    room.block([4.42, 0.84, -0.45], [0.05, 0.05, 1.48], brass);
    let plate = room.block([3.8, 0.845, 0.25], [0.52, 0.05, 0.52], brass);
    room.commands.entity(plate).insert((
        RigidBody::Kinematic,
        Collider::cuboid(1.0, 1.0, 1.0),
        world_layers(),
        Target::Plate,
        PressurePlate {
            center: Vec3::new(3.8, 0.845, 0.25),
            half: Vec3::new(0.26, 0.05, 0.26),
            base_y: 0.845,
            travel: 0.035,
        },
    ));
}

/// 建立南側出口，房門附帶隨鉸鏈旋轉的碰撞體，解鎖後可實際走出房間。
pub(super) fn exit(
    room: &mut Furnisher,
    wood: &Handle<StandardMaterial>,
    walnut: &Handle<StandardMaterial>,
    brass: &Handle<StandardMaterial>,
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
            RigidBody::Kinematic,
            Hinge::Exit,
            Target::Door,
        ))
        .id();
    let door = room.block([0.92, 1.25, 0.0], [1.84, 2.5, 0.12], wood);
    room.commands
        .entity(door)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    let panel = room.block([0.92, 1.45, -0.073], [1.52, 1.62, 0.04], walnut);
    let handle = room.block([1.6, 1.12, -0.14], [0.23, 0.055, 0.08], brass);
    room.commands
        .entity(pivot)
        .add_children(&[door, panel, handle]);
    room.block([1.9, 0.0, 5.65], [2.0, 0.07, 2.4], wood);
    let night = room.material([0.025, 0.045, 0.08], 0.0);
    room.solid_block([1.9, 1.45, 7.0], [2.0, 3.0, 0.1], &night);
    room.static_box([0.85, 1.5, 5.8], [0.2, 3.0, 2.6]);
    room.static_box([2.95, 1.5, 5.8], [0.2, 3.0, 2.6]);
    room.light([1.9, 2.0, 5.9], [0.55, 0.66, 1.0], 480.0, false);
}
