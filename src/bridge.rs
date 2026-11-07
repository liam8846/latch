use crate::{Screen, Session, interaction::Focus, player::Player};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Command(pub String);

#[cfg(target_arch = "wasm32")]
/// 取得靜態 HTML 介面的根節點；頁面尚未就緒時不會中斷遊戲。
fn root() -> Option<web_sys::Element> {
    web_sys::window()?
        .document()?
        .get_element_by_id("interface")
}

/// 讀取並清空單一動作信箱；輸入內容仍由互動系統驗證。
pub fn read_command(mut command: ResMut<Command>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(root) = root() {
        command.0 = root.get_attribute("data-action").unwrap_or_default();
        let _ = root.remove_attribute("data-action");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        command.0.clear();
    }
}

/// 回報瀏覽器是否已鎖定滑鼠；原生開發版本使用未鎖定的滑鼠輸入。
pub fn pointer_locked() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.document())
            .is_some_and(|d| d.pointer_lock_element().is_some())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

/// 將唯讀呈現資料發布至 DOM，UI 不擁有謎題進度，座標使用公尺、角度使用弧度。
pub fn publish(session: Res<Session>, focus: Res<Focus>, player: Query<(&Transform, &Player)>) {
    let screen = match session.screen {
        Screen::Intro => "intro",
        Screen::Playing => "playing",
        Screen::Paused => "paused",
        Screen::Note => "note",
        Screen::Safe => "safe",
        Screen::Won => "won",
    };
    #[cfg(target_arch = "wasm32")]
    if let Some(root) = root() {
        let stage = format!("{:?}", session.progress);
        let target = focus
            .0
            .map(|target| target.label(session.progress))
            .unwrap_or("");
        for (name, value) in [
            ("screen", screen),
            ("stage", stage.as_str()),
            ("objective", session.progress.objective()),
            ("target", target),
            ("message", session.message),
            ("sound", session.sound),
            (
                "inventory",
                if matches!(
                    session.progress,
                    latch::puzzle::Progress::KeyHeld
                        | latch::puzzle::Progress::DoorOpen
                        | latch::puzzle::Progress::Escaped
                ) {
                    "黃銅鑰匙"
                } else {
                    "尚未取得物品"
                },
            ),
        ] {
            let attribute = format!("data-{name}");
            if root.get_attribute(&attribute).as_deref() != Some(value) {
                let _ = root.set_attribute(&attribute, value);
            }
        }
        let _ = root.set_attribute("data-revision", &session.revision.to_string());
        if let Ok((transform, look)) = player.single() {
            let _ = root.set_attribute(
                "data-position",
                &format!(
                    "{:.2},{:.2},{:.2}",
                    transform.translation.x, transform.translation.y, transform.translation.z
                ),
            );
            let _ = root.set_attribute("data-look", &format!("{:.3},{:.3}", look.yaw, look.pitch));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (screen, focus, player);
    }
}
