%on_update = NEW_BLOCK `update_cb, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, CntF
    %data = ARG 2, false, Struct
    %step: CntF = LOAD %ctx, `tick_count, true

    %dist: RealF = LOAD %data, `fall_distance, true
    %delay: RealF = LOAD %data, `delay, true
    %life: Tuple = LOAD %data, `lifetime, true

    %x = MOD %ent, 11
    %x = SUB %x, 5.0

    %z = MOD %ent, 13
    %z = SUB %z, 5.0

    %y = MOD %ent, 3
    %y = ADD %y, %dist

    %pos = NEW_STRUCT [x: %x, y: %y, z: %z]

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
    %color_enum = NEW_SEQUENCE [`Hsva, `hue, %hue, `saturation, 0.75, `value, %r]

    %rot_x = MOD %step, 45

    %angle = NEW_STRUCT [y: %rot_x, x: 0, z: 0]
    %angle: Tuple = NEW_SEQUENCE [`Deg, %angle]

    // Make a cuboid.

    %shape = NEW_SEQUENCE [`Model, "models/tile.glb#Mesh0"]
    %collider_shape = NEW_SEQUENCE [`Cube, x, 1.0, y, 0.025, z, 1.0]

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
        `friction, 1.0,

        `mass, 10.0,

        `CrosshairTargetable, nil,

        `despawn_after, %life,

    ]

    // Package ScriptCreationInfo data.
    %async = NEW_SEQUENCE [`Async]
    %sci = NEW_STRUCT [
        path: `scripts/cube_handler.das,
        mode: %async,
    ]
    SYSCALL `bevy_call_func, [%new, `add_script, %sci]

    SYSCALL `delay, [%delay]

    RETURN nil
}

///////

    //SYSCALL `bevy_call_func, [`set_gravity, -9.8]

    %ctr = NEW_STRUCT [`x: 6.0, `y: 5.0, `z: -5.0]
    //%life = NEW_SEQUENCE [`Temporary, `secs, 60]
    %life = NEW_SEQUENCE [`Keep]

    %init = NEW_STRUCT [
        `fall_distance: 10.0,
        `delay: 3.0,
        `lifetime: %life,
    ]

    %module = NEW_STRUCT [
        `block_size: 0.375,
        `side_length: 2,
        `on_update: %on_update,
        `boom_mass: 50000.0,
        `block_mass: 50.0,
        `static: false,

        `center_pos: %ctr,
        `block_data: %init,
    ]
    RETURN %module
