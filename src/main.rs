#![forbid(unsafe_code)]

mod bridge;
mod interaction;
mod player;
mod room;
pub mod session;

#[cfg(target_arch = "wasm32")]
mod browser;

use bevy::prelude::*;
use session::{Screen, Session};

/// 啟動實際 Bevy PBR 場景；網頁使用既有 canvas，WebGPU 為唯一繪圖後端。
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.045)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.8, 0.88, 0.85),
            brightness: 75.0,
            ..default()
        })
        .init_resource::<Session>()
        .init_resource::<session::UiCommands>()
        .init_resource::<interaction::Focus>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Latch · 午夜書房".into(),
                        canvas: Some("#game".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, (bridge::setup, room::setup, player::setup))
        .add_systems(
            Update,
            (
                bridge::read_commands,
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
