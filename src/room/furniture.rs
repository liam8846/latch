use super::{BalanceBeam, BalanceCounterPan, Hinge, Key, builder::Furnisher, materials::Palette};
use crate::{
    interaction::Target,
    physics::{
        Drawer, Grabbable, LatchBolt, PlateKind, PressurePlate, PropKind, Pushable,
        furniture_layers, prop_layers, world_layers,
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
    let covers = [
        room.material([0.22, 0.31, 0.26], 0.0),
        room.material([0.38, 0.15, 0.09], 0.0),
        room.material([0.45, 0.34, 0.19], 0.0),
        room.material([0.12, 0.2, 0.26], 0.0),
    ];
    for tier in 0..3 {
        for i in 0..13 {
            let cover = &covers[i % 4];
            let height = 0.31 + ((i * 3 + tier) % 5) as f32 * 0.04;
            let z = 0.18 + i as f32 * 0.2;
            let y = 0.27 + tier as f32 * 0.61 + height * 0.5;
            room.block([-3.6, y, z], [0.38, height, 0.14], cover);
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
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(0.11, 0.05, 0.11)),
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

/// 建立東側書信郵資秤重角：復古黃銅雙盤天平、標準砝碼木盒、貼郵票信封與牆面郵資表。
pub(super) fn balance_corner(room: &mut Furnisher, assets: &AssetServer, palette: &Palette) {
    let (walnut, wood, brass, cream, ink, rust) = (
        &palette.walnut,
        &palette.wood,
        &palette.brass,
        &palette.cream,
        &palette.ink,
        &palette.rust,
    );
    room.model(
        assets,
        "models/small_wooden_table_01/small_wooden_table_01_1k.gltf",
        [3.8, 0.015, 0.25],
        std::f32::consts::FRAC_PI_2,
        1.48,
    );
    room.static_box([3.8, 0.40, 0.25], [0.70, 0.80, 1.32]);
    room.block([4.42, 0.84, -0.45], [0.05, 0.05, 1.48], brass);

    room.block([3.80, 0.822, 0.14], [0.24, 0.044, 0.54], walnut);
    room.block([3.80, 0.848, 0.14], [0.26, 0.012, 0.56], brass);
    room.cylinder([3.80, 1.07, 0.14], 0.022, 0.45, brass);
    room.cylinder([3.80, 0.87, 0.14], 0.06, 0.04, brass);
    room.ellipsoid([3.80, 1.32, 0.14], [0.032, 0.042, 0.032], brass);

    let beam = room
        .commands
        .spawn((
            Transform::from_xyz(3.80, 1.28, 0.14).with_rotation(Quat::from_rotation_x(0.18)),
            Visibility::default(),
            BalanceBeam,
        ))
        .id();
    let arm = room.block([0.0, 0.0, 0.0], [0.026, 0.028, 0.58], brass);
    let pointer = room.block([0.0, -0.055, 0.0], [0.012, 0.10, 0.012], brass);
    let north_cap = room.ellipsoid([0.0, 0.0, -0.28], [0.02, 0.02, 0.02], brass);
    let south_cap = room.ellipsoid([0.0, 0.0, 0.28], [0.02, 0.02, 0.02], brass);
    room.commands
        .entity(beam)
        .add_children(&[arm, pointer, north_cap, south_cap]);

    let north_pan = room
        .commands
        .spawn((
            Transform::from_xyz(3.80, 0.92, -0.14),
            Visibility::default(),
            BalanceCounterPan {
                low_y: 0.92,
                high_y: 1.01,
            },
        ))
        .id();
    let north_dish = room.cylinder([0.0, 0.0, 0.0], 0.135, 0.022, brass);
    let north_weight = room.cylinder([0.0, 0.058, 0.0], 0.055, 0.095, brass);
    let north_knob = room.cylinder([0.0, 0.118, 0.0], 0.02, 0.028, brass);
    let north_rod_a = room.cylinder([-0.08, 0.16, 0.0], 0.005, 0.30, brass);
    let north_rod_b = room.cylinder([0.08, 0.16, 0.0], 0.005, 0.30, brass);
    room.commands.entity(north_pan).add_children(&[
        north_dish,
        north_weight,
        north_knob,
        north_rod_a,
        north_rod_b,
    ]);

    let south_pan = room
        .commands
        .spawn((
            Transform::from_xyz(3.80, 1.00, 0.42),
            Visibility::default(),
            RigidBody::Kinematic,
            Collider::cuboid(0.32, 0.04, 0.32),
            world_layers(),
            Target::Plate,
            PressurePlate {
                kind: PlateKind::StudyBalance,
                center: Vec3::new(3.80, 1.00, 0.42),
                half: Vec3::new(0.18, 0.06, 0.18),
                base_y: 1.00,
                travel: 0.075,
            },
        ))
        .id();
    let south_dish = room.cylinder([0.0, 0.0, 0.0], 0.155, 0.024, brass);
    let south_rim = room.cylinder([0.0, 0.015, 0.0], 0.162, 0.012, brass);
    let south_rod_a = room.cylinder([-0.09, 0.145, 0.0], 0.005, 0.28, brass);
    let south_rod_b = room.cylinder([0.09, 0.145, 0.0], 0.005, 0.28, brass);
    room.commands.entity(south_pan).add_children(&[
        south_dish,
        south_rim,
        south_rod_a,
        south_rod_b,
    ]);

    room.block([3.78, 0.832, 0.74], [0.22, 0.06, 0.16], walnut);
    room.block([3.78, 0.864, 0.74], [0.19, 0.008, 0.13], ink);
    room.cylinder([3.72, 0.882, 0.74], 0.016, 0.036, brass);
    room.cylinder([3.77, 0.888, 0.74], 0.021, 0.048, brass);
    room.sensor([3.78, 0.86, 0.74], [0.26, 0.14, 0.20], Target::WeightBox);

    for (i, angle) in [(0, 0.08_f32), (1, -0.06), (2, 0.03)] {
        let y = 0.808 + i as f32 * 0.012;
        let env = room.block([3.78, y, -0.32], [0.22, 0.01, 0.14], cream);
        room.commands.entity(env).insert(
            Transform::from_xyz(3.78, y, -0.32)
                .with_scale(Vec3::new(0.22, 0.01, 0.14))
                .with_rotation(Quat::from_rotation_y(angle)),
        );
    }
    room.block([3.71, 0.838, -0.28], [0.032, 0.004, 0.038], rust);
    room.block([3.65, 0.815, -0.18], [0.022, 0.022, 0.12], rust);
    room.cylinder([3.62, 0.838, -0.26], 0.018, 0.068, brass);
    room.sensor([3.74, 0.84, -0.28], [0.34, 0.12, 0.28], Target::MailStack);

    room.block([4.45, 1.44, 0.22], [0.03, 0.42, 0.56], walnut);
    room.block([4.43, 1.44, 0.22], [0.016, 0.36, 0.48], cream);
    for row in 0..5 {
        room.block(
            [4.418, 1.56 - row as f32 * 0.058, 0.22],
            [0.004, 0.008, 0.36],
            ink,
        );
    }
    room.block([4.42, 1.63, 0.22], [0.02, 0.025, 0.52], wood);
    room.sensor([4.40, 1.44, 0.22], [0.14, 0.46, 0.60], Target::PostageChart);
}

/// 建立南側書房門，房門附帶隨鉸鏈旋轉的碰撞體，解鎖後可無縫走進玄關長廊。
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
}
