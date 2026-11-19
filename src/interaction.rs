use crate::{
    Screen, Session,
    physics::{
        self, Drawer, GameLayer, Grabbable, HeldProp, PhysicsReset, PropControlQuery, PropKind,
    },
    player::{Player, START},
    room::{LightKind, SwitchableLight},
    session::{Exhibit, SoundCue, UiCommand, UiCommands},
};
use avian3d::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use latch::{
    i18n::MessageKey,
    puzzle::{Action, Progress, Reply},
};

#[derive(Clone, Copy, Component, PartialEq, Eq)]
pub enum Target {
    Note,
    Safe,
    Door,
    Photograph,
    Plate,
    DeskLamp,
    WallSwitch,
    Teacups,
    Crate,
    Bookshelf,
    WeightBox,
    MailStack,
    PostageChart,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Static(Target),
    Drawer(Entity, bool),
    Prop(Entity, PropKind),
    Drop(PropKind),
}

impl FocusTarget {
    #[cfg(target_arch = "wasm32")]
    /// 提供準星對準物件或手中持有道具時的可執行動作文字。
    pub fn label(self, progress: Progress) -> MessageKey {
        match self {
            Self::Drop(kind) => kind.drop_label(),
            Self::Prop(_, kind) => kind.pick_label(),
            Self::Drawer(_, open) => {
                if open {
                    MessageKey::CloseDrawer
                } else {
                    MessageKey::OpenDrawer
                }
            }
            Self::Static(Target::Note) => MessageKey::ReadNote,
            Self::Static(Target::Safe) if progress == Progress::SafeOpen => MessageKey::ReadLetter,
            Self::Static(Target::Safe) if progress.safe_is_open() => MessageKey::ViewSafe,
            Self::Static(Target::Safe) => MessageKey::EnterCode,
            Self::Static(Target::Door) if progress.door_is_open() => MessageKey::LeaveRoom,
            Self::Static(Target::Door) => MessageKey::OpenDoor,
            Self::Static(Target::Photograph) => MessageKey::ViewPhotograph,
            Self::Static(Target::Plate) => MessageKey::InspectPlate,
            Self::Static(Target::DeskLamp) => MessageKey::ToggleLamp,
            Self::Static(Target::WallSwitch) => MessageKey::ToggleSwitch,
            Self::Static(Target::Teacups) => MessageKey::InspectTeacups,
            Self::Static(Target::Crate) => MessageKey::InspectCrate,
            Self::Static(Target::Bookshelf) => MessageKey::InspectBookshelf,
            Self::Static(Target::WeightBox) => MessageKey::InspectWeightBox,
            Self::Static(Target::MailStack) => MessageKey::InspectMailStack,
            Self::Static(Target::PostageChart) => MessageKey::InspectPostageChart,
        }
    }
}

#[derive(Resource, Default)]
pub struct Focus(pub Option<FocusTarget>);

#[derive(SystemParam)]
pub struct FocusLookup<'w, 's> {
    targets: Query<'w, 's, &'static Target>,
    drawers: Query<'w, 's, &'static Drawer>,
    props: Query<'w, 's, &'static Grabbable>,
    parents: Query<'w, 's, &'static ChildOf>,
}

#[derive(SystemParam)]
pub struct WorldAct<'w, 's> {
    commands: Commands<'w, 's>,
    reset: ResMut<'w, PhysicsReset>,
    held: ResMut<'w, HeldProp>,
    drawers: Query<'w, 's, &'static mut Drawer>,
    lights: Query<'w, 's, (&'static mut SwitchableLight, &'static mut PointLight)>,
    props: PropControlQuery<'w, 's>,
}

