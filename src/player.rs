use crate::{
    Screen, Session, bridge,
    physics::{GameLayer, Pushable},
    session::SoundCue,
};
use avian3d::prelude::*;
use bevy::{ecs::system::SystemParam, input::mouse::AccumulatedMouseMotion, prelude::*};
use latch::{
    i18n::MessageKey,
    puzzle::{Action, Progress},
};

pub const START: Vec3 = Vec3::new(0.0, 1.65, 2.8);
const PLAYER_RADIUS: f32 = 0.24;
const PLAYER_CYLINDER_HEIGHT: f32 = 1.12;
const BODY_Y_OFFSET: f32 = -0.70;
const COLLISION_SKIN: f32 = 0.02;

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
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

#[derive(SystemParam)]
pub struct MoveInput<'w, 's> {
    time: Res<'w, Time>,
    keys: ResMut<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, AccumulatedMouseMotion>,
    windows: Query<'w, 's, &'static Window>,
}

/// 以 Avian 膠囊體掃描移動玩家、沿牆滑動並推動可移動家具；焦點離開或暫停時停止輸入。
pub fn move_player(
    mut input: MoveInput,
    spatial: SpatialQuery,
    parents: Query<&ChildOf>,
    mut pushables: Query<&mut LinearVelocity, With<Pushable>>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut session: ResMut<Session>,
) {
    let Ok((mut transform, mut look)) = player.single_mut() else {
        return;
    };
    let focused = bridge::window_focused(input.windows.single().is_ok_and(|window| window.focused));
    if !focused && session.screen == Screen::Playing {
        session.screen = Screen::Paused;
    }
    if session.screen != Screen::Playing || !bridge::input_focused() {
        input.keys.reset_all();
        return;
    }
    let dt = input.time.delta_secs().min(0.25);
    let axis = |positive, negative| {
        f32::from(input.keys.pressed(positive)) - f32::from(input.keys.pressed(negative))
    };
    look.yaw -= axis(KeyCode::ArrowRight, KeyCode::ArrowLeft) * dt * 1.5;
    look.pitch += axis(KeyCode::ArrowUp, KeyCode::ArrowDown) * dt * 1.2;
    if bridge::pointer_locked() {
        look.yaw -= input.mouse.delta.x * 0.0022;
        look.pitch -= input.mouse.delta.y * 0.0022;
    }
    look.pitch = look.pitch.clamp(-1.25, 1.25);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
    let local = Vec3::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        0.0,
        -axis(KeyCode::KeyW, KeyCode::KeyS),
    );
    let mut remaining = Quat::from_rotation_y(look.yaw) * local.normalize_or_zero() * dt * 2.25;
    let shape = Collider::capsule(PLAYER_RADIUS, PLAYER_CYLINDER_HEIGHT);
    let filter = SpatialQueryFilter::from_mask([GameLayer::World, GameLayer::Furniture]);
    for _ in 0..3 {
        let distance = remaining.length();
        if distance < 1e-5 {
            break;
        }
        let Ok(direction) = Dir3::new(remaining / distance) else {
            break;
        };
        let origin = transform.translation + Vec3::Y * BODY_Y_OFFSET;
        let config = ShapeCastConfig::from_max_distance(distance + COLLISION_SKIN);
        let Some(hit) =
            spatial.cast_shape(&shape, origin, Quat::IDENTITY, direction, &config, &filter)
        else {
            transform.translation += remaining;
            break;
        };
        let mut current = Some(hit.entity);
        while let Some(entity) = current {
            if let Ok(mut velocity) = pushables.get_mut(entity) {
                let push = Vec3::new(direction.x, 0.0, direction.z) * 2.4;
                velocity.0.x = velocity.0.x.lerp(push.x, 0.65);
                velocity.0.z = velocity.0.z.lerp(push.z, 0.65);
                break;
            }
            current = parents.get(entity).ok().map(ChildOf::parent);
        }
        let travel = (hit.distance - COLLISION_SKIN).max(0.0);
        transform.translation += *direction * travel;
        let leftover = remaining - *direction * travel;
        let mut normal = Vec3::new(hit.normal1.x, 0.0, hit.normal1.z);
        if normal.length_squared() < 1e-5 {
            break;
        }
        normal = normal.normalize();
        remaining = leftover - normal * leftover.dot(normal);
    }
    if session.progress == Progress::DoorOpen && transform.translation.z > 5.65 {
        session.progress.apply(Action::Leave);
        session.screen = Screen::Won;
        session.notify(MessageKey::Escaped, SoundCue::Win);
    }
}
