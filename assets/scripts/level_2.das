%on_update = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    %x = MOD %ent, 11
    %x = SUB %x, 5.0

    %z = MOD %ent, 13
    %z = SUB %z, 5.0

    %y = MOD %ent, 3
    %y = ADD %y, 2

    %pos = NEW_STRUCT [x: %x, y: %y, z: %z]
    %size = NEW_STRUCT [x: 1, y: 0.25, z: 1]

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
    //%color_enum = NEW_SEQUENCE [`Srgba, `red, %r, `green, %g, `blue, %b]

    %hue = MOD %step, 360.0
    %color_enum = NEW_SEQUENCE [`Hsva, `hue, %hue, `saturation, 0.75, `value, %r]

    %rot_x = MOD %step, 45

    %angle = NEW_STRUCT [y: %rot_x, x: 0, z: 0]
    //%angle = NEW_STRUCT [x: %rot_x, y: 0, z: 0]
    //%angle = NEW_STRUCT [x: 0, y: 0, z: %rot_x]
    //w%angle = NEW_STRUCT [x: %rot_x, y: 0, z: %rot_x]
    %angle = NEW_SEQUENCE [`Deg, %angle]

    %life = NEW_SEQUENCE [`Temporary, `secs, 360.0]
    //%life = NEW_SEQUENCE [`Keep]

    // Make a cuboid.

    //%struct = NEW_STRUCT [
    //    offset: %pos,
    //    size: %size,
    //    color: %color_enum,
    //    mass: 50.0,
    //    angle: %angle,
    //    is_static: false,
    //    life_policy: %life,
    //    has_synth: false,
    //]

    %shape = NEW_SEQUENCE [`Model, "models/tile.glb#Mesh0"]
    %collider_shape = NEW_SEQUENCE [`Cube, x, 1.25, y, 0.025, z, 1.25]
    //%collider_shape = NEW_SEQUENCE [`Model, "models/tile.glb#Mesh0"]

    %new = CALL_INTRINSIC `bevy_spawn, [
        `parent, %ent,

        `Name, "TILE",
        `offset, %pos,
        `rotation, %angle,

        `color, %color_enum,

        `rigid_body, `Dynamic,
        `shape, %shape,
        `collider_shape, %collider_shape,

        `mass, 50.0,

        `Cube, nil,
    ]

    //%struct = NEW_STRUCT [y: 1.0,]
    //CALL_INTRINSIC `bevy_call_func, [%ent, `add_velocity, %struct]

    // Package ScriptCreationInfo data.
    %async = NEW_SEQUENCE [`Async]
    %sci = NEW_STRUCT [
        path: `scripts/cube_handler.das,
        mode: %async,
    ]
    //CALL_INTRINSIC `bevy_call_func, [%new, `add_script, %sci]

    //CALL_INTRINSIC `print, [%ent, ' - 1\n']
    CALL_INTRINSIC `delay, [0.5]
    //CALL_INTRINSIC `print, [%ent, ' - 2\n']

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
        `side_length: 4,
        `on_update: %on_update,
        `boom_mass: 50000.0,
        `block_mass: 50.0,
        `static: false,
    ]
    RETURN %module
