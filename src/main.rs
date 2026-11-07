#![forbid(unsafe_code)]

mod bridge;
mod interaction;
mod player;
mod room;

use bevy::prelude::*;
use latch::puzzle::Progress;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Screen {
    #[default]
    Intro,
    Playing,
    Paused,
    Note,
    Safe,
    Won,
}

#[derive(Resource, Default)]
struct Session {
    progress: Progress,
    screen: Screen,
    message: &'static str,
    sound: &'static str,
    revision: u32,
}

impl Session {
    /// 更新提示與音效事件序號，重複提示也會觸發一次新的音效。
    fn notify(&mut self, message: &'static str, sound: &'static str) {
        self.message = message;
        self.sound = sound;
        self.revision = self.revision.wrapping_add(1);
    }
}

/// 啟動實際 Bevy PBR 場景；網頁使用既有 canvas，WebGPU 為唯一繪圖後端。
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.045)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.8, 0.88, 0.85),
            brightness: 180.0,
            ..default()
        })
        .init_resource::<Session>()
        .init_resource::<bridge::Command>()
        .init_resource::<interaction::Focus>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Latch · 午夜書房".into(),
                canvas: Some("#game".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (room::setup, player::setup))
        .add_systems(
            Update,
            (
                bridge::read_command,
                interaction::act,
                player::move_player,
                interaction::focus_target,
                room::animate,
                bridge::publish,
            )
                .chain(),
        )
        .run();
}
