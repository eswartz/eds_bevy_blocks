use avian3d::math::*;
use avian3d::prelude::*;
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;

use eds_bevy_common::*;

/// Spawn player entities into the world, but leave them inert until the map is loaded.
pub(crate) fn spawn_player(world: &mut World, user_id: Uuid) -> Entity {

    info!("Spawning player {user_id}");
    let mut exist_ent = None;
    {
        let mut player_q = world.query::<(Entity, &Player)>();
        for (ent, player) in player_q.query(world) {
            if player.0 == user_id {
                exist_ent = Some(ent);
                break;
            }
        }
    }
    if let Some(ent) = exist_ent {
        // Already here, so kill it.
        world.despawn(ent);
    }

    // let mut meshes =  world.get_resource_mut::<Assets<Mesh>>().unwrap();

    // This matches the eye height of the Quake player. A small figure.
    let player_scale = Vec3::new(0.5, 1.5, 0.3);
    // let mesh = create_player_mesh(&mut meshes, player_scale);

    // let mut materials = world.get_resource_mut::<Assets<StandardMaterial>>().unwrap();
    // let mat = materials.add(StandardMaterial {
    //     base_color: Color::WHITE,
    //     emissive: LinearRgba::BLUE * 2.0,
    //     ..default()
    // });

    let radius = 0.333;
    let collider_shape = Collider::capsule(
        radius as Scalar,
        (player_scale.y - radius * 2. - player_scale.z).max(0.25) as Scalar,
    );
    // let collider_shape = Collider::cuboid(
    //     radius as Scalar, player_scale.y.max(0.25) as Scalar, radius as Scalar,
    //     // (player_scale.y - radius * 2. - player_scale.z).max(0.25) as Scalar,
    // );

    let rounded_size = 0.125 as Scalar;
    let head_size = (player_scale.z as Scalar) - rounded_size;
    let collider_shape = Collider::compound(vec![
        (
            Vector::ZERO,
            Quaternion::IDENTITY,
            collider_shape,
        ),
        (
            Vector::new(0., (player_scale.y - player_scale.z * 2.0) as Scalar - head_size, 0.),
            Quaternion::IDENTITY,
            Collider::round_cuboid(head_size, head_size, head_size, rounded_size),
        ),

    ]);

    // let collider_shape = Collider::cuboid(
    //     radius as Scalar,
    //     player_scale.y as Scalar,
    //     radius as Scalar,
    // );

    let mode = world.get_resource::<PlayerMode>().unwrap().clone();

    let player = world.spawn((
        Name::new("Player"),
        DespawnOnExit(ProgramState::InGame),
        (
            Player(user_id),
            PlayerMovement::default(),
            PlayerLook::default(),
            // PlayerCheats::default(),
            // Mesh3d(mesh),
            // MeshMaterial3d(mat),

            Transform::IDENTITY,
            Visibility::Inherited,  // needed if no Mesh*
        ),
        (
            // RigidBody::Kinematic,
            RigidBody::Dynamic,

            (
                Mass(75.),
                CenterOfMass(player_scale / 2.),
                Restitution::new(0.0),
                Friction::ZERO.with_dynamic_coefficient(0.).with_static_coefficient(0.9),
            ),

            // Do not let physics modify rotation.
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),

            collider_shape.clone(),
            default_player_collision_layers(),

            // Try to avoid falling through trimesh floor.
            CollisionMargin(0.01),
            SweptCcd::default(),

            // Avoid flying too much when e.g. colliding with a projectile.
            MaxLinearSpeed(4096.0),

            GravityScale(if mode == PlayerMode::Fps { 1.0 } else { 0.0 }),
        ),

        // This child component is used to:
        // (1) interact with tiles/areas/buttons
        // (2) provide a collider shape that extends into the
        // ground so that when we step on a tile, we don't lose
        // contact with it.
        // (3) leave the player "body" collider more amenable to
        // ordinary movement in a world.
        // (4) modify collisions to avoid "entering" water at the edge
        children![(
            Name::new("Game Collider"),
            Transform::from_translation(Vec3::new(0., -player_scale.x * 0.05, -player_scale.x)),
            Collider::cuboid(player_scale.x as Scalar, player_scale.y as Scalar, player_scale.x /* yes */ as Scalar),
            // {
            //     let csz = Vec3::new(player_scale.x as f32, player_scale.x as f32 * 0.75, player_scale.x as f32 * 2.0);
            //     Collider::cuboid(csz.x, csz.y, csz.z)
            // },
            // Collider::sphere((player_scale.x / 2.0) as _),
			CollisionLayers::new([
                GameLayer::Player,
            ], [
                GameLayer::Gameplay,
            ]),
            ActiveCollisionHooks::MODIFY_CONTACTS,
        )]
    ))
    .id();

    player
}

pub fn default_player_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameLayer::Player, [
        GameLayer::Default, GameLayer::World,
        // GameLayer::Gameplay, // the Game Collider does this
        GameLayer::Projectiles,
    ])
}
