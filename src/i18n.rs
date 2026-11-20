use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

pub const ZH_HANT: &str = include_str!("../assets/locales/zh-Hant.ftl");
pub const EN: &str = include_str!("../assets/locales/en.ftl");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Locale {
    #[default]
    ZhHant,
    En,
}

impl Locale {
    /// 解析語言選單的值；未知的外部值不會改變目前語言。
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "zh-Hant" => Some(Self::ZhHant),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    /// 提供 HTML lang 與 Fluent 使用的 BCP 47 語言標籤。
    pub fn tag(self) -> &'static str {
        match self {
            Self::ZhHant => "zh-Hant",
            Self::En => "en",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKey {
    Restarted,
    LatchReleased,
    LatchLocked,
    DrawerOpened,
    DrawerClosed,
    PlateHint,
    SafeUnlocked,
    IncorrectCode,
    KeyTaken,
    DoorUnlocked,
    NeedKey,
    DoorAlreadyOpen,
    FoyerEntered,
    ClockSynced,
    ClockNotSynced,
    ClockHourTick,
    ClockMinuteTick,
    ClockAlreadySynced,
    LeverEngaged,
    LeversReset,
    FrontDoorUnlocked,
    FrontDoorLocked,
    FrontDoorAlreadyOpen,
    Escaped,
    SignalRoomEntered,
    SignalGateUnlocked,
    SignalGateLocked,
    SignalMirrorTurned,
    AmberReceived,
    BlueReceived,
    ObjectiveSignalRoom,
    ObjectiveSignalGate,
    TurnSignalMirror,
    ReadSignalManual,
    InspectSignalGate,
    ObjectiveSearch,
    ObjectiveLatch,
    ObjectiveSafe,
    ObjectiveKey,
    ObjectiveDoor,
    ObjectiveFoyer,
    ObjectiveFrontDoor,
    ObjectiveEscaped,
    ReadNote,
    ReadLetter,
    ViewSafe,
    EnterCode,
    LeaveRoom,
    OpenDoor,
    ViewPhotograph,
    OpenDrawer,
    CloseDrawer,
    InspectPlate,
    PickPaperweight,
    PickBook,
    PickBox,
    PickVase,
    DropPaperweight,
    DropBook,
    DropBox,
    DropVase,
    ToggleLamp,
    ToggleSwitch,
    InspectTeacups,
    InspectCrate,
    InspectBookshelf,
    InspectWeightBox,
    InspectMailStack,
    InspectPostageChart,
    ReadTimetable,
    ReadMoonMark,
    PullLeverOne,
    PullLeverTwo,
    PullLeverThree,
    PullLeverFour,
    TurnClockHour,
    TurnClockMinute,
    OpenFrontDoor,
    LeaveHouse,
    InspectMailbox,
    InspectCoatRack,
    ToggleFoyerLight,
    LampOn,
    LampOff,
    ChandelierOn,
    ChandelierOff,
    FoyerLightOn,
    FoyerLightOff,
    TeacupsNote,
    CrateNote,
    BookshelfNote,
    WeightBoxNote,
    MailStackNote,
    PostageChartNote,
    MailboxNote,
    CoatRackNote,
    MoonMarkTooBright,
    InventoryEmpty,
    InventoryKey,
    SafeHint,
}

impl MessageKey {
    /// 將領域呈現識別碼轉成 Fluent key，不把翻譯存入遊戲狀態。
    pub fn id(self) -> &'static str {
        match self {
            Self::Restarted => "notice-restarted",
            Self::LatchReleased => "notice-latch-released",
            Self::LatchLocked => "notice-latch-locked",
            Self::DrawerOpened => "notice-drawer-opened",
            Self::DrawerClosed => "notice-drawer-closed",
            Self::PlateHint => "notice-plate-hint",
            Self::SafeUnlocked => "notice-safe-unlocked",
            Self::IncorrectCode => "notice-incorrect-code",
            Self::KeyTaken => "notice-key-taken",
            Self::DoorUnlocked => "notice-door-unlocked",
            Self::NeedKey => "notice-need-key",
            Self::DoorAlreadyOpen => "notice-door-open",
            Self::FoyerEntered => "notice-foyer-entered",
            Self::ClockSynced => "notice-clock-synced",
            Self::ClockNotSynced => "notice-clock-not-synced",
            Self::ClockHourTick => "notice-clock-hour",
            Self::ClockMinuteTick => "notice-clock-minute",
            Self::ClockAlreadySynced => "notice-clock-running",
            Self::LeverEngaged => "notice-lever-engaged",
            Self::LeversReset => "notice-levers-reset",
            Self::FrontDoorUnlocked => "notice-front-door-unlocked",
            Self::FrontDoorLocked => "notice-front-door-locked",
            Self::FrontDoorAlreadyOpen => "notice-front-door-open",
            Self::Escaped => "notice-escaped",
            Self::SignalRoomEntered => "notice-signal-room-entered",
            Self::SignalGateUnlocked => "notice-signal-gate-unlocked",
            Self::SignalGateLocked => "notice-signal-gate-locked",
            Self::SignalMirrorTurned => "notice-signal-mirror-turned",
            Self::AmberReceived => "notice-amber-received",
            Self::BlueReceived => "notice-blue-received",
            Self::ObjectiveSignalRoom => "objective-signal-room",
            Self::ObjectiveSignalGate => "objective-signal-gate",
            Self::TurnSignalMirror => "target-signal-mirror",
            Self::ReadSignalManual => "target-signal-manual",
            Self::InspectSignalGate => "target-signal-gate",
            Self::ObjectiveSearch => "objective-search",
            Self::ObjectiveLatch => "objective-latch",
            Self::ObjectiveSafe => "objective-safe",
            Self::ObjectiveKey => "objective-key",
            Self::ObjectiveDoor => "objective-door",
            Self::ObjectiveFoyer => "objective-foyer",
            Self::ObjectiveFrontDoor => "objective-front-door",
            Self::ObjectiveEscaped => "objective-escaped",
            Self::ReadNote => "target-note",
            Self::ReadLetter => "target-letter",
            Self::ViewSafe => "target-safe-open",
            Self::EnterCode => "target-safe",
            Self::LeaveRoom => "target-leave",
            Self::OpenDoor => "target-door",
            Self::ViewPhotograph => "target-photograph",
            Self::OpenDrawer => "target-drawer-open",
            Self::CloseDrawer => "target-drawer-close",
            Self::InspectPlate => "target-plate",
            Self::PickPaperweight => "target-pick-paperweight",
            Self::PickBook => "target-pick-book",
            Self::PickBox => "target-pick-box",
            Self::PickVase => "target-pick-vase",
            Self::DropPaperweight => "target-drop-paperweight",
            Self::DropBook => "target-drop-book",
            Self::DropBox => "target-drop-box",
            Self::DropVase => "target-drop-vase",
            Self::ToggleLamp => "target-lamp",
            Self::ToggleSwitch => "target-switch",
            Self::InspectTeacups => "target-teacups",
            Self::InspectCrate => "target-crate",
            Self::InspectBookshelf => "target-bookshelf",
            Self::InspectWeightBox => "target-weight-box",
            Self::InspectMailStack => "target-mail-stack",
            Self::InspectPostageChart => "target-postage-chart",
            Self::ReadTimetable => "target-timetable",
            Self::ReadMoonMark => "target-moon-mark",
            Self::PullLeverOne => "target-lever-one",
            Self::PullLeverTwo => "target-lever-two",
            Self::PullLeverThree => "target-lever-three",
            Self::PullLeverFour => "target-lever-four",
            Self::TurnClockHour => "target-clock-hour",
            Self::TurnClockMinute => "target-clock-minute",
            Self::OpenFrontDoor => "target-front-door",
            Self::LeaveHouse => "target-leave-house",
            Self::InspectMailbox => "target-mailbox",
            Self::InspectCoatRack => "target-coat-rack",
            Self::ToggleFoyerLight => "target-foyer-light",
            Self::LampOn => "notice-lamp-on",
            Self::LampOff => "notice-lamp-off",
            Self::ChandelierOn => "notice-chandelier-on",
            Self::ChandelierOff => "notice-chandelier-off",
            Self::FoyerLightOn => "notice-foyer-light-on",
            Self::FoyerLightOff => "notice-foyer-light-off",
            Self::TeacupsNote => "notice-teacups",
            Self::CrateNote => "notice-crate",
            Self::BookshelfNote => "notice-bookshelf",
            Self::WeightBoxNote => "notice-weight-box",
            Self::MailStackNote => "notice-mail-stack",
            Self::PostageChartNote => "notice-postage-chart",
            Self::MailboxNote => "notice-mailbox",
            Self::CoatRackNote => "notice-coat-rack",
            Self::MoonMarkTooBright => "notice-moon-mark-bright",
            Self::InventoryEmpty => "inventory-empty",
            Self::InventoryKey => "inventory-key",
            Self::SafeHint => "safe-hint",
        }
    }
}

pub struct Localizer {
    locale: Locale,
    chinese: FluentBundle<FluentResource>,
    english: FluentBundle<FluentResource>,
}

impl Localizer {
    /// 建立兩份已驗證的內嵌翻譯；語法錯誤會以 Result 回報。
    pub fn new(locale: Locale) -> Result<Self, String> {
        Ok(Self {
            locale,
            chinese: bundle(Locale::ZhHant, ZH_HANT)?,
            english: bundle(Locale::En, EN)?,
        })
    }

