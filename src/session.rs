use bevy::prelude::*;
use latch::{i18n::MessageKey, puzzle::Progress};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Intro,
    Playing,
    Paused,
    Note,
    Safe,
    Won,
}

impl Screen {
    /// 提供既有 HTML/CSS 的呈現狀態名稱。
    pub fn id(self) -> &'static str {
        match self {
            Self::Intro => "intro",
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Note => "note",
            Self::Safe => "safe",
            Self::Won => "won",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundCue {
    Begin,
    Start,
    Tap,
    Wrong,
    Unlock,
    Key,
    Win,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notice {
    pub message: MessageKey,
    pub sound: SoundCue,
}

#[derive(Resource, Default)]
pub struct Session {
    pub progress: Progress,
    pub screen: Screen,
    pub notice: Option<Notice>,
    pub revision: u32,
}

impl Session {
    /// 更新型別化提示與事件序號，重複提示仍會觸發新的音效。
    pub fn notify(&mut self, message: MessageKey, sound: SoundCue) {
        self.notice = Some(Notice { message, sound });
        self.revision = self.revision.wrapping_add(1);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiCommand {
    Begin,
    Resume,
    Close,
    Pause,
    Restart,
    Interact,
    SubmitCode(Option<[u8; 4]>),
    Failure,
}

#[derive(Resource, Default)]
pub struct UiCommands(pub std::collections::VecDeque<UiCommand>);
