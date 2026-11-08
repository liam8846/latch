use super::{BrowserCommand, audio::Audio, dom::Dom};
use crate::session::{Screen, SoundCue, UiCommand};
use latch::{i18n::Locale, puzzle::parse_code};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc::Sender,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{Document, Event, EventTarget, HtmlElement, KeyboardEvent};

pub struct Listener {
    target: EventTarget,
    kind: &'static str,
    callback: Closure<dyn FnMut(Event)>,
}

impl Listener {
    /// 註冊 callback 並保留其擁有權；任一初始化錯誤由呼叫者處理。
    fn new(
        target: &EventTarget,
        kind: &'static str,
        callback: impl FnMut(Event) + 'static,
    ) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
        target.add_event_listener_with_callback(kind, callback.as_ref().unchecked_ref())?;
        Ok(Self {
            target: target.clone(),
            kind,
            callback,
        })
    }
}

impl Drop for Listener {
    /// 移除相同 DOM listener 後釋放 Closure，不使用永久 forget。
    fn drop(&mut self) {
        let _ = self
            .target
            .remove_event_listener_with_callback(self.kind, self.callback.as_ref().unchecked_ref());
    }
}

/// 綁定瀏覽器邊界；callbacks 只擁有呈現快照、音訊和型別化佇列，不共用 Session。
pub fn bind(
    dom: &Dom,
    screen: Rc<Cell<Screen>>,
    audio: Rc<RefCell<Audio>>,
    failure_message: Rc<RefCell<String>>,
    failed: Rc<Cell<bool>>,
    sender: Sender<BrowserCommand>,
) -> Result<Vec<Listener>, JsValue> {
    let mut listeners = Vec::new();
    for (id, command, enter) in [
        ("begin", UiCommand::Begin, true),
        ("resume", UiCommand::Resume, true),
        ("restart-paused", UiCommand::Restart, true),
        ("restart-won", UiCommand::Restart, true),
        ("close-inspect", UiCommand::Close, true),
        ("close-inspect-footer", UiCommand::Close, true),
        ("take-key", UiCommand::TakeKey, true),
        ("close-safe", UiCommand::Close, true),
        ("pause", UiCommand::Pause, false),
        ("interact", UiCommand::Interact, false),
    ] {
        let canvas = dom.canvas.clone();
        let audio = Rc::clone(&audio);
        let sender = sender.clone();
        let failed = Rc::clone(&failed);
        listeners.push(Listener::new(dom.get(id), "click", move |_| {
            if failed.get() {
                return;
            }
            if command == UiCommand::Pause {
                audio.borrow_mut().silence();
            }
            if command == UiCommand::Restart {
                audio.borrow_mut().restart();
            }
            if enter {
                let mut audio = audio.borrow_mut();
                audio.unlock();
                if command == UiCommand::Begin {
                    audio.begin();
                    audio.cue(SoundCue::Begin);
                }
                drop(audio);
                let _ = canvas.focus();
                request_lock(&canvas);
            } else if command == UiCommand::Interact {
                let _ = canvas.focus();
            }
            let _ = sender.send(BrowserCommand::Game(command));
        })?);
    }
    {
        let audio = Rc::clone(&audio);
        let screen = Rc::clone(&screen);
        let canvas = dom.canvas.clone();
        listeners.push(Listener::new(dom.get("sound"), "click", move |_| {
            let mut audio = audio.borrow_mut();
            let enabled = !audio.enabled();
            audio.unlock();
            audio.set_enabled(enabled);
            if screen.get() == Screen::Playing {
                let _ = canvas.focus();
            }
        })?);
    }
    {
        let language = dom.language.clone();
        let sender = sender.clone();
        listeners.push(Listener::new(&dom.language, "change", move |_| {
            if let Some(locale) = Locale::parse(&language.value()) {
                let _ = sender.send(BrowserCommand::Locale(locale));
            }
        })?);
    }
    {
        let code = dom.code.clone();
        let canvas = dom.canvas.clone();
        let sender = sender.clone();
        let failed = Rc::clone(&failed);
        listeners.push(Listener::new(
            dom.get("code-form"),
            "submit",
            move |event| {
                event.prevent_default();
                if failed.get() {
                    return;
                }
                let value = parse_code(&code.value());
                let _ = canvas.focus();
                let _ = sender.send(BrowserCommand::Game(UiCommand::SubmitCode(value)));
            },
        )?);
        let code = dom.code.clone();
        listeners.push(Listener::new(&dom.code, "input", move |_| {
            let filtered: String = code
                .value()
                .chars()
                .filter(char::is_ascii_digit)
                .take(4)
                .collect();
            if filtered != code.value() {
                code.set_value(&filtered);
            }
        })?);
    }
    {
        let canvas = dom.canvas.clone();
        let document = dom.document.clone();
        let screen = Rc::clone(&screen);
        let audio = Rc::clone(&audio);
        let failed = Rc::clone(&failed);
        listeners.push(Listener::new(&dom.canvas, "click", move |_| {
            if !failed.get()
                && screen.get() == Screen::Playing
                && document.pointer_lock_element().is_none()
            {
                audio.borrow_mut().unlock();
                let _ = canvas.focus();
                request_lock(&canvas);
            }
        })?);
    }
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing window"))?;
    {
        let document = dom.document.clone();
        let screen = Rc::clone(&screen);
        let sender = sender.clone();
        let audio = Rc::clone(&audio);
        let failed = Rc::clone(&failed);
        listeners.push(Listener::new(&dom.document, "keydown", move |event| {
            if failed.get() {
                return;
            }
            let Some(key) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            if key.code() == "Escape" && !matches!(screen.get(), Screen::Intro | Screen::Won) {
                audio.borrow_mut().silence();
                let _ = sender.send(BrowserCommand::Game(UiCommand::Pause));
                event.prevent_default();
            }
            if screen.get() == Screen::Playing
                && document
                    .active_element()
                    .is_some_and(|element| element.id() == "game")
                && matches!(
                    key.code().as_str(),
                    "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Space"
                )
            {
                event.prevent_default();
            }
            if key.code() == "Tab" {
                trap_focus(&document, screen.get(), key);
            }
        })?);
    }
    for (target, kind) in [
        (EventTarget::from(window), "blur"),
        (EventTarget::from(dom.document.clone()), "visibilitychange"),
        (EventTarget::from(dom.document.clone()), "pointerlockchange"),
    ] {
        let screen = Rc::clone(&screen);
        let document = dom.document.clone();
        let sender = sender.clone();
        let audio = Rc::clone(&audio);
        listeners.push(Listener::new(&target, kind, move |_| {
            let should_pause = match kind {
                "blur" => true,
                "visibilitychange" => document.hidden(),
                _ => document.pointer_lock_element().is_none(),
            };
            if should_pause && (screen.get() == Screen::Playing || kind != "pointerlockchange") {
                audio.borrow_mut().silence();
                let _ = sender.send(BrowserCommand::Game(UiCommand::Pause));
            }
            if kind == "blur" && document.pointer_lock_element().is_some() {
                document.exit_pointer_lock();
            }
        })?);
    }
    {
        let sender = sender.clone();
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("Missing window"))?;
        let document = dom.document.clone();
        let root = dom.root.clone();
        let loading = dom.get("loading").clone();
        let panel = dom.get("error").clone();
        let message = dom.get("error-message").clone();
        let reload = dom.get("reload").clone();
        let obscured: Vec<HtmlElement> =
            ["intro", "paused", "inspect", "safe", "won", "hud", "pause"]
                .into_iter()
                .map(|id| dom.get(id).clone())
                .collect();
        listeners.push(Listener::new(&window, "error", move |event| {
            if event.dyn_ref::<web_sys::ErrorEvent>().is_some_and(|error| {
                !error
                    .message()
                    .contains("Using exceptions for control flow")
            }) {
                failed.set(true);
                screen.set(Screen::Paused);
                for element in &obscured {
                    element.set_hidden(true);
                }
                let _ = root.set_attribute("data-screen", Screen::Paused.id());
                loading.set_hidden(true);
                panel.set_hidden(false);
                message.set_text_content(Some(&failure_message.borrow()));
                audio.borrow_mut().silence();
                if document.pointer_lock_element().is_some() {
                    document.exit_pointer_lock();
                }
                let _ = reload.focus();
                let _ = sender.send(BrowserCommand::Failure);
            }
        })?);
    }
    Ok(listeners)
}