/// 以 Avian 射線選取準星三點二公尺內的首個可互動物件；小道具提供窄球體輔助瞄準。
pub fn focus_target(
    spatial: SpatialQuery,
    held: Res<HeldProp>,
    player: Query<&Transform, With<Player>>,
    lookup: FocusLookup,
    mut focus: ResMut<Focus>,
    mut probe_cache: Local<Option<Collider>>,
) {
    focus.0 = None;
    if let Some((_, kind)) = held.0 {
        focus.0 = Some(FocusTarget::Drop(kind));
        return;
    }
    let Ok(camera) = player.single() else { return };
    let origin = camera.translation;
    let forward = Dir3::new_unchecked(*camera.forward());
    let filter = SpatialQueryFilter::from_mask([
        GameLayer::World,
        GameLayer::Prop,
        GameLayer::Furniture,
        GameLayer::Interactable,
    ]);
    let ray_hit = spatial.cast_ray(origin, forward, 3.2, true, &filter);
    if let Some(hit) = ray_hit {
        let mut current = Some(hit.entity);
        while let Some(entity) = current {
            if let Ok(prop) = lookup.props.get(entity) {
                focus.0 = Some(FocusTarget::Prop(entity, prop.kind));
                return;
            }
            if let Ok(drawer) = lookup.drawers.get(entity) {
                focus.0 = Some(FocusTarget::Drawer(entity, drawer.open));
                return;
            }
            if let Ok(target) = lookup.targets.get(entity) {
                focus.0 = Some(FocusTarget::Static(*target));
                return;
            }
            current = lookup.parents.get(entity).ok().map(ChildOf::parent);
        }
    }
    let max_prop_distance = ray_hit.map_or(3.2, |hit| (hit.distance + 0.08).min(3.2));
    let prop_filter = SpatialQueryFilter::from_mask([GameLayer::Prop]);
    let probe = probe_cache.get_or_insert_with(|| Collider::sphere(0.08));
    let config = ShapeCastConfig::from_max_distance(max_prop_distance);
    if let Some(hit) = spatial.cast_shape(
        probe,
        origin,
        Quat::IDENTITY,
        forward,
        &config,
        &prop_filter,
    ) && let Ok(prop) = lookup.props.get(hit.entity)
    {
        focus.0 = Some(FocusTarget::Prop(hit.entity, prop.kind));
    }
}

/// 依序處理型別化命令；只有 Session 擁有謎題與畫面狀態。
pub fn act(
    mut commands: ResMut<UiCommands>,
    mut session: ResMut<Session>,
    mut world: WorldAct,
    focus: Res<Focus>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Player)>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.0.push_back(UiCommand::Pause);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if keys.just_pressed(KeyCode::Enter) {
        match session.screen {
            Screen::Intro => commands.0.push_back(UiCommand::Begin),
            Screen::Paused => commands.0.push_back(UiCommand::Resume),
            _ => {}
        }
    }
    if keys.just_pressed(KeyCode::KeyE) {
        commands.0.push_back(UiCommand::Interact);
    }
    while let Some(command) = commands.0.pop_front() {
        match command {
            UiCommand::TakeKey => session.take_key(),
            UiCommand::Failure => session.screen = Screen::Paused,
            UiCommand::Restart => {
                session.progress.apply(Action::Restart);
                session.screen = Screen::Playing;
                session.notify(MessageKey::Restarted, SoundCue::Start);
                world.reset.0 = true;
                for (mut switchable, mut point) in &mut world.lights {
                    switchable.on = true;
                    point.intensity = switchable.intensity;
                }
                if let Ok((mut transform, mut look)) = player.single_mut() {
                    transform.translation = START;
                    look.yaw = 0.0;
                    look.pitch = -0.08;
                }
            }
            UiCommand::Pause if !matches!(session.screen, Screen::Intro | Screen::Won) => {
                session.screen = Screen::Paused;
            }
            UiCommand::Begin | UiCommand::Resume | UiCommand::Close => {
                session.screen = Screen::Playing;
            }
            UiCommand::SubmitCode(code) if session.screen == Screen::Safe => {
                match code.map(|code| session.progress.apply(Action::EnterCode(code))) {
                    Some(Reply::SafeUnlocked) => {
                        session.screen = Screen::Playing;
                        session.notify(MessageKey::SafeUnlocked, SoundCue::Unlock);
                    }
                    Some(Reply::LatchLocked) => {
                        session.screen = Screen::Playing;
                        session.notify(MessageKey::LatchLocked, SoundCue::Wrong);
                    }
                    _ => session.notify(MessageKey::IncorrectCode, SoundCue::Wrong),
                }
            }
            UiCommand::Interact if session.screen == Screen::Playing => {
                interact(&mut session, &mut world, focus.0);
            }
            _ => {}
        }
    }
}

