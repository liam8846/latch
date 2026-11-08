mod audio;
mod dom;
mod events;

use crate::{
    interaction::Focus,
    player::Player,
    session::{Exhibit, Screen, Session, UiCommand, UiCommands},
};
use audio::Audio;
use bevy::prelude::*;
use dom::Dom;
use events::Listener;
use latch::{
    i18n::{Locale, Localizer, MessageKey},
    puzzle::Progress,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::{self, Receiver},
};
use wasm_bindgen::JsValue;

enum BrowserCommand {
    Game(UiCommand),
    Locale(Locale),
    Failure,
}

pub struct BrowserUi {
    dom: Dom,
    localizer: Localizer,
    receiver: Receiver<BrowserCommand>,
    screen: Rc<Cell<Screen>>,
    audio: Rc<RefCell<Audio>>,
    _listeners: Vec<Listener>,
    last_screen: Option<Screen>,
    last_revision: u32,
    toast: Option<(MessageKey, f64)>,
    safe_message: MessageKey,
    failed: Rc<Cell<bool>>,
    failure_message: Rc<RefCell<String>>,
}

impl BrowserUi {
    /// 在 DOM 邊界完成驗證並建立唯一的瀏覽器呈現擁有者。
    pub fn new() -> Result<Self, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("Missing browser document"))?;
        let dom = Dom::new(document)?;
        let localizer =
            Localizer::new(Locale::ZhHant).map_err(|error| JsValue::from_str(&error))?;
        dom.translate(&localizer);
        let screen = Rc::new(Cell::new(Screen::Intro));
        let audio = Rc::new(RefCell::new(Audio::default()));
        let failed = Rc::new(Cell::new(false));
        let failure_message = Rc::new(RefCell::new(localizer.text("error-runtime", None)));
        let (sender, receiver) = mpsc::channel();
        let listeners = events::bind(
            &dom,
            Rc::clone(&screen),
            Rc::clone(&audio),
            Rc::clone(&failure_message),
            Rc::clone(&failed),
            sender,
        )?;
        Ok(Self {
            dom,
            localizer,
            receiver,
            screen,
            audio,
            _listeners: listeners,
            last_screen: None,
            last_revision: 0,
            toast: None,
            safe_message: MessageKey::SafeHint,
            failed,
            failure_message,
        })
    }

    /// 只將遊戲命令交給 Bevy；語言切換留在呈現層且不修改 Session。
    pub fn read_commands(&mut self, commands: &mut UiCommands) {
        while let Ok(command) = self.receiver.try_recv() {
            match command {
                BrowserCommand::Game(command) if !self.failed.get() => {
                    commands.0.push_back(command)
                }
                BrowserCommand::Game(_) => {}
                BrowserCommand::Locale(locale) if locale != self.localizer.locale() => {
                    self.localizer.set_locale(locale);
                    self.dom.translate(&self.localizer);
                    *self.failure_message.borrow_mut() = self.localizer.text("error-runtime", None);
                }
                BrowserCommand::Locale(_) => {}
                BrowserCommand::Failure => {
                    self.failed.set(true);
                    commands.0.clear();
                    commands.0.push_back(UiCommand::Failure);
                }
            }
        }
    }

    /// 直接呈現唯讀 ECS 資料；now 使用 Bevy 真實經過秒數，語言切換不重設提示期限。
    pub fn render(
        &mut self,
        session: &Session,
        focus: &Focus,
        player: Option<(&Transform, &Player)>,
        now: f64,
    ) {
        if self.failed.get() {
            self.dom.hidden("loading", true);
            self.dom.hidden("error", false);
            self.dom
                .text("error-message", &self.localizer.text("error-runtime", None));
            self.render_sound();
            return;
        }
        self.screen.set(session.screen);
        self.audio.borrow_mut().update(session);
        let screen = session.screen.id();
        attribute(&self.dom.root, "data-screen", screen);
        self.dom.hidden("loading", true);
        for name in ["intro", "paused", "inspect", "safe", "won"] {
            self.dom.hidden(name, name != screen);
        }
        self.dom.hidden("shade", session.screen == Screen::Playing);
        self.dom.hidden("hud", session.screen != Screen::Playing);
        self.dom.hidden("pause", session.screen != Screen::Playing);
        if self.last_screen != Some(session.screen) {
            self.enter_screen(session.screen);
            self.last_screen = Some(session.screen);
        }
        if session.revision != self.last_revision {
            self.last_revision = session.revision;
            if let Some(notice) = session.notice {
                self.toast = Some((notice.message, now + 5.5));
                self.audio.borrow_mut().cue(notice.sound);
                if session.screen == Screen::Safe {
                    self.safe_message = notice.message;
                    let _ = self.dom.code.focus();
                }
            }
        }
        if self.toast.is_some_and(|(_, expires)| now >= expires) {
            self.toast = None;
        }
        let toast = self
            .toast
            .map(|(key, _)| self.localizer.message(key))
            .unwrap_or_default();
        self.dom.text("toast", &toast);
        self.dom
            .text("safe-message", &self.localizer.message(self.safe_message));
        self.dom.text(
            "objective",
            &self.localizer.message(objective(session.progress)),
        );
        let inventory = if matches!(
            session.progress,
            Progress::KeyHeld | Progress::DoorOpen | Progress::Escaped
        ) {
            MessageKey::InventoryKey
        } else {
            MessageKey::InventoryEmpty
        };
        self.dom
            .text("inventory", &self.localizer.message(inventory));
        let target = focus
            .0
            .map(|target| self.localizer.message(target.label(session.progress)))
            .unwrap_or_default();
        self.dom.text("target", &target);
        self.dom.hidden("interact", target.is_empty());
        self.render_inspect(session);
        self.render_sound();
        attribute(
            &self.dom.root,
            "data-stage",
            &format!("{:?}", session.progress),
        );
        attribute(
            &self.dom.root,
            "data-revision",
            &session.revision.to_string(),
        );
        if let Some((transform, look)) = player {
            attribute(
                &self.dom.root,
                "data-position",
                &format!(
                    "{:.2},{:.2},{:.2}",
                    transform.translation.x, transform.translation.y, transform.translation.z
                ),
            );
            attribute(
                &self.dom.root,
                "data-look",
                &format!("{:.3},{:.3}", look.yaw, look.pitch),
            );
        }
    }

    /// 共用閱讀面板從型別化展品導出，語言切換不改變焦點。
    fn render_inspect(&self, session: &Session) {
        let Screen::Inspect(exhibit) = session.screen else {
            return;
        };
        let prefix = match exhibit {
            Exhibit::Note => "note",
            Exhibit::Photograph => "photograph",
            Exhibit::Letter => "letter",
        };
        for part in ["title", "eyebrow", "copy", "caption"] {
            self.dom.text(
                &format!("inspect-{part}"),
                &self.localizer.text(&format!("{prefix}-{part}"), None),
            );
        }
        attribute(self.dom.get("inspect"), "data-exhibit", prefix);
        self.dom
            .hidden("inspect-image", exhibit != Exhibit::Photograph);
        let take = exhibit == Exhibit::Letter && session.progress == Progress::SafeOpen;
        self.dom.hidden("take-key", !take);
        self.dom.hidden("close-inspect-footer", take);
    }

    /// 只在真正畫面轉換時移動焦點與清空新開啟的密碼欄。
    fn enter_screen(&mut self, screen: Screen) {
        if screen != Screen::Playing && self.dom.document.pointer_lock_element().is_some() {
            self.dom.document.exit_pointer_lock();
        }
        let focus = match screen {
            Screen::Playing => Some(&self.dom.canvas),
            Screen::Intro => Some(self.dom.get("begin")),
            Screen::Paused => Some(self.dom.get("resume")),
            Screen::Inspect(_) => Some(self.dom.get("close-inspect")),
            Screen::Won => Some(self.dom.get("restart-won")),
            Screen::Safe => {
                self.dom.code.set_value("");
                self.safe_message = MessageKey::SafeHint;
                let _ = self.dom.code.focus();
                None
            }
        };
        if let Some(element) = focus {
            let _ = element.focus();
        }
    }

    /// 音訊啟用狀態由手勢 callback 擁有，按鈕文案在目前語言格式化。
    fn render_sound(&self) {
        let enabled = self.audio.borrow().enabled();
        let key = if enabled { "sound-on" } else { "sound-off" };
        self.dom.text("sound", &self.localizer.text(key, None));
        attribute(
            self.dom.get("sound"),
            "aria-pressed",
            if enabled { "true" } else { "false" },
        );
        let key = if enabled {
            "sound-disable"
        } else {
            "sound-enable"
        };
        attribute(
            self.dom.get("sound"),
            "aria-label",
            &self.localizer.text(key, None),
        );
    }
}

impl Drop for BrowserUi {
    /// 釋放 UI 時先移除 listeners，再解除相關瀏覽器輸入狀態。
    fn drop(&mut self) {
        self._listeners.clear();
        if self.dom.document.pointer_lock_element().is_some() {
            self.dom.document.exit_pointer_lock();
        }
    }
}

/// 依謎題進度選取型別化目標，翻譯不屬於 reducer。
fn objective(progress: Progress) -> MessageKey {
    match progress {
        Progress::Searching => MessageKey::ObjectiveSearch,
        Progress::SafeOpen => MessageKey::ObjectiveSafe,
        Progress::KeyHeld => MessageKey::ObjectiveKey,
        Progress::DoorOpen => MessageKey::ObjectiveDoor,
        Progress::Escaped => MessageKey::ObjectiveEscaped,
    }
}

/// 只更新已改變的唯讀診斷與無障礙屬性，DOM 不傳送字串命令。
fn attribute(element: &web_sys::Element, name: &str, value: &str) {
    if element.get_attribute(name).as_deref() != Some(value) {
        let _ = element.set_attribute(name, value);
    }
}
