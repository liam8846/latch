use super::{Hinge, builder::Furnisher, materials::Palette};
use crate::{Session, interaction::Target, physics::world_layers};
use avian3d::prelude::*;
use bevy::prelude::*;
use latch::signal::{COLUMNS, Cell, Mirror, Orientation, ROWS, Signal, SignalBoard};
use std::f32::consts::FRAC_PI_4;

const PANEL_Z: f32 = 23.40;
const CELL_SIZE: f32 = 0.36;
const GATE_X: f32 = -1.85;

pub const ENTRY_Z: f32 = 16.9;
pub const EXIT_Z: f32 = 25.0;

#[derive(Component)]
pub struct MirrorDial(Mirror);

#[derive(Component)]
pub struct Receiver(Signal);

#[derive(Component)]
pub struct Beam;

#[derive(Resource)]
pub struct SignalDisplay {
    cube: Handle<Mesh>,
    beams: [Handle<StandardMaterial>; 2],
    dormant: [Handle<StandardMaterial>; 2],
}

impl SignalDisplay {
    /// 回傳指定光路的顯示材質，辨識色和接收窗共用同一組色彩。
    fn lens(&self, signal: Signal, lit: bool) -> Handle<StandardMaterial> {
        let materials = if lit { &self.beams } else { &self.dormant };
        materials[signal as usize].clone()
    }
}

/// 建立潮汐信號室、通往岸邊的門和共用一組鏡片的雙光路面板。
pub(super) fn setup(room: &mut Furnisher, palette: &Palette) {
    shell(room, palette);
    gate(room, palette);
    board(room, palette);
}

/// 由面板格座標取得 3D 世界座標；玩家由北往南看，面板右方對應世界 -X。
fn panel_position(column: f32, row: f32, z: f32) -> Vec3 {
    Vec3::new(
        1.55 + (3.0 - column) * CELL_SIZE,
        1.87 + (2.5 - row) * CELL_SIZE,
        z,
    )
}

/// 將整數格座標投影至指定深度的面板平面。
fn point(cell: Cell, z: f32) -> Vec3 {
    panel_position(f32::from(cell.0), f32::from(cell.1), z)
}

/// 建立北側木廊出口、石質房間、牆面金屬結構與連通岸邊的小平台。
fn shell(room: &mut Furnisher, palette: &Palette) {
    room.solid_block([0.5, -0.10, 20.1], [7.8, 0.2, 7.1], &palette.steel);
    room.block([0.5, 0.011, 20.1], [7.6, 0.024, 7.1], &palette.wood);
    room.solid_block([0.5, 3.76, 20.1], [7.8, 0.16, 7.1], &palette.trim);
    for x in [-3.3, 4.3] {
        room.solid_block([x, 1.87, 20.1], [0.2, 3.74, 7.0], &palette.wall);
        for z in [17.35, 19.6, 21.85] {
            room.block(
                [x.signum().mul_add(-0.13, x), 1.83, z],
                [0.17, 3.65, 0.15],
                &palette.steel,
            );
        }
    }
    for x in [-1.95, 2.95] {
        room.solid_block([x, 1.87, 16.7], [2.7, 3.74, 0.2], &palette.wall);
    }
    room.solid_block([0.5, 3.15, 16.7], [2.2, 1.18, 0.2], &palette.wall);
    room.solid_block([-3.02, 1.87, 23.6], [0.56, 3.74, 0.2], &palette.wall);
    room.solid_block([1.68, 1.87, 23.6], [5.30, 3.74, 0.2], &palette.wall);
    room.solid_block([GATE_X, 3.10, 23.6], [1.78, 1.28, 0.2], &palette.wall);
    room.solid_block([GATE_X, -0.10, 24.86], [2.10, 0.2, 2.96], &palette.wood);
    for x in [GATE_X - 1.0, GATE_X + 1.0] {
        room.solid_block([x, 1.28, 24.9], [0.17, 2.56, 2.72], &palette.wall);
    }
    room.solid_block([GATE_X, 1.75, 26.27], [2.2, 3.5, 0.16], &palette.moon);
    room.light([GATE_X, 2.35, 25.4], [0.34, 0.65, 1.0], 640.0, false);
    room.light([0.5, 2.95, 18.0], [0.41, 0.67, 0.91], 1050.0, false);
    room.light([1.55, 2.95, 21.55], [0.90, 0.68, 0.41], 1000.0, false);
    for x in [-2.53, -1.17] {
        for i in 0..14 {
            room.block(
                [x, 0.034, 18.2 + i as f32 * 0.37],
                [0.055, 0.016, 0.26],
                &palette.brass,
            );
        }
    }
}

