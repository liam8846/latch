use crate::{Screen, Session, bridge};
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use latch::puzzle::{Action, Progress};

pub const START: Vec3 = Vec3::new(0.0, 1.65, 2.8);

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Component)]
pub struct Solid {
    pub center: Vec3,
    pub half: Vec3,
}

impl Solid {
    /// 在公尺制世界 XZ 平面，以玩家半徑 0.23 公尺檢查碰撞。
    fn blocks(self, position: Vec3) -> bool {
        (position.x - self.center.x).abs() < self.half.x + 0.23
            && (position.z - self.center.z).abs() < self.half.z + 0.23
    }

    /// 回傳世界空間射線進入長方體的距離；零方向分量使用平行平面判定。
    pub fn ray_distance(self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let mut near = 0.0_f32;
        let mut far = f32::INFINITY;
        for axis in 0..3 {
            let low = self.center[axis] - self.half[axis];
            let high = self.center[axis] + self.half[axis];
            if direction[axis].abs() < 0.00001 {
                if origin[axis] < low || origin[axis] > high {
                    return None;
                }
            } else {
                let a = (low - origin[axis]) / direction[axis];
                let b = (high - origin[axis]) / direction[axis];
                near = near.max(a.min(b));
                far = far.min(a.max(b));
            }
        }
        (far >= near).then_some(near)
    }
}

/// 建立眼高 1.65 公尺的第一人稱相機；yaw/pitch 使用弧度，零 yaw 朝向 -Z。
pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        bevy::camera::Exposure { ev100: 6.0 },
        Projection::Perspective(PerspectiveProjection {
            fov: 68.0_f32.to_radians(),
            near: 0.05,
            far: 80.0,
            ..default()
        }),
        Transform::from_translation(START),
        Player {
            yaw: 0.0,
            pitch: -0.08,
        },
        Msaa::Off,
    ));
}

/// 依秒步長移動玩家，分軸碰撞可沿牆滑動；焦點離開、模態視窗與暫停時停止輸入。
pub fn move_player(
    time: Res<Time>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    windows: Query<&Window>,
    solids: Query<&Solid>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut session: ResMut<Session>,
) {
    let Ok((mut transform, mut look)) = player.single_mut() else {
        return;
    };
    let focused = windows.single().is_ok_and(|window| window.focused);
    if !focused && session.screen == Screen::Playing {
        session.screen = Screen::Paused;
    }
    if session.screen != Screen::Playing {
        keys.reset_all();
        return;
    }
    let dt = time.delta_secs().min(0.25);
    let axis =
        |positive, negative| f32::from(keys.pressed(positive)) - f32::from(keys.pressed(negative));
    look.yaw -= axis(KeyCode::ArrowRight, KeyCode::ArrowLeft) * dt * 1.5;
    look.pitch += axis(KeyCode::ArrowUp, KeyCode::ArrowDown) * dt * 1.2;
    if bridge::pointer_locked() {
        look.yaw -= mouse.delta.x * 0.0022;
        look.pitch -= mouse.delta.y * 0.0022;
    }
    look.pitch = look.pitch.clamp(-1.25, 1.25);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
    let local = Vec3::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        0.0,
        -axis(KeyCode::KeyW, KeyCode::KeyS),
    );
    let movement = Quat::from_rotation_y(look.yaw) * local.normalize_or_zero() * dt * 2.25;
    let steps = (movement.length() / 0.1).ceil().max(1.0) as u32;
    let movement = movement / steps as f32;
    for displacement in (0..steps).flat_map(|_| [Vec3::X * movement.x, Vec3::Z * movement.z]) {
        let next = transform.translation + displacement;
        let inside_room = next.x.abs() < 4.23
            && next.z > -4.23
            && (next.z < 4.23
                || (session.progress.door_is_open()
                    && (0.95..2.65).contains(&next.x)
                    && next.z < 7.0));
        if inside_room && !solids.iter().any(|solid| solid.blocks(next)) {
            transform.translation = next;
        }
    }
    if session.progress == Progress::DoorOpen && transform.translation.z > 5.65 {
        session.progress.apply(Action::Leave);
        session.screen = Screen::Won;
        session.notify("門外的空氣，終於不再靜止。", "win");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_only_visible_faces_and_handles_parallel_directions() {
        let solid = Solid {
            center: Vec3::ZERO,
            half: Vec3::ONE,
        };
        assert_eq!(
            solid.ray_distance(Vec3::new(0.0, 0.0, 3.0), Vec3::NEG_Z),
            Some(2.0)
        );
        assert_eq!(
            solid.ray_distance(Vec3::new(2.0, 0.0, 3.0), Vec3::NEG_Z),
            None
        );
        assert_eq!(solid.ray_distance(Vec3::new(0.0, 0.0, 3.0), Vec3::Z), None);
        assert_eq!(solid.ray_distance(Vec3::ZERO, Vec3::X), Some(0.0));
        assert!(solid.blocks(Vec3::new(1.2, 1.65, 0.0)));
        assert!(!solid.blocks(Vec3::new(1.3, 1.65, 0.0)));
    }
}
