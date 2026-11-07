use crate::{Session, interaction::Focus, player::Player, session::UiCommands};
use bevy::prelude::*;

/// 建立瀏覽器 NonSend UI；原生版本保留純場景開發路徑。
pub fn setup(_world: &mut World) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::browser::BrowserUi::new() {
            Ok(ui) => _world.insert_non_send(ui),
            Err(error) => {
                web_sys::console::error_1(&error);
                if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                    if let Some(loading) = document.get_element_by_id("loading") {
                        let _ = loading.set_attribute("hidden", "");
                    }
                    if let Some(panel) = document.get_element_by_id("error") {
                        let _ = panel.remove_attribute("hidden");
                    }
                    if let Some(message) = document.get_element_by_id("error-message") {
                        message.set_text_content(Some("介面初始化失敗，請重新載入。 / The interface could not start. Please reload."));
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
/// 將瀏覽器 callback 的型別化佇列移入本幀的 ECS 輸入。
pub fn read_commands(
    mut commands: ResMut<UiCommands>,
    ui: Option<NonSendMut<crate::browser::BrowserUi>>,
) {
    if let Some(mut ui) = ui {
        ui.read_commands(&mut commands);
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 原生場景開發沒有 HTML 命令來源。
pub fn read_commands(_commands: ResMut<UiCommands>) {}

/// 回報瀏覽器是否已鎖定滑鼠；原生開發版本直接讀取滑鼠輸入。
pub fn pointer_locked() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.document())
            .is_some_and(|document| document.pointer_lock_element().is_some())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

/// 瀏覽器以整個文件焦點判斷離開視窗，選擇語言不會改變遊戲 Session。
pub fn window_focused(native_focused: bool) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = native_focused;
        web_sys::window()
            .and_then(|window| window.document())
            .is_some_and(|document| document.has_focus().unwrap_or(false))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native_focused
    }
}

/// 只有 canvas 接收移動與視角按鍵，操作語言選單時會釋放移動輸入。
pub fn input_focused() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .is_some_and(|element| element.id() == "game")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

#[cfg(target_arch = "wasm32")]
/// 呈現目前唯讀狀態；UI 直接更新 DOM，沒有 MutationObserver 或命令信箱。
pub fn publish(
    session: Res<Session>,
    focus: Res<Focus>,
    player: Query<(&Transform, &Player)>,
    time: Res<Time<Real>>,
    ui: Option<NonSendMut<crate::browser::BrowserUi>>,
) {
    if let Some(mut ui) = ui {
        ui.render(
            &session,
            &focus,
            player.single().ok(),
            time.elapsed_secs_f64(),
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// 原生版本僅保持場景與遊戲系統運作，不建立 HTML 呈現。
pub fn publish(session: Res<Session>, _focus: Res<Focus>, _player: Query<(&Transform, &Player)>) {
    let _ = session.screen.id();
}
