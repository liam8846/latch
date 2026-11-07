use crate::session::SoundCue;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{AudioContext, GainNode, OscillatorNode, OscillatorType};

#[derive(Default)]
pub struct Audio {
    synthesis: Option<Synthesis>,
    pub enabled: bool,
}

struct Synthesis {
    context: AudioContext,
    ambient_gain: GainNode,
    ambient: Vec<OscillatorNode>,
}

impl Audio {
    /// 必須由真實使用者手勢 callback 同步呼叫，以建立與恢復 AudioContext。
    pub fn unlock(&mut self) {
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

    /// 只在遊玩且音效開啟時淡入低音環境音，不會重啟振盪器。
    pub fn ambience(&self, playing: bool) {
        if let Some(synthesis) = &self.synthesis {
            let volume = if self.enabled && playing { 0.008 } else { 0.0 };
            let _ = synthesis.ambient_gain.gain().set_target_at_time(
                volume,
                synthesis.context.current_time(),
                0.3,
            );
        }
    }

    /// 播放一次事件音效；語言切換不會呼叫本方法。
    pub fn cue(&self, cue: SoundCue) {
        let Some(synthesis) = self.synthesis.as_ref().filter(|_| self.enabled) else {
            return;
        };
        let tones: &[(f32, f64, f64)] = match cue {
            SoundCue::Wrong => &[(140.0, 0.2, 0.0), (110.0, 0.25, 0.12)],
            SoundCue::Win => &[
                (261.63, 1.2, 0.0),
                (329.63, 1.2, 0.16),
                (392.0, 1.2, 0.32),
                (523.25, 1.2, 0.48),
            ],
            SoundCue::Unlock | SoundCue::Key => &[(440.0, 0.45, 0.0), (659.25, 0.7, 0.12)],
            SoundCue::Begin => &[(261.63, 0.6, 0.0)],
            SoundCue::Start | SoundCue::Tap => &[(330.0, 0.15, 0.0)],
        };
        for &(frequency, duration, delay) in tones {
            let _ = synthesis.tone(frequency, duration, delay);
        }
    }
}

impl Synthesis {
    /// 以 Hz 與秒建立程序音效圖；瀏覽器拒絕音訊裝置時回報錯誤。
    fn new() -> Result<Self, JsValue> {
        let context = AudioContext::new()?;
        let ambient_gain = context.create_gain()?;
        ambient_gain.gain().set_value(0.0);
        ambient_gain.connect_with_audio_node(&context.destination())?;
        let mut synthesis = Self {
            context,
            ambient_gain,
            ambient: Vec::new(),
        };
        for frequency in [61.735, 92.5] {
            let oscillator = synthesis.context.create_oscillator()?;
            oscillator.frequency().set_value(frequency);
            oscillator.connect_with_audio_node(&synthesis.ambient_gain)?;
            oscillator.start()?;
            synthesis.ambient.push(oscillator);
        }
        Ok(synthesis)
    }

    /// 合成單次正弦音；frequency 使用 Hz，duration/delay 使用秒。
    fn tone(&self, frequency: f32, duration: f64, delay: f64) -> Result<(), JsValue> {
        let oscillator = self.context.create_oscillator()?;
        let gain = self.context.create_gain()?;
        let start = self.context.current_time() + delay;
        oscillator.frequency().set_value(frequency);
        oscillator.set_type(OscillatorType::Sine);
        gain.gain().set_value_at_time(0.0, start)?;
        gain.gain()
            .linear_ramp_to_value_at_time(0.035, start + 0.012)?;
        gain.gain()
            .exponential_ramp_to_value_at_time(0.001, start + duration)?;
        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&self.context.destination())?;
        oscillator.start_with_when(start)?;
        oscillator.stop_with_when(start + duration)
    }
}

impl Drop for Synthesis {
    /// 關閉環境音與音訊裝置；BrowserUi 釋放後不保留運作中的圖。
    fn drop(&mut self) {
        for oscillator in &self.ambient {
            let _ = oscillator.stop();
            let _ = oscillator.disconnect();
        }
        if let Ok(promise) = self.context.close() {
            spawn_local(async move {
                let _ = JsFuture::from(promise).await;
            });
        }
    }
}
