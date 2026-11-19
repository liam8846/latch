pub const REQUIRED_PLATE_MASS_KG: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lever {
    One,
    Two,
    Three,
    Four,
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
    FoyerPowered,
    FoyerStep1,
    FoyerStep2,
    FoyerStep3,
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
    EngageFoyerChain,
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
    FoyerChainEngaged,
    FoyerChainSlack,
    LeverEngaged,
    LeversReset,
    FrontDoorUnlocked,
    Escaped,
    Restarted,
    Unchanged,
}

impl Progress {
    /// 套用單一玩家動作；書房完成後無縫進入玄關，須先以重物繃緊銅鏈，再依回程順序 IV → II → I → III 拉下四道拉桿。
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
            (Self::DoorOpen | Self::FoyerEntered, Action::EngageFoyerChain) => {
                (Self::FoyerPowered, Reply::FoyerChainEngaged)
            }
            (Self::DoorOpen | Self::FoyerEntered, Action::PullLever(_)) => {
                (*self, Reply::FoyerChainSlack)
            }
            (Self::FoyerPowered, Action::PullLever(Lever::Four)) => {
                (Self::FoyerStep1, Reply::LeverEngaged)
            }
            (Self::FoyerStep1, Action::PullLever(Lever::Two)) => {
                (Self::FoyerStep2, Reply::LeverEngaged)
            }
            (Self::FoyerStep2, Action::PullLever(Lever::One)) => {
                (Self::FoyerStep3, Reply::LeverEngaged)
            }
            (Self::FoyerStep3, Action::PullLever(Lever::Three)) => {
                (Self::FrontDoorOpen, Reply::FrontDoorUnlocked)
            }
            (
                Self::FoyerPowered | Self::FoyerStep1 | Self::FoyerStep2 | Self::FoyerStep3,
                Action::PullLever(_),
            ) => (Self::FoyerPowered, Reply::LeversReset),
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
        !matches!(
            self,
            Self::Searching | Self::LatchReleased | Self::SafeOpen | Self::KeyHeld
        )
    }

    /// 表示玄關拉桿箱的銅鏈已受重物繃緊，拉桿可正常咬合。
    pub fn foyer_chain_engaged(self) -> bool {
        matches!(
            self,
            Self::FoyerPowered
                | Self::FoyerStep1
                | Self::FoyerStep2
                | Self::FoyerStep3
                | Self::FrontDoorOpen
                | Self::Escaped
        )
    }

    /// 表示指定玄關黃銅拉桿目前處於扣下位置，完全由唯一進度導出。
    pub fn lever_is_down(self, lever: Lever) -> bool {
        match lever {
            Lever::Four => matches!(
                self,
                Self::FoyerStep1
                    | Self::FoyerStep2
                    | Self::FoyerStep3
                    | Self::FrontDoorOpen
                    | Self::Escaped
            ),
            Lever::Two => matches!(
                self,
                Self::FoyerStep2 | Self::FoyerStep3 | Self::FrontDoorOpen | Self::Escaped
            ),
            Lever::One => matches!(self, Self::FoyerStep3 | Self::FrontDoorOpen | Self::Escaped),
            Lever::Three => matches!(self, Self::FrontDoorOpen | Self::Escaped),
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
                | Self::FoyerPowered
                | Self::FoyerStep1
                | Self::FoyerStep2
                | Self::FoyerStep3
                | Self::FrontDoorOpen
                | Self::Escaped
        )
    }
}

/// 判斷秤盤或承重托台上的物件總質量（公斤）是否達標；拒絕 NaN 與無窮值。
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

    const ALL_STAGES: [Progress; 12] = [
        Progress::Searching,
        Progress::LatchReleased,
        Progress::SafeOpen,
        Progress::KeyHeld,
        Progress::DoorOpen,
        Progress::FoyerEntered,
        Progress::FoyerPowered,
        Progress::FoyerStep1,
        Progress::FoyerStep2,
        Progress::FoyerStep3,
        Progress::FrontDoorOpen,
        Progress::Escaped,
    ];

    #[test]
    fn player_completes_study_and_harder_foyer_then_restarts() {
        let mut game = Progress::default();
        for (action, reply) in [
            (Action::ReleaseLatch, Reply::LatchReleased),
            (Action::EnterCode([1, 9, 4, 7]), Reply::SafeUnlocked),
            (Action::TakeKey, Reply::KeyTaken),
            (Action::UnlockDoor, Reply::DoorUnlocked),
            (Action::EnterFoyer, Reply::FoyerEntered),
            (Action::PullLever(Lever::Four), Reply::FoyerChainSlack),
            (Action::EngageFoyerChain, Reply::FoyerChainEngaged),
            (Action::PullLever(Lever::Four), Reply::LeverEngaged),
            (Action::PullLever(Lever::Two), Reply::LeverEngaged),
            (Action::PullLever(Lever::One), Reply::LeverEngaged),
            (Action::PullLever(Lever::Three), Reply::FrontDoorUnlocked),
            (Action::Leave, Reply::Escaped),
            (Action::Restart, Reply::Restarted),
        ] {
            assert_eq!(game.apply(action), reply);
        }
        assert_eq!(game, Progress::Searching);
    }

    #[test]
    fn wrong_lever_order_resets_all_four_levers_to_foyer_powered() {
        for (start, wrong) in [
            (Progress::FoyerPowered, Lever::One),
            (Progress::FoyerPowered, Lever::Two),
            (Progress::FoyerPowered, Lever::Three),
            (Progress::FoyerStep1, Lever::One),
            (Progress::FoyerStep1, Lever::Three),
            (Progress::FoyerStep1, Lever::Four),
            (Progress::FoyerStep2, Lever::Two),
            (Progress::FoyerStep2, Lever::Three),
            (Progress::FoyerStep2, Lever::Four),
            (Progress::FoyerStep3, Lever::One),
            (Progress::FoyerStep3, Lever::Two),
            (Progress::FoyerStep3, Lever::Four),
        ] {
            let mut game = start;
            assert_eq!(game.apply(Action::PullLever(wrong)), Reply::LeversReset);
            assert_eq!(game, Progress::FoyerPowered);
            for lever in [Lever::One, Lever::Two, Lever::Three, Lever::Four] {
                assert!(!game.lever_is_down(lever));
            }
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
                Action::EngageFoyerChain,
                Action::PullLever(Lever::One),
                Action::PullLever(Lever::Two),
                Action::PullLever(Lever::Three),
                Action::PullLever(Lever::Four),
                Action::Leave,
            ] {
                let mut game = progress;
                let reply = game.apply(action);
                if matches!(
                    reply,
                    Reply::Unchanged | Reply::NeedKey | Reply::LatchLocked | Reply::FoyerChainSlack
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
                progress.front_door_is_open(),
                matches!(progress, Progress::FrontDoorOpen | Progress::Escaped)
            );
            assert_eq!(
                progress.lever_is_down(Lever::Three),
                progress.front_door_is_open()
            );
            assert_eq!(
                progress.foyer_chain_engaged(),
                matches!(
                    progress,
                    Progress::FoyerPowered
                        | Progress::FoyerStep1
                        | Progress::FoyerStep2
                        | Progress::FoyerStep3
                        | Progress::FrontDoorOpen
                        | Progress::Escaped
                )
            );
            assert_eq!(
                progress.in_foyer(),
                progress == Progress::FoyerEntered || progress.foyer_chain_engaged()
            );
        }
    }
}
