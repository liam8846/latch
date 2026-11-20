use crate::signal::SignalBoard;
use std::f32::consts::TAU;

pub const REQUIRED_PLATE_MASS_KG: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lever {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockHand {
    Hour,
    Minute,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockFace {
    pub hour: u8,
    pub minute_five: u8,
}

impl ClockFace {
    /// 順時針撥動時針（+1 時）或分針（+5 分），各自以十二格循環。
    pub fn advance(&mut self, hand: ClockHand) {
        match hand {
            ClockHand::Hour => self.hour = (self.hour + 1) % 12,
            ClockHand::Minute => self.minute_five = (self.minute_five + 1) % 12,
        }
    }

    /// 判斷鐘面是否對準回程末班時刻 22:35（10 點 35 分）。
    ///
    /// ```
    /// let clock = latch::puzzle::ClockFace { hour: 10, minute_five: 7 };
    /// assert!(clock.is_target());
    /// assert!(!latch::puzzle::ClockFace::default().is_target());
    /// ```
    pub fn is_target(self) -> bool {
        self.hour == 10 && self.minute_five == 7
    }

    /// 回傳時針繞 +X 軸順時針旋轉的弧度（0 弧度指向 +Y 十二點鐘，包含分針微偏）。
    pub fn hour_angle_rad(self) -> f32 {
        (f32::from(self.hour) + f32::from(self.minute_five) / 12.0) * (TAU / 12.0)
    }

    /// 回傳分針繞 +X 軸順時針旋轉的弧度（0 弧度指向 +Y 十二點鐘）。
    pub fn minute_angle_rad(self) -> f32 {
        f32::from(self.minute_five) * (TAU / 12.0)
    }
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
    FoyerClockSynced,
    FoyerStep1,
    FoyerStep2,
    FoyerStep3,
    FrontDoorOpen,
    SignalRoom,
    SignalGateOpen,
    Escaped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    ReleaseLatch,
    EnterCode([u8; 4]),
    TakeKey,
    UnlockDoor,
    EnterFoyer,
    SyncClock,
    PullLever(Lever),
    EnterSignalRoom,
    AlignSignals(SignalBoard),
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
    ClockSynced,
    ClockNotSynced,
    LeverEngaged,
    LeversReset,
    FrontDoorUnlocked,
    SignalRoomEntered,
    SignalGateUnlocked,
    Escaped,
    Restarted,
    Unchanged,
}

impl Progress {
    /// 依序套用書房、玄關與潮汐信號室的進度；只有兩束光同時對準才能打開最後的閘門。
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
            (Self::DoorOpen | Self::FoyerEntered, Action::SyncClock) => {
                (Self::FoyerClockSynced, Reply::ClockSynced)
            }
            (Self::DoorOpen | Self::FoyerEntered, Action::PullLever(_)) => {
                (*self, Reply::ClockNotSynced)
            }
            (Self::FoyerClockSynced, Action::PullLever(Lever::Four)) => {
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
                Self::FoyerClockSynced | Self::FoyerStep1 | Self::FoyerStep2 | Self::FoyerStep3,
                Action::PullLever(_),
            ) => (Self::FoyerClockSynced, Reply::LeversReset),
            (Self::FrontDoorOpen, Action::EnterSignalRoom) => {
                (Self::SignalRoom, Reply::SignalRoomEntered)
            }
            (Self::SignalRoom, Action::AlignSignals(board)) if board.is_aligned() => {
                (Self::SignalGateOpen, Reply::SignalGateUnlocked)
            }
            (Self::SignalGateOpen, Action::Leave) => (Self::Escaped, Reply::Escaped),
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

    /// 表示玄關掛鐘已校準至 22:35，鐘擺開始擺動且四道拉桿已解除保險。
    pub fn foyer_clock_synced(self) -> bool {
        matches!(
            self,
            Self::FoyerClockSynced | Self::FoyerStep1 | Self::FoyerStep2 | Self::FoyerStep3
        ) || self.front_door_is_open()
    }

    /// 表示指定玄關黃銅拉桿目前處於扣下位置，完全由唯一進度導出。
    pub fn lever_is_down(self, lever: Lever) -> bool {
        match lever {
            Lever::Four => {
                matches!(self, Self::FoyerStep1 | Self::FoyerStep2 | Self::FoyerStep3)
                    || self.front_door_is_open()
            }
            Lever::Two => {
                matches!(self, Self::FoyerStep2 | Self::FoyerStep3) || self.front_door_is_open()
            }
            Lever::One => self == Self::FoyerStep3 || self.front_door_is_open(),
            Lever::Three => self.front_door_is_open(),
        }
    }

    /// 表示玄關臨街大門已解鎖，供門板碰撞與離場判定取用。
    pub fn front_door_is_open(self) -> bool {
        matches!(
            self,
            Self::FrontDoorOpen | Self::SignalRoom | Self::SignalGateOpen | Self::Escaped
        )
    }

    /// 表示潮汐信號室的出口閘門已解鎖。
    pub fn signal_gate_is_open(self) -> bool {
        matches!(self, Self::SignalGateOpen | Self::Escaped)
    }

    /// 表示玩家已從書房無縫走進玄關區域。
    pub fn in_foyer(self) -> bool {
        self == Self::FoyerEntered || self.foyer_clock_synced()
    }
}

/// 判斷書房天平秤盤上的物件總質量（公斤）是否達標；拒絕 NaN 與無窮值。
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

    const ALL_STAGES: [Progress; 14] = [
        Progress::Searching,
        Progress::LatchReleased,
        Progress::SafeOpen,
        Progress::KeyHeld,
        Progress::DoorOpen,
        Progress::FoyerEntered,
        Progress::FoyerClockSynced,
        Progress::FoyerStep1,
        Progress::FoyerStep2,
        Progress::FoyerStep3,
        Progress::FrontDoorOpen,
        Progress::SignalRoom,
        Progress::SignalGateOpen,
        Progress::Escaped,
    ];

    #[test]
    fn clock_face_wraps_at_twelve_and_matches_target_time() {
        let mut clock = ClockFace::default();
        assert!(!clock.is_target());
        assert!((clock.hour_angle_rad() - 0.0).abs() < 1e-5);
        assert!((clock.minute_angle_rad() - 0.0).abs() < 1e-5);
        for _ in 0..10 {
            clock.advance(ClockHand::Hour);
        }
        assert_eq!(clock.hour, 10);
        assert!(!clock.is_target());
        for _ in 0..7 {
            clock.advance(ClockHand::Minute);
        }
        assert_eq!(clock.minute_five, 7);
        assert!(clock.is_target());
        assert!((clock.minute_angle_rad() - 7.0 * TAU / 12.0).abs() < 1e-5);
        assert!((clock.hour_angle_rad() - (10.0 + 7.0 / 12.0) * TAU / 12.0).abs() < 1e-5);
        for _ in 0..2 {
            clock.advance(ClockHand::Hour);
        }
        assert_eq!(clock.hour, 0);
        for _ in 0..5 {
            clock.advance(ClockHand::Minute);
        }
        assert_eq!(clock.minute_five, 0);
    }

    #[test]
    fn player_completes_all_three_rooms_then_restarts() {
        use crate::signal::Mirror;
        let mut aligned = SignalBoard::default();
        for (mirror, turns) in Mirror::ALL.into_iter().zip([2, 2, 0, 1, 2, 1, 0]) {
            for _ in 0..turns {
                aligned.turn(mirror);
            }
        }
        let mut game = Progress::default();
        for (action, reply) in [
            (Action::ReleaseLatch, Reply::LatchReleased),
            (Action::EnterCode([1, 9, 4, 7]), Reply::SafeUnlocked),
            (Action::TakeKey, Reply::KeyTaken),
            (Action::UnlockDoor, Reply::DoorUnlocked),
            (Action::EnterFoyer, Reply::FoyerEntered),
            (Action::PullLever(Lever::Four), Reply::ClockNotSynced),
            (Action::SyncClock, Reply::ClockSynced),
            (Action::PullLever(Lever::Four), Reply::LeverEngaged),
            (Action::PullLever(Lever::Two), Reply::LeverEngaged),
            (Action::PullLever(Lever::One), Reply::LeverEngaged),
            (Action::PullLever(Lever::Three), Reply::FrontDoorUnlocked),
            (Action::Leave, Reply::Unchanged),
            (Action::AlignSignals(aligned), Reply::Unchanged),
            (Action::EnterSignalRoom, Reply::SignalRoomEntered),
            (Action::Leave, Reply::Unchanged),
            (
                Action::AlignSignals(SignalBoard::default()),
                Reply::Unchanged,
            ),
            (Action::AlignSignals(aligned), Reply::SignalGateUnlocked),
            (Action::Leave, Reply::Escaped),
            (Action::Restart, Reply::Restarted),
        ] {
            assert_eq!(game.apply(action), reply);
        }
        assert_eq!(game, Progress::Searching);
    }

    #[test]
    fn wrong_lever_order_resets_all_four_levers_to_clock_synced() {
        for (start, wrong) in [
            (Progress::FoyerClockSynced, Lever::One),
            (Progress::FoyerClockSynced, Lever::Two),
            (Progress::FoyerClockSynced, Lever::Three),
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
            assert_eq!(game, Progress::FoyerClockSynced);
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
                Action::SyncClock,
                Action::PullLever(Lever::One),
                Action::PullLever(Lever::Two),
                Action::PullLever(Lever::Three),
                Action::PullLever(Lever::Four),
                Action::EnterSignalRoom,
                Action::AlignSignals(SignalBoard::default()),
                Action::Leave,
            ] {
                let mut game = progress;
                let reply = game.apply(action);
                if matches!(
                    reply,
                    Reply::Unchanged | Reply::NeedKey | Reply::LatchLocked | Reply::ClockNotSynced
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
                matches!(
                    progress,
                    Progress::FrontDoorOpen
                        | Progress::SignalRoom
                        | Progress::SignalGateOpen
                        | Progress::Escaped
                )
            );
            assert_eq!(
                progress.lever_is_down(Lever::Three),
                progress.front_door_is_open()
            );
            assert_eq!(
                progress.foyer_clock_synced(),
                matches!(
                    progress,
                    Progress::FoyerClockSynced
                        | Progress::FoyerStep1
                        | Progress::FoyerStep2
                        | Progress::FoyerStep3
                        | Progress::FrontDoorOpen
                        | Progress::SignalRoom
                        | Progress::SignalGateOpen
                        | Progress::Escaped
                )
            );
            assert_eq!(
                progress.in_foyer(),
                progress == Progress::FoyerEntered || progress.foyer_clock_synced()
            );
        }
    }
}
