use crate::{Screen, Session, player::Player, session::SoundCue};
use avian3d::prelude::*;
use bevy::prelude::*;
use latch::{
    i18n::MessageKey,
    puzzle::{Action, Progress, Reply, plate_triggered},
};

#[derive(PhysicsLayer, Default, Clone, Copy, Debug)]
pub enum GameLayer {
    #[default]
    World,
    Player,
    Prop,
    Furniture,
    Interactable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Paperweight,
    Book,
    Box,
    Vase,
}

impl PropKind {
    /// 回傳各物理道具的品質（公斤），供配重托盤判定與剛體質量共用。
    pub fn mass_kg(self) -> f32 {
        match self {
            Self::Paperweight => 2.2,
            Self::Vase => 1.2,
            Self::Book => 0.8,
            Self::Box => 0.5,
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// 準星對準可拿取道具時顯示的動作文案。
    pub fn pick_label(self) -> MessageKey {
        match self {
            Self::Paperweight => MessageKey::PickPaperweight,
            Self::Book => MessageKey::PickBook,
            Self::Box => MessageKey::PickBox,
            Self::Vase => MessageKey::PickVase,
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// 手中持有道具時顯示的放下文案。
    pub fn drop_label(self) -> MessageKey {
        match self {
            Self::Paperweight => MessageKey::DropPaperweight,
            Self::Book => MessageKey::DropBook,
            Self::Box => MessageKey::DropBox,
            Self::Vase => MessageKey::DropVase,
        }
    }
}

#[derive(Component)]
pub struct Grabbable {
    pub kind: PropKind,
    pub home: Transform,
}

#[derive(Component)]
pub struct Pushable {
    pub home: Transform,
}

#[derive(Component)]
pub struct Drawer {
    pub open: bool,
    pub closed_position: Vec3,
    pub open_offset: Vec3,
    pub progress: f32,
}

#[derive(Component)]
pub struct PressurePlate {
    pub center: Vec3,
    pub half: Vec3,
    pub base_y: f32,
    pub travel: f32,
}

#[derive(Component)]
pub struct LatchBolt {
    pub locked_y: f32,
    pub released_y: f32,
}

#[derive(Resource, Default)]
pub struct HeldProp(pub Option<(Entity, PropKind)>);

#[derive(Resource, Default)]
pub struct PhysicsReset(pub bool);

pub type PropControlQuery<'w, 's> =
    Query<'w, 's, (&'static mut LinearVelocity, &'static mut AngularVelocity), With<Grabbable>>;

pub type HeldPropQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Transform,
        &'static mut LinearVelocity,
        &'static mut AngularVelocity,
    ),
    (With<Grabbable>, Without<Player>),
>;

type PlatePropQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static Grabbable),
    (Without<PressurePlate>, Without<LatchBolt>),
>;

type ResetPropQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Grabbable,
        &'static mut Transform,
        &'static mut Position,
        &'static mut Rotation,
        &'static mut LinearVelocity,
        &'static mut AngularVelocity,
    ),
    (Without<Pushable>, Without<Drawer>),
>;

type ResetFurnitureQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Pushable,
        &'static mut Transform,
        &'static mut Position,
        &'static mut Rotation,
        &'static mut LinearVelocity,
        &'static mut AngularVelocity,
    ),
    (Without<Grabbable>, Without<Drawer>),
>;

type ResetDrawerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Drawer,
        &'static mut Transform,
        &'static mut Position,
    ),
    (Without<Grabbable>, Without<Pushable>),
>;

/// 靜態或運動學結構的碰撞層：阻擋玩家、道具與可推家具。
pub fn world_layers() -> CollisionLayers {
    CollisionLayers::new(
        GameLayer::World,
        [GameLayer::Player, GameLayer::Prop, GameLayer::Furniture],
    )
}

/// 可拿取動態道具的碰撞層：與環境、道具與家具互撞，不頂飛玩家。
pub fn prop_layers() -> CollisionLayers {
    CollisionLayers::new(
        GameLayer::Prop,
        [GameLayer::World, GameLayer::Prop, GameLayer::Furniture],
    )
}

/// 可推動家具的碰撞層：與牆壁、地板、道具與玩家互動。
pub fn furniture_layers() -> CollisionLayers {
    CollisionLayers::new(
        GameLayer::Furniture,
        [
            GameLayer::World,
            GameLayer::Prop,
            GameLayer::Furniture,
            GameLayer::Player,
        ],
    )
}

