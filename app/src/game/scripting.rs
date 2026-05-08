use crate::game::bevy_funcs::*;

use bevy::prelude::*;

use fedry_bevy_plugin::bevy_system_service::BevySystemService;
use fedry_bevy_plugin::prelude::*;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Startup,
                register_funcs,
            )
        ;
    }
}

fn register_funcs(mut commands: Commands, runtime: Res<ScriptRuntime>) {
    let rt = &runtime.rt;
    let bevy_system_service = rt.get_service::<BevySystemService>().expect("expected BevySystemService");

    bevy_system_service.add_runtime_system(
        rt.pool.for_str("spawn_cube"),
        commands.register_system(spawn_cube));

    bevy_system_service.add_runtime_system(
        rt.pool.for_str("translate"),
        commands.register_system(translate));
    bevy_system_service.add_runtime_system(
        rt.pool.for_str("add_velocity"),
        commands.register_system(add_velocity));
    bevy_system_service.add_runtime_system(
        rt.pool.for_str("set_gravity"),
        commands.register_system(set_gravity));

    bevy_system_service.add_runtime_system(
        rt.pool.for_str("add_script"),
        commands.register_system(add_script));
    bevy_system_service.add_runtime_system(
        rt.pool.for_str("remove_script"),
        commands.register_system(remove_script));

}
