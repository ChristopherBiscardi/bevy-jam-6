use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*, ui::UiDebugOptions,
};

pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(EguiPlugin {
            //     enable_multipass_for_primary_context: true,
            // })
            // .add_plugins(
            //     WorldInspectorPlugin::default().run_if(
            //         input_toggle_active(true, KeyCode::Escape),
            //     ),
            // )
            .add_systems(
                Update,
                toggle_debug_ui.run_if(input_just_pressed(
                    KeyCode::Backquote,
                )),
            );
    }
}

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
