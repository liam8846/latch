use crate::{
    Screen, Session,
    player::{Player, START, Solid},
    session::{Exhibit, SoundCue, UiCommand, UiCommands},
};
use bevy::prelude::*;
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
}

impl Target {
    #[cfg(target_arch = "wasm32")]
    /// 提供準星對準物件時的可執行動作文字。
    pub fn label(self, progress: Progress) -> MessageKey {
        match self {
            Self::Note => MessageKey::ReadNote,
            Self::Safe if progress == Progress::SafeOpen => MessageKey::ReadLetter,
            Self::Safe if progress.safe_is_open() => MessageKey::ViewSafe,
            Self::Safe => MessageKey::EnterCode,
            Self::Door if progress.door_is_open() => MessageKey::LeaveRoom,
            Self::Door => MessageKey::OpenDoor,
            Self::Photograph => MessageKey::ViewPhotograph,
        }
    }
}

#[derive(Resource, Default)]
pub struct Focus(pub Option<Target>);

/// 只選取三公尺內、準星附近且未被實體遮擋的互動節點。
pub fn focus_target(
    player: Query<&Transform, With<Player>>,
    targets: Query<(&Transform, &Target), Without<Player>>,
    solids: Query<&Solid>,
    mut focus: ResMut<Focus>,
) {
    focus.0 = None;
    let Ok(camera) = player.single() else { return };
    let origin = camera.translation;
    let forward = *camera.forward();
    let mut nearest = 3.2_f32;
    for (transform, target) in &targets {
        let offset = transform.translation - origin;
        let along = offset.dot(forward);
        let miss = (offset - forward * along).length();
        if along < 0.0 || along > nearest || miss > 0.42 {
            continue;
        }
        let distance = offset.length();
        let direction = offset / distance;
        let occluded = solids.iter().any(|solid| {
            solid
                .ray_distance(origin, direction)
                .is_some_and(|hit| hit + 0.48 < distance)
        });
        if !occluded {
            nearest = along;
            focus.0 = Some(*target);
        }
    }
}

/// 依序處理型別化命令；只有 Session 擁有謎題與畫面狀態。
pub fn act(
    mut commands: ResMut<UiCommands>,
    mut session: ResMut<Session>,
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
                    _ => session.notify(MessageKey::IncorrectCode, SoundCue::Wrong),
                }
            }
            UiCommand::Interact if session.screen == Screen::Playing => {
                interact(&mut session, focus.0);
            }
            _ => {}
        }
    }
}

/// 將目前準星目標套用為遊戲動作；重複互動由 reducer 的進度處理。
fn interact(session: &mut Session, target: Option<Target>) {
    match target {
        Some(Target::Note) => session.screen = Screen::Inspect(Exhibit::Note),
        Some(Target::Safe) if session.progress == Progress::Searching => {
            session.screen = Screen::Safe
        }
        Some(Target::Safe) => session.screen = Screen::Inspect(Exhibit::Letter),
        Some(Target::Door) => match session.progress.apply(Action::UnlockDoor) {
            Reply::DoorUnlocked => session.notify(MessageKey::DoorUnlocked, SoundCue::Unlock),
            Reply::NeedKey => session.notify(MessageKey::NeedKey, SoundCue::Wrong),
            _ => session.notify(MessageKey::DoorAlreadyOpen, SoundCue::Tap),
        },
        Some(Target::Photograph) => session.screen = Screen::Inspect(Exhibit::Photograph),
        None => {}
    }
}
