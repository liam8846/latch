use crate::{
    Screen, Session,
    bridge::Command,
    player::{Player, START, Solid},
};
use bevy::prelude::*;
use latch::puzzle::{Action, Progress, Reply, parse_code};

#[derive(Clone, Copy, Component, PartialEq, Eq)]
pub enum Target {
    Note,
    Safe,
    Door,
    Painting,
}

impl Target {
    #[cfg(target_arch = "wasm32")]
    /// 提供準星對準物件時的可執行動作文字。
    pub fn label(self, progress: Progress) -> &'static str {
        match self {
            Self::Note => "閱讀便箋",
            Self::Safe if progress == Progress::SafeOpen => "拿取黃銅鑰匙",
            Self::Safe if progress.safe_is_open() => "查看保險箱",
            Self::Safe => "輸入保險箱密碼",
            Self::Door if progress.door_is_open() => "走出房間",
            Self::Door => "開啟房門",
            Self::Painting => "查看畫作",
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

/// 在 Rust 中處理 UI 命令與場景互動；瀏覽器只傳送動作，不能直接設定進度。
pub fn act(
    mut command: ResMut<Command>,
    mut session: ResMut<Session>,
    focus: Res<Focus>,
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Player)>,
) {
    let input = std::mem::take(&mut command.0);
    if input == "restart" {
        session.progress.apply(Action::Restart);
        session.screen = Screen::Playing;
        session.notify("午夜書房，再一次。", "start");
        if let Ok((mut transform, mut look)) = player.single_mut() {
            transform.translation = START;
            look.yaw = 0.0;
            look.pitch = -0.08;
        }
        return;
    }
    if input == "pause" || keys.just_pressed(KeyCode::Escape) {
        if session.screen != Screen::Intro && session.screen != Screen::Won {
            session.screen = Screen::Paused;
        }
        return;
    }
    if input == "begin"
        || input == "resume"
        || input == "close"
        || (keys.just_pressed(KeyCode::Enter)
            && matches!(session.screen, Screen::Intro | Screen::Paused))
    {
        session.screen = Screen::Playing;
        return;
    }
    if let Some(code) = input.strip_prefix("code:") {
        if session.screen == Screen::Safe {
            match parse_code(code).map(|code| session.progress.apply(Action::EnterCode(code))) {
                Some(Reply::SafeUnlocked) => {
                    session.screen = Screen::Playing;
                    session.notify("喀。保險箱的鎖鬆開了。", "unlock");
                }
                _ => session.notify("密碼不正確。再看看那張便箋。", "wrong"),
            }
        }
        return;
    }
    if session.screen != Screen::Playing
        || !(input == "interact" || keys.just_pressed(KeyCode::KeyE))
    {
        return;
    }
    match focus.0 {
        Some(Target::Note) => session.screen = Screen::Note,
        Some(Target::Safe) if session.progress == Progress::Searching => {
            session.screen = Screen::Safe
        }
        Some(Target::Safe) if session.progress == Progress::SafeOpen => {
            session.progress.apply(Action::TakeKey);
            session.notify("已取得黃銅鑰匙。房門在你的身後。", "key");
        }
        Some(Target::Safe) => session.notify("裡面已經空了。", "tap"),
        Some(Target::Door) => match session.progress.apply(Action::UnlockDoor) {
            Reply::DoorUnlocked => session.notify("鑰匙轉動。走進門外的光裡。", "unlock"),
            Reply::NeedKey => session.notify("門鎖住了。需要一把黃銅鑰匙。", "wrong"),
            _ => session.notify("門已打開，向前走。", "tap"),
        },
        Some(Target::Painting) => session.notify("《午夜之後》。畫框背面沒有藏東西。", "tap"),
        None => {}
    }
}
