use latch::i18n::{Locale, Localizer};
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement, HtmlInputElement};

#[derive(Clone)]
pub struct FailurePanel {
    document: Document,
}

impl FailurePanel {
    /// 保留文件控制代碼，讓錯誤呈現不依賴 Bevy 的下一幀。
    pub fn new(document: Document) -> Self {
        Self { document }
    }

    /// 同步移除遊戲控制、釋放輸入並顯示純文字錯誤；詳細內容不解析 HTML。
    pub fn show(&self, message: &str, details: Option<&str>) {
        for id in [
            "loading", "intro", "paused", "inspect", "safe", "won", "hud", "pause",
        ] {
            self.hidden(id, true);
        }
        self.hidden("shade", false);
        self.hidden("error", false);
        self.hidden("error-diagnostics", details.is_none());
        self.text("error-message", message);
        self.text("error-details", details.unwrap_or_default());
        for id in ["language", "sound"] {
            if let Some(element) = self.document.get_element_by_id(id) {
                let _ = element.set_attribute("disabled", "");
                let _ = element.set_attribute("hidden", "");
            }
        }
        if let Some(root) = self.document.get_element_by_id("interface") {
            let _ = root.set_attribute("data-screen", "failed");
        }
        if let Some(code) = self
            .document
            .get_element_by_id("code")
            .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
        {
            code.set_value("");
        }
        if self.document.pointer_lock_element().is_some() {
            self.document.exit_pointer_lock();
        }
        if let Some(reload) = self
            .document
            .get_element_by_id("reload")
            .and_then(|element| element.dyn_into::<HtmlElement>().ok())
        {
            let _ = reload.focus();
        }
    }

    /// 設定既有節點的可見狀態；初始化尚未完成時允許節點不存在。
    fn hidden(&self, id: &str, hidden: bool) {
        if let Some(element) = self.document.get_element_by_id(id) {
            if hidden {
                let _ = element.set_attribute("hidden", "");
            } else {
                let _ = element.remove_attribute("hidden");
            }
        }
    }

    /// 以 textContent 保留瀏覽器提供的診斷文字，不執行其中的標記。
    fn text(&self, id: &str, value: &str) {
        if let Some(element) = self.document.get_element_by_id(id) {
            element.set_text_content(Some(value));
        }
    }
}

/// UI 尚未建立時仍直接顯示 renderer 錯誤，使用目前文件語言的 Fluent 文案。
pub fn renderer_fallback(details: &str) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let locale = document
        .document_element()
        .and_then(|html| html.get_attribute("lang"))
        .as_deref()
        .and_then(Locale::parse)
        .unwrap_or(Locale::ZhHant);
    let message = Localizer::new(locale)
        .map(|localizer| localizer.text("error-renderer", None))
        .unwrap_or_else(|_| {
            "繪圖失敗，請展開診斷資訊。 / Rendering failed. Open diagnostic details.".to_owned()
        });
    FailurePanel::new(document).show(&message, Some(details));
}