/// 將目前準星目標套用為遊戲動作；重複互動由 reducer 的進度處理。
fn interact(session: &mut Session, world: &mut WorldAct, target: Option<FocusTarget>) {
    match target {
        Some(FocusTarget::Drop(_)) => {
            physics::drop_held(&mut world.commands, &mut world.held, &mut world.props);
        }
        Some(FocusTarget::Prop(entity, kind)) => {
            physics::pick_up(
                &mut world.commands,
                &mut world.held,
                entity,
                kind,
                &mut world.props,
            );
        }
        Some(FocusTarget::Drawer(entity, _)) => {
            if let Ok(mut drawer) = world.drawers.get_mut(entity) {
                drawer.open = !drawer.open;
                let message = if drawer.open {
                    MessageKey::DrawerOpened
                } else {
                    MessageKey::DrawerClosed
                };
                session.notify(message, SoundCue::Tap);
            }
        }
        Some(FocusTarget::Static(Target::Note)) => {
            session.screen = Screen::Inspect(Exhibit::Note);
        }
        Some(FocusTarget::Static(Target::Safe)) if session.progress == Progress::Searching => {
            session.notify(MessageKey::LatchLocked, SoundCue::Wrong);
        }
        Some(FocusTarget::Static(Target::Safe)) if session.progress == Progress::LatchReleased => {
            session.screen = Screen::Safe;
        }
        Some(FocusTarget::Static(Target::Safe)) => {
            session.screen = Screen::Inspect(Exhibit::Letter);
        }
        Some(FocusTarget::Static(Target::Door)) => match session.progress.apply(Action::UnlockDoor)
        {
            Reply::DoorUnlocked => session.notify(MessageKey::DoorUnlocked, SoundCue::Unlock),
            Reply::NeedKey => session.notify(MessageKey::NeedKey, SoundCue::Wrong),
            _ => session.notify(MessageKey::DoorAlreadyOpen, SoundCue::Tap),
        },
        Some(FocusTarget::Static(Target::Photograph)) => {
            session.screen = Screen::Inspect(Exhibit::Photograph);
        }
        Some(FocusTarget::Static(Target::Plate)) => {
            session.notify(MessageKey::PlateHint, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::DeskLamp)) => {
            toggle_light(
                session,
                &mut world.lights,
                LightKind::DeskLamp,
                MessageKey::LampOn,
                MessageKey::LampOff,
            );
        }
        Some(FocusTarget::Static(Target::WallSwitch)) => {
            toggle_light(
                session,
                &mut world.lights,
                LightKind::Chandelier,
                MessageKey::ChandelierOn,
                MessageKey::ChandelierOff,
            );
        }
        Some(FocusTarget::Static(Target::Teacups)) => {
            session.notify(MessageKey::TeacupsNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::Crate)) => {
            session.notify(MessageKey::CrateNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::Bookshelf)) => {
            session.notify(MessageKey::BookshelfNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::WeightBox)) => {
            session.notify(MessageKey::WeightBoxNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::MailStack)) => {
            session.notify(MessageKey::MailStackNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::PostageChart)) => {
            session.notify(MessageKey::PostageChartNote, SoundCue::Tap);
        }
        None => {}
    }
}

/// 切換指定書房光源的開關狀態，同步更新點光源強度與提示訊息。
fn toggle_light(
    session: &mut Session,
    lights: &mut Query<(&mut SwitchableLight, &mut PointLight)>,
    kind: LightKind,
    on_message: MessageKey,
    off_message: MessageKey,
) {
    for (mut switchable, mut point) in lights.iter_mut() {
        if switchable.kind == kind {
            switchable.on = !switchable.on;
            point.intensity = if switchable.on {
                switchable.intensity
            } else {
                0.0
            };
            session.notify(
                if switchable.on {
                    on_message
                } else {
                    off_message
                },
                SoundCue::Tap,
            );
            break;
        }
    }
}
