use crate::game::bevy_funcs::add_script;
use crate::game::bevy_funcs::add_velocity;
use crate::game::bevy_funcs::remove_script;
use crate::game::bevy_funcs::send_midi_message;
use crate::game::bevy_funcs::set_gravity;
use crate::game::bevy_funcs::spawn_cube;
use crate::game::bevy_funcs::translate;

use bevy::prelude::*;

use fedry_bevy_plugin::bevy_world_function_service::BevyWorldFunctionService;
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
    let bevy_world_service = rt.get_service::<BevyWorldFunctionService>().expect("expected BevyWorldService");
    bevy_world_service.add_world_func(
        rt.pool.for_str("spawn_cube"),
        commands.register_system(spawn_cube));
    bevy_world_service.add_world_func(
        rt.pool.for_str("send_midi_message"),
        commands.register_system(send_midi_message));
    bevy_world_service.add_world_func(
        rt.pool.for_str("add_script"),
        commands.register_system(add_script));
    bevy_world_service.add_world_func(
        rt.pool.for_str("remove_script"),
        commands.register_system(remove_script));
    bevy_world_service.add_world_func(
        rt.pool.for_str("translate"),
        commands.register_system(translate));
    bevy_world_service.add_world_func(
        rt.pool.for_str("add_velocity"),
        commands.register_system(add_velocity));
    bevy_world_service.add_world_func(
        rt.pool.for_str("set_gravity"),
        commands.register_system(set_gravity));
}
