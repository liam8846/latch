#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Progress {
    #[default]
    Searching,
    SafeOpen,
    KeyHeld,
    DoorOpen,
    Escaped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    EnterCode([u8; 4]),
    TakeKey,
    UnlockDoor,
    Leave,
    Restart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reply {
    SafeUnlocked,
    IncorrectCode,
    KeyTaken,
    DoorUnlocked,
    NeedKey,
    Escaped,
    Restarted,
    Unchanged,
}

impl Progress {
    /// 套用單一玩家動作；只有正確順序可抵達出口，重複動作不會重複取得道具。
    pub fn apply(&mut self, action: Action) -> Reply {
        let (next, reply) = match (*self, action) {
            (_, Action::Restart) => (Self::Searching, Reply::Restarted),
            (Self::Searching, Action::EnterCode([1, 9, 4, 7])) => {
                (Self::SafeOpen, Reply::SafeUnlocked)
            }
            (Self::Searching, Action::EnterCode(_)) => (Self::Searching, Reply::IncorrectCode),
            (Self::SafeOpen, Action::TakeKey) => (Self::KeyHeld, Reply::KeyTaken),
            (Self::KeyHeld, Action::UnlockDoor) => (Self::DoorOpen, Reply::DoorUnlocked),
            (Self::Searching | Self::SafeOpen, Action::UnlockDoor) => (*self, Reply::NeedKey),
            (Self::DoorOpen, Action::Leave) => (Self::Escaped, Reply::Escaped),
            _ => (*self, Reply::Unchanged),
        };
        *self = next;
        reply
    }

    /// 表示保險箱門已解鎖，供場景動畫取用。
    pub fn safe_is_open(self) -> bool {
        self != Self::Searching
    }

    /// 表示房門已解鎖，供碰撞及場景動畫取用。
    pub fn door_is_open(self) -> bool {
        matches!(self, Self::DoorOpen | Self::Escaped)
    }
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

    #[test]
    fn player_can_escape_and_restart_without_reading_note() {
        let mut game = Progress::default();
        for (action, reply) in [
            (Action::EnterCode([1, 9, 4, 7]), Reply::SafeUnlocked),
            (Action::TakeKey, Reply::KeyTaken),
            (Action::UnlockDoor, Reply::DoorUnlocked),
            (Action::Leave, Reply::Escaped),
            (Action::Restart, Reply::Restarted),
        ] {
            assert_eq!(game.apply(action), reply);
        }
        assert_eq!(game, Progress::Searching);
    }

    #[test]
    fn wrong_codes_do_not_advance_progress() {
        for code in [[0, 0, 0, 0], [1, 9, 4, 8], [9, 9, 9, 9]] {
            let mut game = Progress::Searching;
            assert_eq!(game.apply(Action::EnterCode(code)), Reply::IncorrectCode);
            assert_eq!(game, Progress::Searching);
        }
    }

    #[test]
    fn every_wrong_order_and_repeated_action_preserves_progress() {
        for progress in [
            Progress::Searching,
            Progress::SafeOpen,
            Progress::KeyHeld,
            Progress::DoorOpen,
            Progress::Escaped,
        ] {
            for action in [
                Action::EnterCode([1, 9, 4, 7]),
                Action::EnterCode([0; 4]),
                Action::TakeKey,
                Action::UnlockDoor,
                Action::Leave,
            ] {
                let mut game = progress;
                let reply = game.apply(action);
                match (progress, action) {
                    (Progress::Searching, Action::EnterCode(_))
                    | (Progress::SafeOpen, Action::TakeKey)
                    | (Progress::KeyHeld, Action::UnlockDoor)
                    | (Progress::DoorOpen, Action::Leave) => {}
                    _ => assert_eq!(game, progress),
                }
                if reply == Reply::Unchanged || reply == Reply::NeedKey {
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
    fn visual_state_matches_each_stage() {
        for progress in [
            Progress::Searching,
            Progress::SafeOpen,
            Progress::KeyHeld,
            Progress::DoorOpen,
            Progress::Escaped,
        ] {
            assert_eq!(progress.safe_is_open(), progress != Progress::Searching);
            assert_eq!(
                progress.door_is_open(),
                matches!(progress, Progress::DoorOpen | Progress::Escaped)
            );
        }
    }
}
