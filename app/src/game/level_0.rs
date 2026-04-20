use crate::assets::*;
use crate::game::BoomMass;
use crate::game::Cube;
use crate::game::ScriptMain;
use avian3d::math::Scalar;
use eds_bevy_common::*;

use avian3d::prelude::*;
use bevy::prelude::*;

use fedry_bevy_plugin::prelude::*;
use fedry_runtime::prelude::RtNumber;
use fedry_runtime::prelude::RtReal;
use fedry_runtime::prelude::RtSInt;

pub(crate) const ID: &str = "level0";
pub(crate) const NAME: &str = "Level 0";

fn register_level(mut list: ResMut<LevelList>, maps: Res<MapAssets>) {
    list.0.push(LevelInfo {
        id: ID.to_string(),
        label: NAME.to_string(),
        scene: maps.level_0.clone(),
    });
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ProgramState::New), register_level)
            .add_systems(
                OnEnter(LevelState::LevelLoaded),
                (
                    on_level_loaded,
                    setup_skybox,
                )
                    .run_if(is_in_level(ID)),
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

fn on_level_loaded(
    mut commands: Commands,
    world: Res<WorldMarkerEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,

    scripting: Res<ScriptRuntime>,
    script_assets: Res<ScriptAssets>,
    modules: Res<Assets<ScriptModule>>,
) -> Result {

    let script: Script<ScriptMain> = Script::new(
        &*modules,
        script_assets.level_0.clone(),
        &scripting.rt,
        "on_update",
        ExecutionMode::Async,
        // ExecutionMode::Sync,
    )?;

    let cube_size = if let Some(size) = script.get_module().map().get(&scripting.rt.pool.for_str("block_size"))
    && let Some(size) = RtReal::new(&size) {
        *size as f32
    } else {
        0.75
    };

    let cube_mass = if let Some(mass) = scripting.get_struct_value(script.get_module(), "block_mass")
    && let Some(mass) = RtNumber::new(&mass) {
        mass.value_real() as f32
    } else {
        10.0f32
    };

    // Spawn cube stacks
    let mat = materials.add(Color::srgb(0.2, 0.7, 0.9));
    let cube_mesh = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

    #[allow(unused)]
    let cuboid_size = cube_size * 0.95;
    #[allow(unused)]
    let cuboid_round = (cube_size - cuboid_size) / 2.0;

    const CUBE_GAP: f32 = 0.05;
    let axis_scale = Vec3::splat(cube_size + CUBE_GAP);

    let collider = Collider::cuboid(
        cube_size as Scalar,
        cube_size as Scalar,
        cube_size as Scalar,
    );
    // let collider = Collider::round_cuboid(
    //     (cube_size - 0.05 * 2.0) as Scalar,
    //     (cube_size - 0.05 * 2.0) as Scalar,
    //     (cube_size - 0.05 * 2.0) as Scalar,
    //     0.05
    // );

    let half_size = if let Some(half_side_length) = scripting.get_struct_value(script.get_module(),
    "half_side_length")
    && let Some(half_side_length) = RtSInt::new(&half_side_length) {
        *half_side_length as i32
    } else {
        6
    };

    let rigid_body = if let Some(is_static) = scripting.get_struct_value(script.get_module(), "static")
    && is_static.as_bool() {
        RigidBody::Static
    } else {
        RigidBody::Dynamic
    };

    let boom_mass = if let Some(mass) = scripting.get_struct_value(script.get_module(), "boom_mass")
        && let Some(mass) = RtNumber::new(&mass) {
        mass.value_real() as f32
    } else {
        50.0f32
    };
    commands.insert_resource(BoomMass(boom_mass));

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
                        Mass(cube_mass),
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


fn setup_skybox(
    mut commands: Commands,
    skybox_q: Query<Entity, (With<SkyboxModel>,)>,
    cam_q: Query<Entity, (With<Camera3d>, With<WorldCamera>)>,
    skyboxes: Res<CommonSkyboxAssets>,
) {
    let Ok(cam) = cam_q.single() else { return };

    // If there isn't one in the level, add a default?
    if skybox_q.is_empty() {
        // let with_reflection_probe = Some((cam, 100.0));  // looks ... not so good when real lights are present
        let with_reflection_probe = None;

        commands.entity(cam).insert(SkyboxModel {
            image: Some(skyboxes.dresden_station_night.clone()),
            brightness: bevy::prelude::light_consts::lux::CIVIL_TWILIGHT,
            mapping: CubemapMapping::From1_0_2f_3f_4_5,
            with_reflection_probe,
            .. default()
        });
    }
    commands.insert_resource(SkyboxSetup::WaitingSkybox);
}
