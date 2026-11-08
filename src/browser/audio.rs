use crate::session::{Screen, Session, SoundCue};
use latch::{
    puzzle::Progress,
    score::{self, Note, ScoreClock},
};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{AudioContext, AudioContextState, GainNode, OscillatorNode, OscillatorType};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum MusicScene {
    #[default]
    Silent,
    Study,
    Reading,
    Departure,
}

impl MusicScene {
    /// 音樂情境只依目前畫面與謎題進度導出，不另建遊戲狀態。
    fn from_session(session: &Session) -> Self {
        match session.screen {
            Screen::Intro | Screen::Paused => Self::Silent,
            Screen::Inspect(_) | Screen::Safe => Self::Reading,
            Screen::Won => Self::Departure,
            Screen::Playing
                if session.progress != Progress::Searching
                    && session.progress != Progress::SafeOpen =>
            {
                Self::Departure
            }
            Screen::Playing => Self::Study,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SoundPreference {
    #[default]
    Automatic,
    Enabled,
    Muted,
}

#[derive(Default)]
pub struct Audio {
    synthesis: Option<Synthesis>,
    preference: SoundPreference,
    interrupted: bool,
    resolved: bool,
}

struct Voice {
    oscillators: [OscillatorNode; 2],
    gain: GainNode,
    overtone: GainNode,
    end: f64,
}

struct Synthesis {
    context: AudioContext,
    master: GainNode,
    music: GainNode,
    voices: Vec<Voice>,
    clock: ScoreClock,
    searching: Vec<Note>,
    departing: Vec<Note>,
    master_level: f32,
    music_level: f32,
    coda: Option<(f64, usize)>,
}

impl Audio {
    /// 在真實手勢 callback 同步建立或恢復裝置；失敗時保留無聲遊戲。
    pub fn unlock(&mut self) {
        self.interrupted = false;
        if self.synthesis.is_none() {
            self.synthesis = Synthesis::new().ok();
        }
        if let Some(synthesis) = &self.synthesis
            && let Ok(promise) = synthesis.context.resume()
        {
            spawn_local(async move {
                let _ = JsFuture::from(promise).await;
            });
        }
    }

    /// 介面只有在裝置確實運行且使用者允許音訊時顯示開啟。
    pub fn enabled(&self) -> bool {
        self.preference == SoundPreference::Enabled
            && !self.interrupted
            && self
                .synthesis
                .as_ref()
                .is_some_and(|s| s.context.state() == AudioContextState::Running)
    }

    /// 首次開始預設啟用音訊，保留玩家在標題畫面的明確選擇。
    pub fn begin(&mut self) {
        if self.preference == SoundPreference::Automatic {
            self.preference = SoundPreference::Enabled;
        }
    }

    /// 關閉時同步截斷所有已排程聲音，包含事件尾音。
    pub fn set_enabled(&mut self, enabled: bool) {
        self.preference = if enabled {
            SoundPreference::Enabled
        } else {
            SoundPreference::Muted
        };
        if !enabled {
            self.silence();
        }
    }

    /// 失焦、隱藏、暫停與錯誤 callbacks 直接靜音，不等待下一幀。
    pub fn silence(&mut self) {
        self.interrupted = true;
        if let Some(synthesis) = &mut self.synthesis {
            synthesis.master_level(0.0);
            synthesis.voices.clear();
            synthesis.coda = None;
        }
    }

    /// 重新開始才重設樂句與一次性收束；恢復及語言切換保留游標。
    pub fn restart(&mut self) {
        self.resolved = false;
        if let Some(synthesis) = &mut self.synthesis {
            synthesis.voices.clear();
            synthesis.clock = ScoreClock::default();
            synthesis.coda = None;
        }
    }

    /// 以短 lookahead 排程音符；閱讀降低音量但保留旋律位置。
    pub fn update(&mut self, session: &Session) {
        let scene = MusicScene::from_session(session);
        let resolved = matches!(
            session.progress,
            Progress::KeyHeld | Progress::DoorOpen | Progress::Escaped
        );
        let Some(synthesis) = &mut self.synthesis else {
            return;
        };
        let now = synthesis.context.current_time();
        synthesis.voices.retain(|voice| voice.end > now);
        let audible = self.preference == SoundPreference::Enabled
            && !self.interrupted
            && scene != MusicScene::Silent;
        synthesis.master_level(if audible { 0.55 } else { 0.0 });
        let music_level = if scene == MusicScene::Reading {
            0.38
        } else {
            0.72
        };
        synthesis.music_level(music_level);
        if resolved && !self.resolved {
            self.resolved = true;
            if audible {
                synthesis.coda = Some((now, 0));
            }
        }
        if !audible {
            return;
        }
        synthesis.schedule_coda(now);
        let notes = if self.resolved {
            &synthesis.departing
        } else {
            &synthesis.searching
        };
        let due = synthesis.clock.window(now, 0.18, notes);
        if audible {
            for (time, note) in due {
                let _ = synthesis.note(note, time, true);
            }
        }
    }

    /// 單次互動提示也通過 master，音量切換立即作用於尾音。
    pub fn cue(&mut self, cue: SoundCue) {
        if self.preference != SoundPreference::Enabled {
            return;
        }
        let Some(synthesis) = &mut self.synthesis else {
            return;
        };
        let tones: &[(u8, f64)] = match cue {
            SoundCue::Wrong => &[(50, 0.0), (49, 0.15)],
            SoundCue::Win => &[(65, 0.0), (69, 0.2), (72, 0.4)],
            SoundCue::Unlock => &[(69, 0.0), (76, 0.12)],
            SoundCue::Key => &[],
            SoundCue::Begin | SoundCue::Start => &[(62, 0.0)],
            SoundCue::Tap => &[(64, 0.0)],
        };
        let now = synthesis.context.current_time();
        for &(midi, delay) in tones {
            let _ = synthesis.note(
                Note {
                    beat: 0.0,
                    midi,
                    length: 0.22,
                    velocity: 0.3,
                },
                now + delay,
                false,
            );
        }
    }
}

impl Synthesis {
    /// 建立單一 master 音訊圖；所有聲音共享可同步切斷的輸出。
    fn new() -> Result<Self, JsValue> {
        let context = AudioContext::new()?;
        let master = context.create_gain()?;
        master.gain().set_value(0.0);
        master.connect_with_audio_node(&context.destination())?;
        let music = context.create_gain()?;
        music.gain().set_value(0.72);
        music.connect_with_audio_node(&master)?;
        Ok(Self {
            context,
            master,
            music,
            voices: Vec::new(),
            clock: ScoreClock::default(),
            searching: score::theme(false),
            departing: score::theme(true),
            master_level: 0.0,
            music_level: 0.72,
            coda: None,
        })
    }

    /// 答句也僅預排 180 毫秒，延遲或暫停後不補播舊音符。
    fn schedule_coda(&mut self, now: f64) {
        let Some((origin, mut index)) = self.coda else {
            return;
        };
        let notes = score::coda();
        while let Some(&note) = notes.get(index) {
            let time = origin + note.beat * score::BEAT_SECONDS;
            if time >= now + 0.18 {
                break;
            }
            if time >= now {
                let _ = self.note(note, time, true);
            }
            index += 1;
        }
        self.coda = (index < notes.len()).then_some((origin, index));
    }

    /// 音量只在值改變時排程，硬靜音取消先前 automation。
    fn master_level(&mut self, level: f32) {
        if self.master_level == level {
            return;
        }
        self.master_level = level;
        let now = self.context.current_time();
        let _ = self.master.gain().cancel_scheduled_values(now);
        let _ = self.master.gain().set_value_at_time(level, now);
    }

    /// 閱讀時用 150 毫秒緩降音樂，不重啟樂句。
    fn music_level(&mut self, level: f32) {
        if self.music_level == level {
            return;
        }
        self.music_level = level;
        let _ = self
            .music
            .gain()
            .set_target_at_time(level, self.context.current_time(), 0.15);
    }

    /// 以兩條正弦諧波與衰減包絡合成柔鍵盤；時間是 AudioContext 絕對秒。
    fn note(&mut self, note: Note, start: f64, music: bool) -> Result<(), JsValue> {
        let fundamental = self.context.create_oscillator()?;
        let harmonic = self.context.create_oscillator()?;
        let gain = self.context.create_gain()?;
        let overtone = self.context.create_gain()?;
        let end = start + note.length * score::BEAT_SECONDS + score::RELEASE_SECONDS;
        fundamental.set_type(OscillatorType::Sine);
        harmonic.set_type(OscillatorType::Sine);
        fundamental.frequency().set_value(note.frequency());
        harmonic.frequency().set_value(note.frequency() * 2.0);
        overtone.gain().set_value(score::SECOND_HARMONIC);
        fundamental.connect_with_audio_node(&gain)?;
        harmonic.connect_with_audio_node(&overtone)?;
        overtone.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(if music { &self.music } else { &self.master })?;
        gain.gain().set_value_at_time(0.0, start)?;
        gain.gain()
            .linear_ramp_to_value_at_time(note.velocity * 0.13, start + score::ATTACK_SECONDS)?;
        gain.gain().exponential_ramp_to_value_at_time(0.0001, end)?;
        fundamental.start_with_when(start)?;
        harmonic.start_with_when(start)?;
        fundamental.stop_with_when(end)?;
        harmonic.stop_with_when(end)?;
        self.voices.push(Voice {
            oscillators: [fundamental, harmonic],
            gain,
            overtone,
            end,
        });
        Ok(())
    }
}

impl Drop for Voice {
    /// 回收已結束、重啟或裝置釋放的音符節點，不保留背景圖。
    fn drop(&mut self) {
        for oscillator in &self.oscillators {
            let _ = oscillator.stop();
            let _ = oscillator.disconnect();
        }
        let _ = self.gain.disconnect();
        let _ = self.overtone.disconnect();
    }
}

impl Drop for Synthesis {
    /// 關閉裝置並釋放所有聲部。
    fn drop(&mut self) {
        self.voices.clear();
        let _ = self.master.disconnect();
        let _ = self.music.disconnect();
        if let Ok(promise) = self.context.close() {
            spawn_local(async move {
                let _ = JsFuture::from(promise).await;
            });
        }
    }
}
