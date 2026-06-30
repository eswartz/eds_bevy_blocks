// Mandelbrot example, using one entity per grid square.
//
// Each square has a script to handle its own computation,
// and that square is moved and recolored once an answer is reached.
//
// (Yes, it's ridiculously expensive.)

%on_init = NEW_BLOCK `init, (ctx: Struct, ent: Cnt64, data: Struct) {
    %ctx = ARG 0
    %ent = ARG 1
    %data = ARG 2

    %step: CntF = LOAD %ctx, `tick_count, true

    %grid_units: CntF = LOAD %data, `grid_units, true
    %max_iters: CntF = LOAD %data, `max_iters, true
    %range_min: CntF = LOAD %data, `range_min, true
    %range_size: CntF = LOAD %data, `range_size, true
    %depth_size: RealF = LOAD %data, `depth_size, true

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 4.0
    // World grid size
    %gridsz = COPY 16.0

    %unitsz: RealF = DIV %gridsz, %grid_units
    %unitsz_h: RealF = DIV %unitsz, 2.0

    // Default computation center
    //%cx = COPY 0.0
    //%cy = COPY 0.0
    //%sz = DIV 4.0, 1.0

    // Zoom to specific spot
    %cx = LOAD %data, `cx
    %cy = LOAD %data, `cy
    %sz = LOAD %data, `sz

    %cellsz = DIV %sz, %grid_units

    // Note: don't try to use alpha here, else we really slow down from OIT sorting!
    %color_enum = NEW_SEQUENCE [`Srgba, `red, 1, `green, 1, `blue, 1]
    //%shape = NEW_SEQUENCE [`Plane, x, %unitsz_h, y, %unitsz_h]
    %shape = NEW_SEQUENCE [`Cube, x, %unitsz_h, y, %unitsz_h, z, 0.01]
    //%info =  NEW_SEQUENCE [`subdivisions, 2]
    %info =  NEW_SEQUENCE []
    %shape: Struct = NEW_STRUCT [`kind, %shape, `info, %info]

    // Spawn all the grid points, initializing them with
    // the coordinates to render.

    %tag = NEW_SEQUENCE [0]

    %grid_units_h = LSHR %grid_units, 1
    %y: IntF = SUB 0, %grid_units_h

y_loop:
    %cmp = LESS_I %y, %grid_units_h
    IF_TRUE %cmp, y_step, exit

y_step:
    %x: IntF = SUB 0, %grid_units_h

x_loop:
    %cmp = LESS_I %x, %grid_units_h
    IF_TRUE %cmp, x_step, y_next

x_step:
    //%wx = DIV %x, %gridsx
    %wx = MUL %x, %unitsz_h
    %wx = ADD %wx, %gridcx

    //%wy = DIV %y, %gridsy
    %wy = MUL %y, %unitsz_h
    %wy: RealF = NEG %wy    // top-down
    %wy = ADD %wy, %gridcy

    %pos = NEW_STRUCT [x: %wx, y: %wy, z: 1.0]

    %name = SYSCALL `to_string, ["(", %x, ", ", %y, ")"]

    %rot = NEW_STRUCT [x: -90.0]
    %rot: Tuple = NEW_SEQUENCE [`Deg, %rot]

    %prod: CntF = MUL %x, %y
    SET_INDEX %tag, 0, %prod

    %new = SYSCALL `bevy_spawn, [
        `parent, %ent,
        `scriptable, true,

        `offset, %pos,

        //`color, %color_enum,
        `unlit, true,
        //`double_sided, true,
        `cull_face, `None,
        //`color_texture, "textures/palette.png",

        `shape, %shape,
        //`rotation, %rot,
        //`rigid_body, `Static,

        //`CrosshairTargetable, nil,    // avoid unwanted crosshair activity
        `Name, %name,

        `MeshTag, %tag,
    ]

    //SYSCALL `print, ["making ", %name, " -> ", %new, "\n"]

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
        `iters: 0,
        `max_iters: %max_iters,
        `range_min: %range_min,
        `range_size: %range_size,
        `depth_size: %depth_size,
        `depth: 8.0,

        `pos: %pos,
    ]
    //SYSCALL `print, [%data, "\n"]

    // Package ScriptCreationInfo data.
    %sci = NEW_STRUCT [
        path: `scripts/mandel_ent.das,
        mode: `Async,
    ]
    SYSCALL `bevy_call_func, [%new, `add_script, %sci, %data]

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

    %init_data = NEW_STRUCT [
        `max_iters: 4000,
        `range_min: 0,
        `range_size: 3000,
        `depth_size: 4.0,

        //`cx: 0.390018071300935778312029,
        //`cy: 0.138320184041392651089585,
        //`sz: 0.0000005,

        // `cx: -0.74364643,
        // `cy: 0.1318054051,
        // `sz: 0.00000005,

        `cx: -0.743_643_900_055,
        `cy: 0.131_828_895_901,
        `sz: 0.0000001,

        //`grid_units: 32,
        //`grid_units: 64,
        //`grid_units: 128,
        `grid_units: 192,
        //`grid_units: 256,
        //`grid_units: 512, // too too many
    ]

    %module = NEW_STRUCT [
        `on_init: %on_init,
        `init_data: %init_data,
    ]
    RETURN %module
