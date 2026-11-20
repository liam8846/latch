#![forbid(unsafe_code)]

mod bridge;
mod interaction;
mod physics;
mod player;
mod room;
pub mod session;

#[cfg(any(target_arch = "wasm32", test))]
mod render_failure;

#[cfg(target_arch = "wasm32")]
mod browser;

use bevy::prelude::*;
use session::{Screen, Session};

/// 啟動實際 Bevy PBR 場景；網頁使用既有 canvas，WebGPU 為唯一繪圖後端。
fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.045)))
        .insert_resource(bevy::light::PointLightShadowMap { size: 512 })
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.8, 0.88, 0.85),
            brightness: 75.0,
            affects_lightmapped_meshes: false,
        })
        .init_resource::<Session>()
        .init_resource::<session::UiCommands>()
        .init_resource::<interaction::Focus>()
        .init_resource::<physics::HeldProp>()
        .init_resource::<physics::PhysicsReset>()
        .add_plugins((
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
            avian3d::PhysicsPlugins::default(),
        ))
        .add_systems(Startup, (bridge::setup, room::setup, player::setup))
        .add_systems(
            Update,
            (
                bridge::read_commands,
                interaction::act,
                physics::apply_reset,
                player::move_player,
                physics::animate_drawers,
                physics::carry_held_prop,
                physics::update_pressure_plate,
                interaction::focus_target,
                room::animate,
                room::study_lighting::update_study_lightmaps,
                room::signal::show_signals,
                bridge::publish,
            )
                .chain(),
        );
    #[cfg(target_arch = "wasm32")]
    app.insert_resource(bevy::render::error_handler::RenderErrorHandler(
        render_failure::handle,
    ));
    app.run();
}