/// 純射線互動感應區的碰撞層，不產生實體阻擋。
pub fn interactable_layers() -> CollisionLayers {
    CollisionLayers::new(GameLayer::Interactable, LayerMask::NONE)
}

/// 拿起指定動態剛體並暫時關閉重力與實體碰撞，確保物件能順暢離開抽屜並跟隨視線。
pub fn pick_up(
    commands: &mut Commands,
    held: &mut HeldProp,
    entity: Entity,
    kind: PropKind,
    props: &mut PropControlQuery,
) {
    if let Ok((mut linear, mut angular)) = props.get_mut(entity) {
        linear.0 = Vec3::ZERO;
        angular.0 = Vec3::ZERO;
        commands
            .entity(entity)
            .insert((GravityScale(0.0), CollisionLayers::NONE));
        held.0 = Some((entity, kind));
    }
}

/// 放下目前持有的動態剛體並恢復重力與碰撞，限制殘餘速度避免穿透牆面。
pub fn drop_held(commands: &mut Commands, held: &mut HeldProp, props: &mut PropControlQuery) {
    let Some((entity, _)) = held.0.take() else {
        return;
    };
    if let Ok((mut linear, mut angular)) = props.get_mut(entity) {
        linear.0 = linear.0.clamp_length_max(1.2);
        angular.0 = angular.0.clamp_length_max(2.0);
        commands
            .entity(entity)
            .insert((GravityScale(1.0), prop_layers()));
    }
}

/// 以時間步長平滑滑動書桌抽屜，並同步帶動停留在抽屜槽內的未持有道具。
pub fn animate_drawers(
    time: Res<Time>,
    held: Res<HeldProp>,
    mut drawers: Query<(&mut Drawer, &mut Transform, &mut Position), Without<Grabbable>>,
    mut props: Query<(Entity, &mut Transform, &mut Position), With<Grabbable>>,
) {
    let dt = time.delta_secs().min(0.1);
    for (mut drawer, mut transform, mut position) in &mut drawers {
        let target = if drawer.open { 1.0 } else { 0.0 };
        if (target - drawer.progress).abs() < 1e-4 {
            continue;
        }
        let previous = transform.translation;
        let factor = 1.0 - (-dt * 7.5).exp();
        drawer.progress += (target - drawer.progress) * factor;
        let next = drawer.closed_position + drawer.open_offset * drawer.progress;
        let delta = next - previous;
        transform.translation = next;
        position.0 = next;
        if delta.length_squared() < 1e-10 {
            continue;
        }
        let held_entity = held.0.map(|(entity, _)| entity);
        for (entity, mut prop_transform, mut prop_position) in &mut props {
            if Some(entity) == held_entity {
                continue;
            }
            let offset = prop_transform.translation - previous;
            if offset.x.abs() < 0.34 && offset.z.abs() < 0.44 && offset.y > -0.18 && offset.y < 0.16
            {
                prop_transform.translation += delta;
                prop_position.0 = prop_transform.translation;
            }
        }
    }
}

/// 維持手中道具在相機前方安全距離，以射線避開牆壁與桌面；離開遊玩畫面時自動放下。
pub fn carry_held_prop(
    mut commands: Commands,
    session: Res<Session>,
    spatial: SpatialQuery,
    player: Query<&Transform, With<Player>>,
    mut held: ResMut<HeldProp>,
    mut props: HeldPropQuery,
) {
    let Some((entity, _)) = held.0 else {
        return;
    };
    if session.screen != Screen::Playing {
        if let Ok((_, mut linear, mut angular)) = props.get_mut(entity) {
            linear.0 = Vec3::ZERO;
            angular.0 = Vec3::ZERO;
            commands
                .entity(entity)
                .insert((GravityScale(1.0), prop_layers()));
        }
        held.0 = None;
        return;
    }
    let Ok(camera) = player.single() else {
        return;
    };
    let Ok((prop_transform, mut linear, mut angular)) = props.get_mut(entity) else {
        held.0 = None;
        return;
    };
    let origin = camera.translation;
    let forward = *camera.forward();
    let filter = SpatialQueryFilter::from_mask([GameLayer::World, GameLayer::Furniture]);
    let reach = spatial
        .cast_ray(origin, Dir3::new_unchecked(forward), 1.15, true, &filter)
        .map_or(1.0, |hit| (hit.distance - 0.22).clamp(0.45, 1.0));
    let target = origin + forward * reach + Vec3::NEG_Y * 0.12;
    let to_target = target - prop_transform.translation;
    linear.0 = (to_target * 14.0).clamp_length_max(7.5);
    angular.0 *= 0.6;
}