/// 將帶碰撞的出口閘門掛在左側門軸，通電後朝岸邊開啟。
fn gate(room: &mut Furnisher, palette: &Palette) {
    for x in [GATE_X - 0.91, GATE_X + 0.91] {
        room.block([x, 1.28, 23.51], [0.15, 2.56, 0.26], &palette.brass);
    }
    room.block([GATE_X, 2.52, 23.51], [1.92, 0.16, 0.24], &palette.brass);
    let pivot = room
        .commands
        .spawn((
            Transform::from_xyz(GATE_X - 0.89, 0.0, 23.56),
            Visibility::default(),
            RigidBody::Kinematic,
            Hinge::SignalGate,
            Target::SignalGate,
        ))
        .id();
    let slab = room.block([0.89, 1.23, 0.0], [1.78, 2.46, 0.15], &palette.steel);
    room.commands
        .entity(slab)
        .insert((Collider::cuboid(1.0, 1.0, 1.0), world_layers()));
    room.commands.entity(pivot).add_child(slab);
    for y in [0.34, 1.18, 2.13] {
        let band = room.block([0.89, y, -0.085], [1.62, 0.08, 0.055], &palette.brass);
        room.commands.entity(pivot).add_child(band);
    }
    for x in [0.40, 0.89, 1.38] {
        let slat = room.block([x, 1.25, -0.082], [0.055, 1.76, 0.04], &palette.brass);
        room.commands.entity(pivot).add_child(slat);
    }
}

