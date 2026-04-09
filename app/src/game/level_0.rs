use crate::assets::*;
use crate::game::Cube;
use avian3d::math::Scalar;
use eds_bevy_common::*;

use avian3d::prelude::*;
use bevy::prelude::*;
use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::RtSInt;

pub(crate) const ID: &str = "level0";
pub(crate) const NAME: &str = "Level 0";

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                    on_level_loaded.run_if(is_in_level(ID)),
            )
            .add_systems(
                Update,
                    check_pause_request,
            )
        ;
    }
}

fn check_pause_request(
    paused: ResMut<PauseState>,
    mut control: ResMut<ScriptControl>,
) {
    if !paused.is_changed() {
        return
    }
    control.set_paused(paused.is_paused());
}

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_0.clone(),
    });
}

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,

    scripting: Res<ScriptRuntime>,
    script_assets: Res<ScriptAssets>,
    modules: Res<Assets<ScriptModule>>,
) -> Result {
    const CUBE_SIZE: f32 = 0.75;
    const CUBE_MASS: f32 = 50.0 * 2.0 / 5.0;

    // Spawn cube stacks
    let mat = materials.add(Color::srgb(0.2, 0.7, 0.9));
    let cube_mesh = meshes.add(Cuboid::new(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE));

    #[allow(unused)]
    let cuboid_size = CUBE_SIZE * 0.95;
    #[allow(unused)]
    let cuboid_round = (CUBE_SIZE - cuboid_size) / 2.0;

    const CUBE_GAP: f32 = 0.05;
    let axis_scale = Vec3::splat(CUBE_SIZE + CUBE_GAP);

    let collider = Collider::cuboid(
        CUBE_SIZE as Scalar,
        CUBE_SIZE as Scalar,
        CUBE_SIZE as Scalar,
    );

    let script = Script::new(modules
        .get(&script_assets.level_0)
        .ok_or(anyhow::anyhow!("missing script asset"))?,
    )?;
    let half_size = if let Some(side_length) = script.get_module().map().get(&scripting.rt.pool.for_str("side_length"))
    && let Some(side_length) = RtSInt::new(&side_length) {
        *side_length as i32 / 2
    } else {
        6
    };

    let rigid_body = if let Some(is_static) = script.get_module().map().get(&scripting.rt.pool.for_str("static"))
    && is_static.as_bool() {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let center = Vec3::new(-5.0, axis_scale.y / 2.0, 5.0);
    for x in -half_size..half_size {
        for y in 0..half_size * 2 {
            for z in -half_size..half_size {
                let position =
                    Vec3::new(x as f32, y as f32, z as f32) * axis_scale + center;
                commands.spawn((
                    (
                        ChildOf(world.0),
                        Name::new("CUBE"),
                        Cube,
                        Spawned,
                        CrosshairTargetable,
                        Mesh3d(cube_mesh.clone()),
                        MeshMaterial3d(mat.clone()),
                        // Transform::from_translation(position).with_scale(Vec3::splat(cube_size as f32)),
                        Transform::from_translation(position),
                    ),
                    (
                        // CollisionEventsEnabled,
                        // RigidBody::Dynamic,
                        rigid_body.clone(),
                        collider.clone(),
                        // Collider::round_cuboid(cuboid_size, cuboid_size, cuboid_size, cuboid_round),
                        // Restitution::new(0.0),
                        Restitution::new(0.0), //.with_combine_rule(CoefficientCombine::Min),
                        Friction::new(0.9),
                        SleepThreshold {
                            linear: 0.125,
                            angular: 0.125,
                        },
                        LinearDamping(0.25),
                        AngularDamping(0.25),
                        // CenterOfMass::new(0., -cube_size / 4.0, 0.0),
                        Mass(CUBE_MASS),
                        CollisionMargin(0.0),
                        // CollisionMargin(0.01),
                    ),
                    // Dominance(-y as i8 - 1),
                    // (
                    //     // SweptCcd::default(),
                    //     SweptCcd::LINEAR,
                    // )
                    (script.clone(),),
                ));
            }
        }
    }

    // commands.insert_resource(Spawning(false));
    // commands.insert_resource(SpawnDelay(Duration::from_secs(1)));
    // commands.insert_resource(SpawnTimer(Timer::new(Duration::from_secs(1), TimerMode::Repeating)));
    // commands.insert_resource(ShakeTime(Duration::ZERO));
    Ok(())
}