    /// 只切換格式化語言，不修改遊戲狀態或事件序號。
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    /// 回傳目前 UI 語言。
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// 格式化訊息；缺漏或格式化錯誤以預設繁中回退，仍失敗時顯示 key。
    pub fn text(&self, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        self.try_text(key, args).unwrap_or_else(|_| {
            format_message(&self.chinese, key, args).unwrap_or_else(|_| format!("[{key}]"))
        })
    }

    /// 嚴格格式化供驗證與診斷使用，缺漏 key、變數與 Fluent 錯誤都會回報。
    pub fn try_text(&self, key: &str, args: Option<&FluentArgs<'_>>) -> Result<String, String> {
        let selected = match self.locale {
            Locale::ZhHant => &self.chinese,
            Locale::En => &self.english,
        };
        format_message(selected, key, args)
    }

    /// 格式化型別化的動態遊戲文案。
    pub fn message(&self, key: MessageKey) -> String {
        self.text(key.id(), None)
    }
}

/// 解析內嵌 Fluent 資源，關閉雙向隔離符以適應現有數字及 UI 標記。
fn bundle(locale: Locale, source: &str) -> Result<FluentBundle<FluentResource>, String> {
    let language: LanguageIdentifier = locale.tag().parse().map_err(|e| format!("{e}"))?;
    let resource = FluentResource::try_new(source.to_owned()).map_err(|(_, e)| format!("{e:?}"))?;
    let mut bundle = FluentBundle::new(vec![language]);
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|e| format!("{e:?}"))?;
    Ok(bundle)
}