/// 建立刻線、圓形發射燈、方框接收窗、可轉動的鏡片和一旁的操作牌。
fn board(room: &mut Furnisher, palette: &Palette) {
    let base = room.material([0.16, 0.22, 0.25], 0.68);
    let rail = room.material([0.31, 0.39, 0.41], 0.56);
    let mirror_face = room.material([0.83, 0.88, 0.88], 0.66);
    let display = SignalDisplay {
        cube: room.cube.clone(),
        beams: [[1.0, 0.57, 0.16], [0.25, 0.74, 1.0]].map(|[red, green, blue]| {
            room.materials.add(StandardMaterial {
                base_color: Color::srgb(red, green, blue),
                unlit: true,
                ..default()
            })
        }),
        dormant: [[0.40, 0.19, 0.045], [0.06, 0.24, 0.36]].map(|rgb| room.material(rgb, 0.35)),
    };
    room.block(
        [1.55, 1.87, PANEL_Z + 0.08],
        [3.38, 2.98, 0.15],
        &palette.brass,
    );
    room.block([1.55, 1.87, PANEL_Z + 0.03], [3.28, 2.88, 0.15], &base);
    for row in 0..ROWS {
        room.block(
            panel_position(3.0, f32::from(row), PANEL_Z - 0.052).to_array(),
            [2.88, 0.012, 0.014],
            &rail,
        );
    }
    for column in 0..COLUMNS {
        room.block(
            panel_position(f32::from(column), 2.5, PANEL_Z - 0.052).to_array(),
            [0.012, 2.52, 0.014],
            &rail,
        );
    }
    for mirror in Mirror::ALL {
        let at = point(mirror.cell(), PANEL_Z - 0.07);
        room.block(at.to_array(), [0.25, 0.25, 0.032], &palette.walnut);
        let dial = room.block(
            (at - Vec3::Z * 0.036).to_array(),
            [0.225, 0.046, 0.034],
            &mirror_face,
        );
        room.commands.entity(dial).insert(MirrorDial(mirror));
        room.sensor(
            (at - Vec3::Z * 0.085).to_array(),
            [0.29, 0.29, 0.14],
            Target::SignalMirror(mirror),
        );
    }
    for signal in Signal::ALL {
        let source = point(signal.source(), PANEL_Z - 0.08);
        room.ellipsoid(source.to_array(), [0.12, 0.12, 0.044], &palette.brass);
        room.ellipsoid(
            (source - Vec3::Z * 0.038).to_array(),
            [0.086, 0.086, 0.033],
            &display.lens(signal, true),
        );
        let receiver = point(signal.receiver(), PANEL_Z - 0.08);
        room.block(receiver.to_array(), [0.23, 0.23, 0.08], &palette.brass);
        let glass = room.block(
            (receiver - Vec3::Z * 0.05).to_array(),
            [0.166, 0.166, 0.036],
            &display.lens(signal, false),
        );
        room.commands.entity(glass).insert(Receiver(signal));
    }
    let plaque = Vec3::new(-0.40, 1.45, 23.43);
    room.block(plaque.to_array(), [0.54, 0.57, 0.09], &palette.brass);
    let face = plaque - Vec3::Z * 0.051;
    room.block(face.to_array(), [0.46, 0.49, 0.022], &palette.cream);
    for y in [-0.13, -0.04, 0.05, 0.14] {
        room.block(
            (face + Vec3::new(0.0, y, -0.016)).to_array(),
            [0.29, 0.017, 0.008],
            &palette.ink,
        );
    }
    room.sensor(
        (plaque - Vec3::Z * 0.075).to_array(),
        [0.56, 0.59, 0.14],
        Target::SignalManual,
    );
    room.commands.insert_resource(display);
}

/// 每次檔位改變時更新面板符號、投射兩條光線，並點亮各自有收到訊號的方窗。
pub fn show_signals(
    mut commands: Commands,
    session: Res<Session>,
    display: Res<SignalDisplay>,
    mut previous: Local<Option<SignalBoard>>,
    mut mirrors: Query<(&MirrorDial, &mut Transform)>,
    mut receivers: Query<(&Receiver, &mut MeshMaterial3d<StandardMaterial>)>,
    beams: Query<Entity, With<Beam>>,
) {
    let board = session.signal_board;
    if *previous == Some(board) {
        return;
    }
    *previous = Some(board);
    for (mirror, mut transform) in &mut mirrors {
        let angle = match board.orientation(mirror.0) {
            Orientation::Clear => 0.0,
            Orientation::Slash => -FRAC_PI_4,
            Orientation::Backslash => FRAC_PI_4,
        };
        transform.rotation = Quat::from_rotation_z(angle);
    }
    for entity in &beams {
        commands.entity(entity).despawn();
    }
    for signal in Signal::ALL {
        let route = board.trace(signal);
        for (receiver, mut lens) in &mut receivers {
            if receiver.0 == signal {
                lens.0 = display.lens(signal, route.received);
            }
        }
        for pair in route.points.windows(2) {
            let (z, width) = match signal {
                Signal::Amber => (PANEL_Z - 0.116, 0.030),
                Signal::Blue => (PANEL_Z - 0.122, 0.016),
            };
            let start = point(pair[0], z);
            let end = point(pair[1], z);
            let delta = end - start;
            commands.spawn((
                Beam,
                Mesh3d(display.cube.clone()),
                MeshMaterial3d(display.lens(signal, true)),
                Transform::from_translation(start.lerp(end, 0.5))
                    .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x)))
                    .with_scale(Vec3::new(delta.length() + width, width, 0.004)),
            ));
        }
    }
}
