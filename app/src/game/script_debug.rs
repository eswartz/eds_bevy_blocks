use bevy::prelude::*;
use eds_bevy_common::*;
use fedry_bevy_plugin::debug_egui::ScriptDebugVisible;
use fedry_bevy_plugin::prelude::ScriptControl;

#[cfg(feature = "ebc")]
use fedry_bevy_plugin::debug_ebc::add_script_provider;

pub struct ScriptDebugPlugin;

impl Plugin for ScriptDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                init_reset_on_edit,
                add_script_provider.run_if(resource_exists::<StatsRegistry>),
            ),
        )
        .add_systems(
            Update,
            sync_debug_settings.run_if(resource_changed::<GuiState>),
        );
    }
}

fn init_reset_on_edit(mut control: ResMut<ScriptControl>) {
    if dev_tools_enabled() {
        control.set_reset_on_edit(true);
    }
}

fn sync_debug_settings(mut commands: Commands, gui_state: Option<Res<GuiState>>) {
    commands.insert_resource(ScriptDebugVisible(gui_state.is_some_and(|g| g.enabled)));
}
