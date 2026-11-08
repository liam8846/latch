use super::{Hinge, Key, builder::Furnisher};
use crate::interaction::Target;
use bevy::prelude::*;

/// 建立西側書架與有節奏的書脊；不依賴任何外部模型或貼圖。
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
    room.solid([-3.94, 1.3, 1.42], [0.5, 1.3, 1.45]);
}

/// 建立面向 -X 的保險箱與沿 Y 軸旋轉的獨立門鉸鏈。
pub(super) fn safe(
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
    let night = room.material([0.025, 0.045, 0.08], 0.0);
    room.block([1.9, 1.45, 7.0], [2.0, 3.0, 0.1], &night);
    room.light([1.9, 2.0, 5.9], [0.55, 0.66, 1.0], 480.0, false);
}
