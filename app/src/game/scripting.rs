use std::any::TypeId;
use std::sync::Arc;

use crate::game::Cube;
use crate::game::OurMidiSynth;
use bevy::ecs::world::CommandQueue;
use eds_bevy_common::*;

use avian3d::prelude::*;
use bevy::prelude::*;

use fedry_bevy_plugin::bevy_world_function_service::BevyWorldFunctionService;
use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::*;

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

#[derive(Reflect, Debug, Clone)]
struct CubeInfo {
    offset: Vec3,
    size: Vec3,
    color: Color,
    mass: f32,
    is_static: bool,
    has_synth: bool,
}

impl Default for CubeInfo {
    fn default() -> Self {
        Self {
            offset: Vec3::ZERO,
            size: Vec3::splat(1.0),
            color: Color::WHITE,
            mass: 0.0,
            is_static: false,
            has_synth: false,
        }
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
}

fn spawn_cube(In((entity, rt, args)): In<(Entity, Arc<Runtime>, Vec<ObjectPtr>)>, world: &mut World) -> ExecResult {
    if args.is_empty() {
        return Err(RuntimeError::MissingArgument(0));
    }

    let xfrm = {
        let mut query = world.query::<&Transform>();
        query.get(world, entity).map(|x| *x).unwrap_or_default()
    };
    let world_entity = {
        let world_entity = world.get_resource::<WorldMarkerEntity>().ok_or(RuntimeError::LiteralError(format!("no WorldMarkerEntity")))?;
        world_entity.0
    };
    let info = {
        let type_regy = world.get_resource::<AppTypeRegistry>().ok_or(RuntimeError::LiteralError(format!("no AppTypeRegistry")))?;
        convert_obj_to_value::<CubeInfo>(&rt, &type_regy, TypeId::of::<CubeInfo>(), &args[0])
            .map_err(|e| RuntimeError::LiteralError(format!("{e}")))?
    };

    let mesh = {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().ok_or(RuntimeError::LiteralError(format!("no Assets<Mesh>")))?;
        meshes.add(Cuboid::from_size(info.size))
    };

    let collider = Collider::cuboid(info.size.x, info.size.y, info.size.z);

    let mat = {
        let mut materials = world.get_resource_mut::<Assets<StandardMaterial>>().ok_or(RuntimeError::LiteralError(format!("no Assets<StandardMaterial>")))?;
        materials.add(info.color)
    };

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);

    let id = commands.spawn((
        (
            ChildOf(world_entity),
            Name::new("SCRIPTED CUBE"),
            Cube,
            Spawned,
            CrosshairTargetable,
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(info.offset + xfrm.translation),
        ),
        (
            if info.is_static { RigidBody::Static } else { RigidBody::Dynamic },
            collider,
            Restitution::new(0.0),
            Friction::new(0.9),
            SleepThreshold {
                linear: 0.125,
                angular: 0.125,
            },
            LinearDamping(0.25),
            AngularDamping(0.75),
            Mass(if info.mass <= 0.0 { 1.0 } else { info.mass }),
            CollisionMargin(0.0),
        ),
        CollisionLayers::new(
            GameLayer::World,
            [
                GameLayer::Default,
                GameLayer::World,
                GameLayer::Player,
                GameLayer::Projectiles,
            ],
        ),
    )).id();

    if info.has_synth {
        commands.entity(id).insert(OurMidiSynth);
    }

    queue.apply(world);

    Ok(ExecState::Result(rt.for_typed_uint(&rt.types.u64_type, id.to_bits() as usize)?))
}

fn send_midi_message(In((_entity, rt, args)): In<(Entity, Arc<Runtime>, Vec<ObjectPtr>)>,
    mut commands: Commands,
    type_regy: Res<AppTypeRegistry>,
) -> ExecResult {
    if args.len() < 2 {
        return Err(RuntimeError::MissingArgument(1));
    }

    let Some(target_ent) = RtUInt::new(&args[0]) else {
        return Err(RuntimeError::LiteralError(format!("expected an entity, got {}",
            RtDisplay::new(&rt, &args[0]))));
    };
    let delay = {
        if args.len() >= 3 {
            let Some(delay) = RtNumber::new(&args[2]) else {
                return Err(RuntimeError::LiteralError(format!("expected a number for the delay argument, got {}",
                    RtDisplay::new(&rt, &args[2]))));
            };
            if delay.as_real() < 0.0 {
                return Err(RuntimeError::LiteralError(format!("delay cannot be negative, got {}",
                    RtDisplay::new(&rt, &args[2]))));
            }
            delay.as_real() as f32
        } else {
            0.0f32
        }
    };

    let command = convert_obj_to_value::<SynthCommand>(
        &rt, &type_regy, TypeId::of::<SynthCommand>(), &args[1])
    .map_err(|e| RuntimeError::LiteralError(format!("{e}")))?;

    let message = SynthMessage::new(Entity::from_bits(*target_ent as u64), command).after_secs(delay);
    commands.write_message(message);

    Ok(ExecState::Result(ObjectPtr::for_nil()))
}
