
use fedry_bevy_plugin::debug::ScriptDebugVisible;
use fedry_bevy_plugin::debug::add_script_provider;
use eds_bevy_common::*;
use bevy::prelude::*;

pub struct ScriptDebugPlugin;

impl Plugin for ScriptDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Startup,
                add_script_provider.run_if(resource_exists::<StatsRegistry>)
            )
            .add_systems(
                Update,
                sync_debug_settings.run_if(resource_changed::<GuiState>)
            )
        ;
    }
}

fn sync_debug_settings(
    mut commands: Commands,
    gui_state: Option<Res<GuiState>>,
) {
    commands.insert_resource(ScriptDebugVisible(gui_state.is_some_and(|g| g.enabled)));
}
