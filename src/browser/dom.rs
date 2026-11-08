use super::failure::FailurePanel;
use fluent_bundle::FluentArgs;
use latch::i18n::Localizer;
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, HtmlElement, HtmlInputElement, HtmlSelectElement};

pub struct Dom {
    pub document: Document,
    pub root: HtmlElement,
    pub canvas: HtmlElement,
    pub code: HtmlInputElement,
    pub language: HtmlSelectElement,
    pub failure: FailurePanel,
    elements: BTreeMap<&'static str, HtmlElement>,
    translations: Vec<Element>,
}

impl Dom {
    /// 一次驗證 HTML 邊界的必要節點；之後的固定 ID 由本模組保證存在。
    pub fn new(document: Document) -> Result<Self, JsValue> {
        let root = element(&document, "interface")?;
        let canvas = element(&document, "game")?;
        let code = element(&document, "code")?.dyn_into()?;
        let language = element(&document, "language")?.dyn_into()?;
        let mut elements = BTreeMap::new();
        for id in [
            "loading",
            "error",
            "error-message",
            "reload",
            "intro",
            "paused",
            "inspect",
            "safe",
            "won",
            "shade",
            "hud",
            "pause",
            "objective",
            "inventory",
            "target",
            "interact",
            "sound",
            "begin",
            "resume",
            "restart-paused",
            "restart-won",
            "close-inspect",
            "close-inspect-footer",
            "close-safe",
            "code-form",
            "toast",
            "safe-message",
            "intro-duration",
            "inspect-title",
            "inspect-eyebrow",
            "inspect-copy",
            "inspect-caption",
            "inspect-image",
            "take-key",
        ] {
            elements.insert(id, element(&document, id)?);
        }
        let nodes =
            document.query_selector_all("[data-l10n],[data-l10n-aria],[data-l10n-content]")?;
        let translations = (0..nodes.length())
            .filter_map(|i| nodes.item(i))
            .filter_map(|node| node.dyn_into().ok())
            .collect();
        Ok(Self {
            failure: FailurePanel::new(document.clone()),
            document,
            root,
            canvas,
            code,
            language,
            elements,
            translations,
        })
    }

    /// 存取建構時已驗證存在的固定節點。
    pub fn get(&self, id: &str) -> &HtmlElement {
        &self.elements[id]
    }

    /// 只在內容改變時更新文字，避免每幀重複觸發 live region。
    pub fn text(&self, id: &str, value: &str) {
        set_text(self.get(id), value);
    }

    /// 切換既有節點顯示狀態。
    pub fn hidden(&self, id: &str, hidden: bool) {
        let element = self.get(id);
        if element.hidden() != hidden {
            element.set_hidden(hidden);
        }
    }

    /// 翻譯靜態文字與無障礙屬性；不重建節點或改變輸入值及焦點。
    pub fn translate(&self, localizer: &Localizer) {
        for element in &self.translations {
            if let Some(key) = element.get_attribute("data-l10n") {
                set_text(element, &localizer.text(&key, None));
            }
            for (source, target) in [
                ("data-l10n-aria", "aria-label"),
                ("data-l10n-content", "content"),
            ] {
                if let Some(key) = element.get_attribute(source) {
                    let _ = element.set_attribute(target, &localizer.text(&key, None));
                }
            }
        }
        self.document.set_title(&localizer.text("page-title", None));
        if let Some(html) = self.document.document_element() {
            let _ = html.set_attribute("lang", localizer.locale().tag());
        }
        let mut args = FluentArgs::new();
        args.set("minutes", 5);
        self.text(
            "intro-duration",
            &localizer.text("intro-duration", Some(&args)),
        );
        self.language.set_value(localizer.locale().tag());
    }
}

/// 將 DOM 節點解析為可聚焦的 HTML 元素，缺漏時回報可診斷的 ID。
fn element(document: &Document, id: &str) -> Result<HtmlElement, JsValue> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Missing #{id}")))?
        .dyn_into()
        .map_err(|_| JsValue::from_str(&format!("Invalid #{id}")))
}

/// 更新純文字，翻譯資源不會解析成可執行的 HTML。
fn set_text(element: &Element, text: &str) {
    if element.text_content().as_deref() != Some(text) {
        element.set_text_content(Some(text));
    }
}
