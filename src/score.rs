pub const BEAT_SECONDS: f64 = 60.0 / 66.0;
pub const PHRASE_BEATS: f64 = 36.0;
pub const ATTACK_SECONDS: f64 = 0.014;
pub const RELEASE_SECONDS: f64 = 1.7;
pub const SECOND_HARMONIC: f32 = 0.16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub beat: f64,
    pub midi: u8,
    pub length: f64,
    pub velocity: f32,
}

impl Note {
    /// 將 MIDI 音高換成 Hz；A4 = MIDI 69 = 440 Hz。
    pub fn frequency(self) -> f32 {
        440.0 * 2.0_f32.powf((f32::from(self.midi) - 69.0) / 12.0)
    }
}

const MELODY: [[u8; 3]; 12] = [
    [69, 76, 74],
    [72, 69, 0],
    [65, 69, 72],
    [74, 72, 0],
    [69, 72, 77],
    [76, 72, 0],
    [67, 74, 72],
    [69, 67, 0],
    [69, 76, 74],
    [72, 69, 65],
    [67, 72, 74],
    [76, 74, 0],
];
const HARMONY: [[u8; 3]; 4] = [[50, 57, 64], [46, 53, 60], [48, 57, 65], [48, 55, 62]];

/// 原創十二小節 3/4 柔鍵盤主題，beat 與 length 使用四分音符拍數。
/// 鑰匙取得後採 F 大調末句，原本懸停的 D/E 音改為 C/F 的舒展收束。
pub fn theme(resolved: bool) -> Vec<Note> {
    let mut notes = Vec::new();
    for (bar, melody) in MELODY.iter().enumerate() {
        let chord = HARMONY[(bar / 2) % 4];
        for (voice, midi) in chord.into_iter().enumerate() {
            notes.push(Note {
                beat: bar as f64 * 3.0 + voice as f64 * 0.5,
                midi,
                length: 2.3 - voice as f64 * 0.3,
                velocity: [0.27, 0.18, 0.16][voice],
            });
        }
        for (beat, &midi) in melody.iter().enumerate() {
            if midi == 0 {
                continue;
            }
            let midi = if resolved && bar >= 10 {
                [[69, 72, 76], [77, 72, 69]][bar - 10][beat]
            } else {
                midi
            };
            notes.push(Note {
                beat: bar as f64 * 3.0 + beat as f64,
                midi,
                length: if beat == 2 { 1.5 } else { 0.8 },
                velocity: if beat == 0 { 0.56 } else { 0.43 },
            });
        }
    }
    notes.sort_by(|a, b| a.beat.total_cmp(&b.beat));
    notes
}

/// 只在首次收好信並取得鑰匙時使用的兩小節答句，與主題共用音色。
pub fn coda() -> [Note; 6] {
    [
        Note {
            beat: 0.0,
            midi: 53,
            length: 5.0,
            velocity: 0.32,
        },
        Note {
            beat: 0.0,
            midi: 69,
            length: 1.4,
            velocity: 0.55,
        },
        Note {
            beat: 1.0,
            midi: 72,
            length: 1.4,
            velocity: 0.48,
        },
        Note {
            beat: 2.0,
            midi: 76,
            length: 1.4,
            velocity: 0.43,
        },
        Note {
            beat: 3.0,
            midi: 77,
            length: 3.0,
            velocity: 0.52,
        },
        Note {
            beat: 3.0,
            midi: 60,
            length: 3.0,
            velocity: 0.27,
        },
    ]
}

#[derive(Default)]
pub struct ScoreClock {
    origin: Option<f64>,
    scheduled_until: f64,
}

impl ScoreClock {
    /// 回傳本次可排程的絕對秒數與音符。延遲幀跳過已過音符，不補播。
    /// 時鐘以 AudioContext 的單調秒數為準；暫停不重新開始樂句。
    pub fn window(&mut self, now: f64, horizon: f64, notes: &[Note]) -> Vec<(f64, Note)> {
        if !now.is_finite() || !horizon.is_finite() || horizon <= 0.0 {
            return Vec::new();
        }
        let origin = *self.origin.get_or_insert(now);
        let start = self.scheduled_until.max(now);
        let end = now + horizon.min(0.25);
        if start >= end {
            return Vec::new();
        }
        let period = PHRASE_BEATS * BEAT_SECONDS;
        let first = ((start - origin) / period).floor().max(0.0) as u64;
        let last = ((end - origin) / period).floor().max(0.0) as u64;
        let mut events = Vec::new();
        for cycle in first..=last {
            for &note in notes {
                let time = origin + cycle as f64 * period + note.beat * BEAT_SECONDS;
                if time >= start && time < end {
                    events.push((time, note));
                }
            }
        }
        self.scheduled_until = end;
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn theme_has_twelve_bars_with_playable_notes_and_a_different_ending() {
        let theme = theme(false);
        assert!(theme.iter().all(|n| n.beat >= 0.0
            && n.beat < 36.0
            && n.length > 0.0
            && n.velocity > 0.0
            && n.velocity <= 1.0
            && (21..=108).contains(&n.midi)));
        for bar in 0..12 {
            assert!(theme.iter().any(|n| n.beat == f64::from(bar * 3)));
        }
        assert_eq!(
            theme.iter().filter(|n| n.beat < 30.0).collect::<Vec<_>>(),
            super::theme(true)
                .iter()
                .filter(|n| n.beat < 30.0)
                .collect::<Vec<_>>()
        );
        assert_ne!(theme, super::theme(true));
        assert!(
            (Note {
                beat: 0.0,
                midi: 69,
                length: 1.0,
                velocity: 1.0
            }
            .frequency()
                - 440.0)
                .abs()
                < 0.01
        );
    }
    #[test]
    fn scheduler_never_replays_or_catches_up_and_wraps_the_phrase() {
        let notes = theme(false);
        let mut clock = ScoreClock::default();
        assert!(!clock.window(10.0, 0.2, &notes).is_empty());
        assert!(clock.window(10.0, 0.2, &notes).is_empty());
        let after_lag = clock.window(18.0, 0.2, &notes);
        assert!(
            after_lag
                .iter()
                .all(|(time, _)| *time >= 18.0 && *time < 18.2)
        );
        let wrap = 10.0 + PHRASE_BEATS * BEAT_SECONDS;
        let next = clock.window(wrap, 0.2, &notes);
        assert!(next.iter().any(|(time, _)| (*time - wrap).abs() < 0.001));
        assert!(clock.window(f64::NAN, 0.2, &notes).is_empty());
        assert!(clock.window(50.0, 0.0, &notes).is_empty());
    }
}
