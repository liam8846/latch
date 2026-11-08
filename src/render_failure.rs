use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    render::error_handler::{RenderError, RenderErrorHandler, RenderErrorPolicy},
};

/// 在預設結束政策前同步保留 renderer 原始錯誤，清除輸入且不等待下一次 Update。
pub fn handle(
    error: &RenderError,
    main_world: &mut World,
    render_world: &mut World,
) -> RenderErrorPolicy {
    #[cfg(target_arch = "wasm32")]
    {
        let details = format!("{error:#?}");
        if let Some(mut ui) = main_world.get_non_send_mut::<crate::browser::BrowserUi>() {
            ui.fail_renderer(&details);
        } else {
            crate::browser::renderer_fallback(&details);
        }
    }
    if let Some(mut session) = main_world.get_resource_mut::<crate::Session>() {
        session.screen = crate::Screen::Paused;
    }
    if let Some(mut commands) = main_world.get_resource_mut::<crate::session::UiCommands>() {
        commands.0.clear();
    }
    if let Some(mut keys) = main_world.get_resource_mut::<ButtonInput<KeyCode>>() {
        keys.reset_all();
    }
    if let Some(mut buttons) = main_world.get_resource_mut::<ButtonInput<MouseButton>>() {
        buttons.reset_all();
    }
    if let Some(mut mouse) = main_world.get_resource_mut::<AccumulatedMouseMotion>() {
        mouse.delta = Vec2::ZERO;
    }
    (RenderErrorHandler::default().0)(error, main_world, render_world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{UiCommand, UiCommands};
    use bevy::render::error_handler::ErrorType;

    #[test]
    fn fatal_renderer_error_releases_input_and_preserves_default_exit_policy() {
        let mut world = World::new();
        world.init_resource::<Messages<AppExit>>();
        world.insert_resource(crate::Session {
            screen: crate::Screen::Playing,
            ..default()
        });
        world.insert_resource(UiCommands([UiCommand::Begin].into()));
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        world.insert_resource(keys);
        let mut buttons = ButtonInput::default();
        buttons.press(MouseButton::Left);
        world.insert_resource(buttons);
        world.insert_resource(AccumulatedMouseMotion { delta: Vec2::ONE });
        let error = RenderError {
            ty: ErrorType::Validation,
            description: "fixture validation failure".to_owned(),
            source: None,
        };

        let policy = handle(&error, &mut world, &mut World::new());

        assert!(matches!(policy, RenderErrorPolicy::StopRendering));
        assert_eq!(
            world.resource::<crate::Session>().screen,
            crate::Screen::Paused
        );
        assert!(world.resource::<UiCommands>().0.is_empty());
        assert!(
            !world
                .resource::<ButtonInput<KeyCode>>()
                .pressed(KeyCode::KeyW)
        );
        assert!(
            !world
                .resource::<ButtonInput<MouseButton>>()
                .pressed(MouseButton::Left)
        );
        assert_eq!(world.resource::<AccumulatedMouseMotion>().delta, Vec2::ZERO);
        assert!(
            world
                .resource_mut::<Messages<AppExit>>()
                .drain()
                .any(|exit| exit.is_error())
        );
    }
}
