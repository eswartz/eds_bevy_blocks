
use fedry_bevy_plugin::debug::ScriptDebugVisible;
use fedry_bevy_plugin::script::ScriptMarker;
use eds_bevy_common::*;
use bevy::prelude::*;
use fedry_bevy_plugin::prelude::ScriptStatsHistory;

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

#[derive(Default)]
pub struct ScriptCountProvider;

impl StatsProvider for ScriptCountProvider {
    fn get_label(&self) -> String {
        "Script Entities".to_string()
    }

    fn format_value(&self, world: &World) -> String {
        if let Some(mut query) = world.try_query::<&ScriptMarker>() {
            format!("{}", query.iter(world).len())
        } else {
            String::new()
        }
    }
}

#[derive(Default)]
pub struct ScriptTimeProvider;

impl StatsProvider for ScriptTimeProvider {
    fn get_label(&self) -> String {
        "Script Time/Frame".to_string()
    }

    fn format_value(&self, world: &World) -> String {
        if let Some(stats) = world.get_resource::<ScriptStatsHistory>() {
            format!("{:.2?}", stats.recent_avg.script_time)
        } else {
            String::new()
        }
    }
}

fn add_script_provider(mut regy: ResMut<eds_bevy_common::StatsRegistry>) {
    regy.add_provider(Box::new(ScriptCountProvider));
    regy.add_provider(Box::new(ScriptTimeProvider));
}

fn sync_debug_settings(
    mut commands: Commands,
    gui_state: Option<Res<GuiState>>,
) {
    commands.insert_resource(ScriptDebugVisible(gui_state.is_some_and(|g| g.enabled)));
}