/// 取得訊息的值並保留格式化錯誤，不將不完整結果當成成功。
fn format_message(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: Option<&FluentArgs<'_>>,
) -> Result<String, String> {
    let pattern = bundle
        .get_message(key)
        .and_then(|m| m.value())
        .ok_or_else(|| format!("Missing Fluent message {key}"))?;
    let mut errors = Vec::new();
    let value = bundle
        .format_pattern(pattern, args, &mut errors)
        .into_owned();
    if errors.is_empty() {
        Ok(value)
    } else {
        Err(format!("{key}: {errors:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// 只擷取資源的頂層訊息識別碼，不把 select 分支當作 key。
    fn keys(source: &str) -> BTreeSet<&str> {
        source
            .lines()
            .filter(|line| !line.starts_with(char::is_whitespace))
            .filter_map(|line| line.split_once(" = ").map(|(key, _)| key))
            .collect()
    }

    #[test]
    fn both_locales_have_identical_valid_messages() {
        assert_eq!(keys(ZH_HANT), keys(EN));
        for locale in [Locale::ZhHant, Locale::En] {
            let localizer = Localizer::new(locale).unwrap();
            let mut args = FluentArgs::new();
            args.set("minutes", 3);
            args.set("seconds", 5);
            for key in keys(ZH_HANT) {
                assert!(
                    localizer.try_text(key, Some(&args)).is_ok(),
                    "{locale:?}: {key}"
                );
            }
        }
    }

    #[test]
    fn runtime_switch_formats_variables_and_english_plural() {
        let mut localizer = Localizer::new(Locale::En).unwrap();
        let mut args = FluentArgs::new();
        args.set("minutes", 1);
        assert_eq!(
            localizer.text("intro-duration", Some(&args)),
            "About 1 minute"
        );
        args.set("minutes", 3);
        assert_eq!(
            localizer.text("intro-duration", Some(&args)),
            "About 3 minutes"
        );
        args.set("seconds", 1);
        assert_eq!(
            localizer.text("win-redirect", Some(&args)),
            "Redirecting to GitHub in 1 second…"
        );
        args.set("seconds", 5);
        assert_eq!(
            localizer.text("win-redirect", Some(&args)),
            "Redirecting to GitHub in 5 seconds…"
        );
        localizer.set_locale(Locale::ZhHant);
        assert_eq!(localizer.text("intro-duration", Some(&args)), "約 3 分鐘");
        assert_eq!(
            localizer.text("win-redirect", Some(&args)),
            "5 秒後將自動前往 GitHub…"
        );
        assert_eq!(localizer.message(MessageKey::KeyTaken), "已收起黃銅鑰匙。");
    }

    #[test]
    fn invalid_resources_missing_keys_and_arguments_report_errors() {
        assert!(bundle(Locale::En, "bad = { ").is_err());
        let localizer = Localizer::new(Locale::En).unwrap();
        assert!(localizer.try_text("missing-key", None).is_err());
        assert_eq!(localizer.text("missing-key", None), "[missing-key]");
        assert!(localizer.try_text("intro-duration", None).is_err());
        let fallback = Localizer {
            locale: Locale::En,
            chinese: bundle(Locale::ZhHant, "greeting = 預設文字").unwrap(),
            english: bundle(Locale::En, "other = Other").unwrap(),
        };
        assert_eq!(fallback.text("greeting", None), "預設文字");
        assert_eq!(Locale::parse("fr"), None);
    }
}
