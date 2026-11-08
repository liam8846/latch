use bevy::prelude::*;
use latch::{i18n::MessageKey, puzzle::Progress};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exhibit {
    Note,
    Photograph,
    Letter,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Intro,
    Playing,
    Paused,
    Inspect(Exhibit),
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
            Self::Inspect(_) => "inspect",
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
    /// 只有閱讀已開啟保險箱的信時可領取鑰匙；重複命令不改變狀態。
    pub fn take_key(&mut self) {
        if self.screen == Screen::Inspect(Exhibit::Letter) && self.progress == Progress::SafeOpen {
            self.progress.apply(latch::puzzle::Action::TakeKey);
            self.screen = Screen::Playing;
            self.notify(MessageKey::KeyTaken, SoundCue::Key);
        }
    }

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
    TakeKey,
    Failure,
}

#[derive(Resource, Default)]
pub struct UiCommands(pub std::collections::VecDeque<UiCommand>);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_requires_open_safe_and_letter_and_cannot_repeat() {
        for screen in [
            Screen::Playing,
            Screen::Safe,
            Screen::Inspect(Exhibit::Note),
            Screen::Inspect(Exhibit::Photograph),
        ] {
            let mut session = Session {
                screen,
                progress: Progress::SafeOpen,
                ..Default::default()
            };
            session.take_key();
            assert_eq!(session.progress, Progress::SafeOpen);
            assert_eq!(session.revision, 0);
        }
        let mut session = Session {
            screen: Screen::Inspect(Exhibit::Letter),
            ..Default::default()
        };
        session.take_key();
        assert_eq!(session.progress, Progress::Searching);
        session.progress = Progress::SafeOpen;
        session.take_key();
        assert_eq!(session.progress, Progress::KeyHeld);
        assert_eq!(session.screen, Screen::Playing);
        assert_eq!(session.revision, 1);
        session.screen = Screen::Inspect(Exhibit::Letter);
        session.take_key();
        assert_eq!(session.revision, 1);
    }
}
