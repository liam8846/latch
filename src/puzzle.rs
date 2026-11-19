pub const REQUIRED_PLATE_MASS_KG: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lever {
    Coast,
    Harbor,
    Station,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Progress {
    #[default]
    Searching,
    LatchReleased,
    SafeOpen,
    KeyHeld,
    DoorOpen,
    FoyerEntered,
    FoyerFirst,
    FoyerSecond,
    FrontDoorOpen,
    Escaped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ReleaseLatch,
    EnterCode([u8; 4]),
    TakeKey,
    UnlockDoor,
    EnterFoyer,
    PullLever(Lever),
    Leave,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    LatchReleased,
    LatchLocked,
    SafeUnlocked,
    IncorrectCode,
    KeyTaken,
    DoorUnlocked,
    NeedKey,
    FoyerEntered,
    LeverEngaged,
    LeversReset,
    FrontDoorUnlocked,
    Escaped,
    Restarted,
    Unchanged,
}

impl Progress {
    /// 套用單一玩家動作；依序完成書房天平與密碼鎖、無縫步入玄關並依站序拉下三道黃銅拉桿。
    pub fn apply(&mut self, action: Action) -> Reply {
        let (next, reply) = match (*self, action) {
            (_, Action::Restart) => (Self::Searching, Reply::Restarted),
            (Self::Searching, Action::ReleaseLatch) => (Self::LatchReleased, Reply::LatchReleased),
            (Self::Searching, Action::EnterCode(_)) => (Self::Searching, Reply::LatchLocked),
            (Self::LatchReleased, Action::EnterCode([1, 9, 4, 7])) => {
                (Self::SafeOpen, Reply::SafeUnlocked)
            }
            (Self::LatchReleased, Action::EnterCode(_)) => {
                (Self::LatchReleased, Reply::IncorrectCode)
            }
            (Self::SafeOpen, Action::TakeKey) => (Self::KeyHeld, Reply::KeyTaken),
            (Self::KeyHeld, Action::UnlockDoor) => (Self::DoorOpen, Reply::DoorUnlocked),
            (Self::Searching | Self::LatchReleased | Self::SafeOpen, Action::UnlockDoor) => {
                (*self, Reply::NeedKey)
            }
            (Self::DoorOpen, Action::EnterFoyer) => (Self::FoyerEntered, Reply::FoyerEntered),
            (Self::FoyerEntered, Action::PullLever(Lever::Coast)) => {
                (Self::FoyerFirst, Reply::LeverEngaged)
            }
            (Self::FoyerFirst, Action::PullLever(Lever::Harbor)) => {
                (Self::FoyerSecond, Reply::LeverEngaged)
            }
            (Self::FoyerSecond, Action::PullLever(Lever::Station)) => {
                (Self::FrontDoorOpen, Reply::FrontDoorUnlocked)
            }
            (Self::FoyerEntered | Self::FoyerFirst | Self::FoyerSecond, Action::PullLever(_)) => {
                (Self::FoyerEntered, Reply::LeversReset)
            }
            (Self::FrontDoorOpen, Action::Leave) => (Self::Escaped, Reply::Escaped),
            _ => (*self, Reply::Unchanged),
        };
        *self = next;
        reply
    }

    /// 表示天平已解開書房保險箱暗扣，供機關動畫與互動判斷取用。
    pub fn latch_is_released(self) -> bool {
        self != Self::Searching
    }

    /// 表示書房保險箱門已解鎖，供場景動畫取用。
    pub fn safe_is_open(self) -> bool {
        !matches!(self, Self::Searching | Self::LatchReleased)
    }

    /// 表示書房通往玄關的木門已解鎖，供碰撞及場景動畫取用。
    pub fn door_is_open(self) -> bool {
        matches!(
            self,
            Self::DoorOpen
                | Self::FoyerEntered
                | Self::FoyerFirst
                | Self::FoyerSecond
                | Self::FrontDoorOpen
                | Self::Escaped
        )
    }

    /// 表示指定玄關黃銅拉桿目前處於扣下位置，完全由唯一進度導出。
    pub fn lever_is_down(self, lever: Lever) -> bool {
        match lever {
            Lever::Coast => matches!(
                self,
                Self::FoyerFirst | Self::FoyerSecond | Self::FrontDoorOpen | Self::Escaped
            ),
            Lever::Harbor => matches!(
                self,
                Self::FoyerSecond | Self::FrontDoorOpen | Self::Escaped
            ),
            Lever::Station => matches!(self, Self::FrontDoorOpen | Self::Escaped),
        }
    }

    /// 表示玄關臨街大門已解鎖，供門板碰撞與離場判定取用。
    pub fn front_door_is_open(self) -> bool {
        matches!(self, Self::FrontDoorOpen | Self::Escaped)
    }

    /// 表示玩家已從書房無縫走進玄關區域。
    pub fn in_foyer(self) -> bool {
        matches!(
            self,
            Self::FoyerEntered
                | Self::FoyerFirst
                | Self::FoyerSecond
                | Self::FrontDoorOpen
                | Self::Escaped
        )
    }
}

/// 判斷天平秤盤上的物件總質量（公斤）是否達到觸發暗扣的門檻；拒絕 NaN 與無窮值。
///
/// ```
/// assert!(!latch::puzzle::plate_triggered(1.49));
/// assert!(latch::puzzle::plate_triggered(1.5));
/// assert!(!latch::puzzle::plate_triggered(f32::NAN));
/// ```
pub fn plate_triggered(total_mass_kg: f32) -> bool {
    total_mass_kg.is_finite() && total_mass_kg >= REQUIRED_PLATE_MASS_KG
}

/// 將四個 ASCII 數字解析為密碼，拒絕長度錯誤、空白與非 ASCII 數字。
///
/// ```
/// assert_eq!(latch::puzzle::parse_code("1947"), Some([1, 9, 4, 7]));
/// assert_eq!(latch::puzzle::parse_code("194"), None);
/// ```
pub fn parse_code(input: &str) -> Option<[u8; 4]> {
    let bytes: [u8; 4] = input.as_bytes().try_into().ok()?;
    bytes
        .iter()
        .all(u8::is_ascii_digit)
        .then(|| bytes.map(|b| b - b'0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_STAGES: [Progress; 10] = [
        Progress::Searching,
        Progress::LatchReleased,
        Progress::SafeOpen,
        Progress::KeyHeld,
        Progress::DoorOpen,
        Progress::FoyerEntered,
        Progress::FoyerFirst,
        Progress::FoyerSecond,
        Progress::FrontDoorOpen,
        Progress::Escaped,
    ];

    #[test]
    fn player_completes_study_and_seamless_foyer_then_restarts() {
        let mut game = Progress::default();
        for (action, reply) in [
            (Action::ReleaseLatch, Reply::LatchReleased),
            (Action::EnterCode([1, 9, 4, 7]), Reply::SafeUnlocked),
            (Action::TakeKey, Reply::KeyTaken),
            (Action::UnlockDoor, Reply::DoorUnlocked),
            (Action::EnterFoyer, Reply::FoyerEntered),
            (Action::PullLever(Lever::Coast), Reply::LeverEngaged),
            (Action::PullLever(Lever::Harbor), Reply::LeverEngaged),
            (Action::PullLever(Lever::Station), Reply::FrontDoorUnlocked),
            (Action::Leave, Reply::Escaped),
            (Action::Restart, Reply::Restarted),
        ] {
            assert_eq!(game.apply(action), reply);
        }
        assert_eq!(game, Progress::Searching);
    }

    #[test]
    fn wrong_lever_order_resets_all_levers_to_foyer_entered() {
        for (start, wrong) in [
            (Progress::FoyerEntered, Lever::Harbor),
            (Progress::FoyerEntered, Lever::Station),
            (Progress::FoyerFirst, Lever::Coast),
            (Progress::FoyerFirst, Lever::Station),
            (Progress::FoyerSecond, Lever::Coast),
            (Progress::FoyerSecond, Lever::Harbor),
        ] {
            let mut game = start;
            assert_eq!(game.apply(Action::PullLever(wrong)), Reply::LeversReset);
            assert_eq!(game, Progress::FoyerEntered);
            assert!(!game.lever_is_down(Lever::Coast));
            assert!(!game.lever_is_down(Lever::Harbor));
            assert!(!game.lever_is_down(Lever::Station));
        }
    }

    #[test]
    fn safe_requires_released_latch_and_rejects_wrong_codes() {
        let mut locked = Progress::Searching;
        assert_eq!(
            locked.apply(Action::EnterCode([1, 9, 4, 7])),
            Reply::LatchLocked
        );
        assert_eq!(locked, Progress::Searching);
        for code in [[0, 0, 0, 0], [1, 9, 4, 8], [9, 9, 9, 9]] {
            let mut game = Progress::LatchReleased;
            assert_eq!(game.apply(Action::EnterCode(code)), Reply::IncorrectCode);
            assert_eq!(game, Progress::LatchReleased);
        }
    }

    #[test]
    fn every_wrong_order_and_repeated_action_preserves_invariants() {
        for progress in ALL_STAGES {
            for action in [
                Action::ReleaseLatch,
                Action::EnterCode([1, 9, 4, 7]),
                Action::EnterCode([0; 4]),
                Action::TakeKey,
                Action::UnlockDoor,
                Action::EnterFoyer,
                Action::PullLever(Lever::Coast),
                Action::PullLever(Lever::Harbor),
                Action::PullLever(Lever::Station),
                Action::Leave,
            ] {
                let mut game = progress;
                let reply = game.apply(action);
                if matches!(
                    reply,
                    Reply::Unchanged | Reply::NeedKey | Reply::LatchLocked
                ) {
                    assert_eq!(game, progress);
                }
            }
            let mut game = progress;
            assert_eq!(game.apply(Action::Restart), Reply::Restarted);
            assert_eq!(game, Progress::Searching);
        }
    }

    #[test]
    fn external_code_input_has_exact_ascii_boundaries() {
        assert_eq!(parse_code("1947"), Some([1, 9, 4, 7]));
        assert_eq!(parse_code("0000"), Some([0; 4]));
        for input in ["", "194", "19477", " 1947", "19a7", "１９４７", "-947"] {
            assert_eq!(parse_code(input), None);
        }
    }

    #[test]
    fn plate_weight_threshold_and_non_finite_values() {
        assert!(!plate_triggered(-1.0));
        assert!(!plate_triggered(0.0));
        assert!(!plate_triggered(1.499));
        assert!(plate_triggered(REQUIRED_PLATE_MASS_KG));
        assert!(plate_triggered(2.2));
        assert!(!plate_triggered(f32::NAN));
        assert!(!plate_triggered(f32::INFINITY));
        assert!(!plate_triggered(f32::NEG_INFINITY));
    }

    #[test]
    fn visual_state_matches_each_stage() {
        for progress in ALL_STAGES {
            assert_eq!(
                progress.latch_is_released(),
                progress != Progress::Searching
            );
            assert_eq!(
                progress.safe_is_open(),
                !matches!(progress, Progress::Searching | Progress::LatchReleased)
            );
            assert_eq!(
                progress.door_is_open(),
                !matches!(
                    progress,
                    Progress::Searching
                        | Progress::LatchReleased
                        | Progress::SafeOpen
                        | Progress::KeyHeld
                )
            );
            assert_eq!(
                progress.front_door_is_open(),
                matches!(progress, Progress::FrontDoorOpen | Progress::Escaped)
            );
            assert_eq!(
                progress.in_foyer(),
                matches!(
                    progress,
                    Progress::FoyerEntered
                        | Progress::FoyerFirst
                        | Progress::FoyerSecond
                        | Progress::FrontDoorOpen
                        | Progress::Escaped
                )
            );
            assert_eq!(
                progress.lever_is_down(Lever::Station),
                progress.front_door_is_open()
            );
        }
    }
}