/// 依托盤上方未持有道具的總質量更新下沉高度，達標時解開保險箱暗扣。
pub fn update_pressure_plate(
    time: Res<Time>,
    held: Res<HeldProp>,
    mut session: ResMut<Session>,
    props: PlatePropQuery,
    mut plates: Query<(&PressurePlate, &mut Transform, &mut Position), Without<LatchBolt>>,
    mut bolts: Query<(&LatchBolt, &mut Transform), Without<PressurePlate>>,
) {
    let dt = time.delta_secs().min(0.1);
    let factor = 1.0 - (-dt * 8.0).exp();
    let held_entity = held.0.map(|(entity, _)| entity);
    for (plate, mut transform, mut position) in &mut plates {
        let mut total_mass = 0.0_f32;
        for (entity, prop_transform, prop) in &props {
            if Some(entity) == held_entity {
                continue;
            }
            let delta = prop_transform.translation - plate.center;
            if delta.x.abs() <= plate.half.x + 0.08
                && delta.z.abs() <= plate.half.z + 0.08
                && delta.y >= -0.08
                && delta.y <= 0.45
            {
                total_mass += prop.kind.mass_kg();
            }
        }
        let triggered = plate_triggered(total_mass);
        if triggered
            && session.progress == Progress::Searching
            && session.progress.apply(Action::ReleaseLatch) == Reply::LatchReleased
        {
            session.notify(MessageKey::LatchReleased, SoundCue::Unlock);
        }
        let depressed = triggered || session.progress.latch_is_released();
        let target_y = if depressed {
            plate.base_y - plate.travel
        } else {
            plate.base_y
        };
        if (target_y - transform.translation.y).abs() > 1e-4 {
            transform.translation.y += (target_y - transform.translation.y) * factor;
            position.0.y = transform.translation.y;
        }
    }
    for (bolt, mut transform) in &mut bolts {
        let target_y = if session.progress.latch_is_released() {
            bolt.released_y
        } else {
            bolt.locked_y
        };
        if (target_y - transform.translation.y).abs() > 1e-4 {
            transform.translation.y += (target_y - transform.translation.y) * factor;
        }
    }
}