/// 在真實手勢內同步請求 pointer lock，並吸收舊式 throw 與新版 Promise 拒絕。
fn request_lock(canvas: &HtmlElement) {
    let result = js_sys::Reflect::get(canvas, &JsValue::from_str("requestPointerLock"))
        .and_then(|method| method.dyn_into::<js_sys::Function>())
        .and_then(|method| method.call0(canvas));
    if let Ok(value) = result
        && let Ok(promise) = value.dyn_into::<js_sys::Promise>()
    {
        spawn_local(async move {
            let _ = JsFuture::from(promise).await;
        });
    }
}

/// 模態視窗的 Tab 循環包含語言與音效控制，避免焦點落到隱藏場景。
fn trap_focus(document: &Document, screen: Screen, key: &KeyboardEvent) {
    if !matches!(
        screen,
        Screen::Paused | Screen::Inspect(_) | Screen::Safe | Screen::Won
    ) {
        return;
    }
    let selector = format!(
        "#{} button:not([hidden]), #{} input, .header-actions button:not([hidden]), .header-actions select",
        screen.id(),
        screen.id()
    );
    let Ok(nodes) = document.query_selector_all(&selector) else {
        return;
    };
    let elements: Vec<HtmlElement> = (0..nodes.length())
        .filter_map(|i| nodes.item(i))
        .filter_map(|node| node.dyn_into().ok())
        .collect();
    if elements.is_empty() {
        return;
    }
    let current = document.active_element().and_then(|active| {
        elements
            .iter()
            .position(|element| element.is_same_node(Some(&active)))
    });
    let next = match current {
        Some(index) if key.shift_key() => (index + elements.len() - 1) % elements.len(),
        Some(index) => (index + 1) % elements.len(),
        None if key.shift_key() => elements.len() - 1,
        None => 0,
    };
    key.prevent_default();
    let _ = elements[next].focus();
}
