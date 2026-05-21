%on_playing = NEW_BLOCK `update_cb, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
//    %ent = ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 1.0
    // World grid size
    %gridsz = COPY 4.0

    // World grid unit count
    %grid_units = COPY 128      // this squared is # of new entities

    %unitsz: RealF = DIV %gridsz, %grid_units
    %unitsz_h: RealF = DIV %unitsz, 2.0

    // Default computation center
    %cx = COPY 0.0
    %cy = COPY 0.0
    %sz = DIV 4.0, 1.0

    // Zoom to specific spot
    %cx = COPY 0.39001807130093577
    %cy = COPY 0.13832018404139265
    //%sz = DIV 1.0, 1024.0
    %sz = DIV 1.0, 2.0

    %max_iters = COPY 1_000

    %cellsz = DIV %sz, %grid_units

    // Note: don't try to use alpha here, else we really slow down from OIT sorting!
    %color_enum = NEW_SEQUENCE [`Srgba, `red, 1, `green, 1, `blue, 0.75]
    %shape = NEW_SEQUENCE [`Cube, x, %unitsz_h, y, %unitsz_h, z, 0.01]

    // Spawn all the grid points, initializing them with
    // the coordinates to render.

    %grid_units_h = LSHR %grid_units, 1
    %y: IntF = SUB 0, %grid_units_h

y_loop:
    %cmp = LESS_SINT %y, %grid_units_h
    IF_TRUE %cmp, y_step, exit

y_step:
    %x: IntF = SUB 0, %grid_units_h

x_loop:
    %cmp = LESS_SINT %x, %grid_units_h
    IF_TRUE %cmp, x_step, y_next

x_step:
    //%wx = DIV %x, %gridsx
    %wx = MUL %x, %unitsz_h
    %wx = ADD %wx, %gridcx

    //%wy = DIV %y, %gridsy
    %wy = MUL %y, %unitsz_h
    %wy: RealF = NEG %wy    // top-down
    %wy = ADD %wy, %gridcy

    %pos = NEW_STRUCT [x: %wx, y: %wy, z: -0.5]

    %name = CALL_INTRINSIC `to_string, ["(", %x, ", ", %y, ")"]

    %new = CALL_INTRINSIC `bevy_spawn, [
        `offset, %pos,

        `color, %color_enum,
        `shape, %shape,
        `unlit, true,

        `CrosshairTargetable, nil,
        `Name, %name,
    ]
    //CALL_INTRINSIC `print, ["making ", %name, " -> ", %new, "\n"]
    //CONSUME_FUEL 10

    // State for cell.
    %cr: RealF = MUL %x, %cellsz
    %cr: RealF = ADD %cr, %cx
    %ci: RealF = MUL %y, %cellsz
    %ci: RealF = ADD %ci, %cy

    %data = NEW_STRUCT [
        `cr: %cr,
        `ci: %ci,
        `zr: 0.0,
        `zi: 0.0,
        `max_iters: %max_iters,
        `iters: 0,
    ]
    //CALL_INTRINSIC `print, [%data, "\n"]

    // Package ScriptCreationInfo data.
    %async = NEW_SEQUENCE [`Async]
    %sci = NEW_STRUCT [
        path: `scripts/mandel.das,
        mode: %async,
    ]
    CALL_INTRINSIC `bevy_call_func, [%new, `add_script, %sci, %data]
    //CONSUME_FUEL 1

x_next:
    %x = ADD %x, 1
    JUMP x_loop

y_next:
    %y = ADD %y, 1
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
