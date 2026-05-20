%on_playing = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
//    %ent = ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    // Range
    %xlo = COPY -1.5
    %xhi = COPY 1.0
    %ylo = COPY -1.25
    %yhi = COPY 1.25

    // Grid size
    %sz: RealF = DIV 1.0, 8.0
    %hsz: RealF = DIV %sz, 2.0

    // Move to center of grid square
    %xlo = ADD %xlo, %hsz
    %xyo = ADD %ylo, %hsz

    //%color_enum = NEW_SEQUENCE [`Srgba, `red, 0.5, `green, 0.5, `blue, 0.5]
    %shape = NEW_SEQUENCE [`Cube, x, %sz, y, %sz, z, %sz]
    %size = NEW_STRUCT [x: %sz, y: %sz, z: %sz]

    %y = COPY %ylo

y_loop:
    %cmp = LESS_REAL %y, %yhi
    IF_TRUE %cmp, y_step, exit

y_step:
    %x = COPY %xlo

x_loop:
    %cmp = LESS_REAL %x, %xhi
    IF_TRUE %cmp, x_step, y_next

x_step:
    %wx = COPY %x

    %wy = SUB %yhi, %y      // top-down
    %wy = ADD %wy, 1.0
    %pos = NEW_STRUCT [x: %wx, y: %wy, z: 1.0]

    %name = CALL_INTRINSIC `to_string, ["(", %x, ", ", %y, ")"]

    %color_enum = NEW_SEQUENCE [`Srgba, `red, %x, `green, 0.5, `blue, %y]
    %new = CALL_INTRINSIC `bevy_spawn, [
        `offset, %pos,

        `color, %color_enum,
        `shape, %shape,

        `CrosshairTargetable, nil,
        `Name, %name,
    ]
    //CALL_INTRINSIC `print, ["making ", %name, " -> ", %new, "\n"]

    %async = NEW_SEQUENCE [`Async]
    %struct = NEW_STRUCT [
        path: `scripts/mandel.das,
        mode: %async,
    ]
    CALL_INTRINSIC `bevy_call_func, [%new, `add_script, %struct]


x_next:
    %x: RealF = ADD %x, %sz
    JUMP x_loop

y_next:
    %y: RealF = ADD %y, %sz
    JUMP y_loop

exit:
    RETURN nil
}

///////

    %module = NEW_STRUCT [
        `on_playing: %on_playing,
    ]
    CONSUME_FUEL 3
    RETURN %module
