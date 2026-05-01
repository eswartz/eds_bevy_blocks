%on_update = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = GET_ARG 0, false, Struct
    %ent = GET_ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    %x = MOD %ent, 11
    %x = SUB %x, 5.0

    %z = MOD %ent, 13
    %z = SUB %z, 5.0

    %y = MOD %ent, 3
    %y = ADD %y, 2

    %pos = NEW_STRUCT [x: %x, y: %y, z: %z]
    %size = NEW_STRUCT [x: 1, y: 0.05, z: 1]

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

    // Color::Srgba(Srgba(red, green, blue))
    //%srgba = NEW_STRUCT [red: %r, green: %g, blue: %b]
    //%color = NEW_SEQUENCE [%srgba]
    %color_enum = NEW_SEQUENCE [`Srgba, `red, %r, `green, %g, `blue, %b]

    %rot_x = MOD %step, 45

    %angle = NEW_STRUCT [x:0, y: %rot_x, z: 0]

    //%life = NEW_SEQUENCE [`Temporary, `secs, 240.0]
    %life = NEW_SEQUENCE [`Keep]

    // Make a cuboid.

    %struct = NEW_STRUCT [
        offset: %pos,
        size: %size,
        color: %color_enum,
        mass: 10.0,
        angle: %angle,
        //is_static: true,
        life_policy: %life,

        has_synth: true,
    ]
    %ent = CALL_INTRINSIC `bevy_call_func, [`spawn_cube, %struct]

    %struct = NEW_STRUCT [
        y: 10.0,
    ]
    //%ent = CALL_INTRINSIC `bevy_call_func, [%ent, `add_velocity, %struct]

    %async = NEW_SEQUENCE [`Async]
    %struct = NEW_STRUCT [
        path: `scripts/cube_handler.das,
        mode: %async,
    ]
    CALL_INTRINSIC `bevy_call_func, [%ent, `add_script, %struct]

    CALL_INTRINSIC `delay, [0.5]

    //CALL_INTRINSIC `bevy_call_func, [`set_gravity, 0]

    //CALL_INTRINSIC `delay, [2.5]

    //CALL_INTRINSIC `bevy_call_func, [`set_gravity, -9.8]
    //CALL_INTRINSIC `bevy_call_func, [%ent, `set_gravity, 100.0]

    RETURN nil
}

///////

    //CALL_INTRINSIC `bevy_call_func, [`set_gravity, -9.8]

    %module = NEW_STRUCT [
        `block_size: 0.375,
        `side_length: 3,
        `on_update: %on_update,
        `boom_mass: 50000.0,
        `block_mass: 500.0,
        `static: false,
    ]
    CONSUME_FUEL 3
    RETURN %module
