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
    puzzle::{Action, ClockFace, ClockHand, Lever, Progress, Reply},
    signal::{Mirror, Signal, SignalBoard},
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
    Timetable,
    MoonMark,
    Lever(Lever),
    ClockDial(ClockHand),
    FrontDoor,
    Mailbox,
    CoatRack,
    FoyerLight,
    SignalMirror(Mirror),
    SignalManual,
    SignalGate,
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
            Self::Static(Target::Timetable) => MessageKey::ReadTimetable,
            Self::Static(Target::MoonMark) => MessageKey::ReadMoonMark,
            Self::Static(Target::Lever(Lever::One)) => MessageKey::PullLeverOne,
            Self::Static(Target::Lever(Lever::Two)) => MessageKey::PullLeverTwo,
            Self::Static(Target::Lever(Lever::Three)) => MessageKey::PullLeverThree,
            Self::Static(Target::Lever(Lever::Four)) => MessageKey::PullLeverFour,
            Self::Static(Target::ClockDial(ClockHand::Hour)) => MessageKey::TurnClockHour,
            Self::Static(Target::ClockDial(ClockHand::Minute)) => MessageKey::TurnClockMinute,
            Self::Static(Target::FrontDoor) if progress.front_door_is_open() => {
                MessageKey::LeaveHouse
            }
            Self::Static(Target::FrontDoor) => MessageKey::OpenFrontDoor,
            Self::Static(Target::Mailbox) => MessageKey::InspectMailbox,
            Self::Static(Target::CoatRack) => MessageKey::InspectCoatRack,
            Self::Static(Target::FoyerLight) => MessageKey::ToggleFoyerLight,
            Self::Static(Target::SignalMirror(_)) => MessageKey::TurnSignalMirror,
            Self::Static(Target::SignalManual) => MessageKey::ReadSignalManual,
            Self::Static(Target::SignalGate) => MessageKey::InspectSignalGate,
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
    session: Res<Session>,
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
                if !matches!(target, Target::SignalMirror(_))
                    || session.progress == Progress::SignalRoom
                {
                    focus.0 = Some(FocusTarget::Static(*target));
                }
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
                session.clock = ClockFace::default();
                session.signal_board = SignalBoard::default();
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
        Some(FocusTarget::Static(Target::Timetable)) => {
            session.screen = Screen::Inspect(Exhibit::Timetable);
        }
        Some(FocusTarget::Static(Target::MoonMark)) => {
            let foyer_dark = world
                .lights
                .iter()
                .any(|(light, _)| light.kind == LightKind::FoyerLamp && !light.on);
            if foyer_dark {
                session.screen = Screen::Inspect(Exhibit::MoonMark);
            } else {
                session.notify(MessageKey::MoonMarkTooBright, SoundCue::Tap);
            }
        }
        Some(FocusTarget::Static(Target::ClockDial(hand))) => {
            if session.progress.foyer_clock_synced() {
                session.notify(MessageKey::ClockAlreadySynced, SoundCue::Tap);
            } else {
                session.clock.advance(hand);
                if session.clock.is_target()
                    && session.progress.apply(Action::SyncClock) == Reply::ClockSynced
                {
                    session.notify(MessageKey::ClockSynced, SoundCue::Chime);
                } else {
                    let message = match hand {
                        ClockHand::Hour => MessageKey::ClockHourTick,
                        ClockHand::Minute => MessageKey::ClockMinuteTick,
                    };
                    session.notify(message, SoundCue::Tap);
                }
            }
        }
        Some(FocusTarget::Static(Target::Lever(lever))) => {
            if session.progress == Progress::DoorOpen {
                session.progress.apply(Action::EnterFoyer);
            }
            match session.progress.apply(Action::PullLever(lever)) {
                Reply::ClockNotSynced => {
                    session.notify(MessageKey::ClockNotSynced, SoundCue::Wrong);
                }
                Reply::LeverEngaged => {
                    session.notify(MessageKey::LeverEngaged, SoundCue::Tap);
                }
                Reply::LeversReset => {
                    session.notify(MessageKey::LeversReset, SoundCue::Wrong);
                }
                Reply::FrontDoorUnlocked => {
                    session.notify(MessageKey::FrontDoorUnlocked, SoundCue::Unlock);
                }
                _ => {}
            }
        }
        Some(FocusTarget::Static(Target::FrontDoor)) => {
            if session.progress.front_door_is_open() {
                session.notify(MessageKey::FrontDoorAlreadyOpen, SoundCue::Tap);
            } else {
                session.notify(MessageKey::FrontDoorLocked, SoundCue::Wrong);
            }
        }
        Some(FocusTarget::Static(Target::Mailbox)) => {
            session.notify(MessageKey::MailboxNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::CoatRack)) => {
            session.notify(MessageKey::CoatRackNote, SoundCue::Tap);
        }
        Some(FocusTarget::Static(Target::FoyerLight)) => {
            toggle_light(
                session,
                &mut world.lights,
                LightKind::FoyerLamp,
                MessageKey::FoyerLightOn,
                MessageKey::FoyerLightOff,
            );
        }
        Some(FocusTarget::Static(Target::SignalMirror(mirror))) => {
            turn_signal_mirror(session, mirror)
        }
        Some(FocusTarget::Static(Target::SignalManual)) => {
            session.screen = Screen::Inspect(Exhibit::SignalManual);
        }
        Some(FocusTarget::Static(Target::SignalGate)) => {
            let message = if session.progress.signal_gate_is_open() {
                MessageKey::ObjectiveSignalGate
            } else {
                MessageKey::SignalGateLocked
            };
            session.notify(message, SoundCue::Tap);
        }
        None => {}
    }
}

/// 調整鏡片後立即反映兩座接收器的狀態；閘門接電後鎖定鏡片檔位。
fn turn_signal_mirror(session: &mut Session, mirror: Mirror) {
    if session.progress != Progress::SignalRoom {
        return;
    }
    let [amber, blue] = Signal::ALL.map(|signal| session.signal_board.trace(signal).received);
    session.signal_board.turn(mirror);
    let board = session.signal_board;
    if session.progress.apply(Action::AlignSignals(board)) == Reply::SignalGateUnlocked {
        session.notify(MessageKey::SignalGateUnlocked, SoundCue::Unlock);
    } else if !amber && board.trace(Signal::Amber).received {
        session.notify(MessageKey::AmberReceived, SoundCue::Chime);
    } else if !blue && board.trace(Signal::Blue).received {
        session.notify(MessageKey::BlueReceived, SoundCue::Chime);
    } else {
        session.notify(MessageKey::SignalMirrorTurned, SoundCue::Tap);
    }
}

/// 切換同類型所有書房或玄關光源的開關狀態，同步更新點光源強度與提示訊息。
fn toggle_light(
    session: &mut Session,
    lights: &mut Query<(&mut SwitchableLight, &mut PointLight)>,
    kind: LightKind,
    on_message: MessageKey,
    off_message: MessageKey,
) {
    let mut next_state = None;
    for (mut switchable, mut point) in lights.iter_mut() {
        if switchable.kind == kind {
            let on = *next_state.get_or_insert(!switchable.on);
            switchable.on = on;
            point.intensity = if on { switchable.intensity } else { 0.0 };
        }
    }
    if let Some(on) = next_state {
        session.notify(if on { on_message } else { off_message }, SoundCue::Tap);
    }
}
