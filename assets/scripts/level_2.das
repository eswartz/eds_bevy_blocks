/// Metallic plates falling from the sky relative to spawned blue cubes.

%on_update = NEW_BLOCK `update_cb, (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent = ARG 1
    %data = ARG 2
    %step: CntF = LOAD %ctx, `tick_count, true

    %dist: RealF = LOAD %data, `fall_distance, true
    %delay: RealF = LOAD %data, `delay, true
    %life: Tuple = LOAD %data, `lifetime, true
    %restitution: RealF = LOAD %data, `restitution, true

    %x = MOD %ent, 11
    %x = SUB %x, 5.0

    %z = MOD %ent, 13
    %z = SUB %z, 5.0

    %y = MOD %ent, 3
    %y = ADD %y, %dist

    %pos = [x: %x, y: %y, z: %z]

    // Make a color.
    %rr = ADD %x, %step
    %r = DIV %rr, 11.0
    %r = MOD %r, 5.0

    %gg = ADD %z, %step
    %g = DIV %gg, 11.0
    %g = MOD %g, 5.0

    %bb = SUB %step, 17.0
    %b = DIV %z, 13.0
    %b = MOD %b, 5.0

    %hue = MOD %step, 360.0
    %color_enum = [`Hsva, `hue, %hue, `saturation, 0.75, `value, %r]

    %rot_x = MOD %step, 45

    %angle = [y: %rot_x, x: 0, z: 0]
    %angle: Tuple = [`Deg, %angle]

    // Make a cuboid.
    %mesh_mat = [mesh: "models/tile.glb#Mesh0/Primitive0", material: "models/tile.glb#Material0/std"]
    %shape = [`MeshMaterial, %mesh_mat]
    %collider_shape = [`Cube, x, 1.0, y, 0.025, z, 1.0]

    %rest = [`coefficient, %restitution, `combine_rule, `Average]

    %new = SYSCALL `bevy_spawn, [
        `Name, "TILE",
        `Cube, nil,
        `swept_ccd, `linear,

        // `parent, %ent,
        `relative_to, %ent,

        `offset, %pos,
        `rotation, %angle,
        `scale, 0.5,

        `color, %color_enum,

        `rigid_body, `Dynamic,
        `shape, %shape,
        `collider_shape, %collider_shape,
        `friction, 0.9,
        `Restitution, %rest,

        `mass, 10.0,

        `CrosshairTargetable, nil,

        `SurfaceMaterial, `Metal,
        `Projectile, nil,
        //`CollisionEventsEnabled, nil,

        `despawn_after, %life,
        `swept_ccd, `linear,
    ]

    // Package ScriptCreationInfo data.
    %async = [`Async]
    %sci = [
        path: `scripts/cube_handler.das,
        mode: %async,
    ]
    SYSCALL `bevy_call_func, [%new, `add_script, %sci]

    SYSCALL `delay, [%delay]

    RETURN nil
}

///////

    //SYSCALL `bevy_call_func, [`set_gravity, -9.8]

    %ctr = [`x: 6.0, `y: 5.0, `z: -5.0]
    %life = [`Temporary, `secs, 60]
    //%life = [`Keep]

    %init = [
        `fall_distance: 5.0,
        `delay: 1.0,
        `restitution: 0.2,
        `lifetime: %life,
    ]

    %module = [
        `block_size: 0.375,
        `side_length: 3,
        `on_update: %on_update,
        `boom_mass: 500.0,
        `block_mass: 50.0,
        `static: false,

        `center_pos: %ctr,
        `block_data: %init,
    ]
    RETURN %module