/// 重新開始時還原所有動態道具、可推家具與抽屜到初始位置並歸零速度。
pub fn apply_reset(
    mut commands: Commands,
    mut reset: ResMut<PhysicsReset>,
    mut held: ResMut<HeldProp>,
    mut props: ResetPropQuery,
    mut furniture: ResetFurnitureQuery,
    mut drawers: ResetDrawerQuery,
) {
    if !reset.0 {
        return;
    }
    reset.0 = false;
    held.0 = None;
    for (entity, prop, mut transform, mut position, mut rotation, mut linear, mut angular) in
        &mut props
    {
        *transform = prop.home;
        position.0 = prop.home.translation;
        rotation.0 = prop.home.rotation;
        linear.0 = Vec3::ZERO;
        angular.0 = Vec3::ZERO;
        commands
            .entity(entity)
            .insert((GravityScale(1.0), prop_layers()));
    }
    for (item, mut transform, mut position, mut rotation, mut linear, mut angular) in &mut furniture
    {
        *transform = item.home;
        position.0 = item.home.translation;
        rotation.0 = item.home.rotation;
        linear.0 = Vec3::ZERO;
        angular.0 = Vec3::ZERO;
    }
    for (mut drawer, mut transform, mut position) in &mut drawers {
        drawer.open = false;
        drawer.progress = 0.0;
        transform.translation = drawer.closed_position;
        position.0 = drawer.closed_position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        interaction::{self, Focus, FocusTarget},
        session::{UiCommand, UiCommands},
    };
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// 建立無頭 Avian3D 測試 App，以固定 60Hz 手動步長驅動物理與互動系統。
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            PhysicsPlugins::default(),
            bevy::asset::AssetPlugin::default(),
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .insert_resource(Session {
            screen: Screen::Playing,
            ..Default::default()
        })
        .init_resource::<UiCommands>()
        .init_resource::<Focus>()
        .init_resource::<HeldProp>()
        .init_resource::<PhysicsReset>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(
            Update,
            (
                interaction::act,
                apply_reset,
                animate_drawers,
                carry_held_prop,
                update_pressure_plate,
                interaction::focus_target,
            )
                .chain(),
        );
        app.finish();
        app
    }

    #[test]
    fn held_prop_lifts_to_camera_and_triggers_plate_when_dropped() {
        let mut app = test_app();
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 1.65, 0.0),
            Player {
                yaw: 0.0,
                pitch: 0.0,
            },
        ));
        let prop_home = Transform::from_xyz(0.0, 0.625, -1.5);
        let prop = app
            .world_mut()
            .spawn((
                prop_home,
                RigidBody::Dynamic,
                SleepingDisabled,
                Collider::cuboid(0.18, 0.10, 0.14),
                Mass(PropKind::Paperweight.mass_kg()),
                prop_layers(),
                Grabbable {
                    kind: PropKind::Paperweight,
                    home: prop_home,
                },
            ))
            .id();
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.5, -1.5),
            RigidBody::Static,
            Collider::cuboid(2.0, 0.1, 2.0),
            world_layers(),
        ));
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.58, -1.5),
            RigidBody::Kinematic,
            Collider::cuboid(0.52, 0.05, 0.52),
            world_layers(),
            PressurePlate {
                center: Vec3::new(0.0, 0.58, -1.5),
                half: Vec3::new(0.26, 0.05, 0.26),
                base_y: 0.58,
                travel: 0.035,
            },
        ));
        for _ in 0..10 {
            app.update();
        }
        app.world_mut().resource_mut::<HeldProp>().0 = Some((prop, PropKind::Paperweight));
        app.world_mut()
            .entity_mut(prop)
            .insert((GravityScale(0.0), CollisionLayers::NONE));
        for _ in 0..40 {
            app.update();
        }
        assert!(app.world().get::<Transform>(prop).unwrap().translation.y > 1.2);
        app.world_mut().resource_mut::<HeldProp>().0 = None;
        app.world_mut()
            .entity_mut(prop)
            .insert((GravityScale(1.0), prop_layers()));
        for _ in 0..80 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Session>().progress,
            Progress::LatchReleased
        );
    }

    #[test]
    fn focus_target_hits_props_and_act_picks_them_up() {
        let mut app = test_app();
        app.world_mut().spawn((
            Transform::from_xyz(-0.8, 0.94, -2.85).with_scale(Vec3::new(2.65, 0.13, 1.12)),
            RigidBody::Static,
            Collider::cuboid(1.0, 1.0, 1.0),
            world_layers(),
        ));
        let book_home = Transform::from_xyz(-1.72, 1.06, -3.08);
        let book = app
            .world_mut()
            .spawn((
                book_home,
                RigidBody::Dynamic,
                SleepingDisabled,
                Collider::cuboid(0.46, 0.09, 0.30),
                Mass(PropKind::Book.mass_kg()),
                LinearDamping(0.6),
                AngularDamping(1.2),
                prop_layers(),
                Grabbable {
                    kind: PropKind::Book,
                    home: book_home,
                },
            ))
            .id();
        let player = app
            .world_mut()
            .spawn((
                Transform::from_xyz(-1.72, 1.65, -2.0)
                    .looking_at(Vec3::new(-1.72, 1.06, -3.08), Vec3::Y),
                Player {
                    yaw: 0.0,
                    pitch: -0.5,
                },
            ))
            .id();
        for _ in 0..30 {
            app.update();
        }
        assert!(matches!(
            app.world().resource::<Focus>().0,
            Some(FocusTarget::Prop(e, PropKind::Book)) if e == book
        ));
        app.world_mut()
            .resource_mut::<UiCommands>()
            .0
            .push_back(UiCommand::Interact);
        app.update();
        assert!(app.world().resource::<HeldProp>().0.is_some());
        *app.world_mut().get_mut::<Transform>(player).unwrap() =
            Transform::from_xyz(-1.72, 1.65, -2.0);
        for _ in 0..40 {
            app.update();
        }
        assert!(app.world().get::<Transform>(book).unwrap().translation.y > 1.4);
    }
}
