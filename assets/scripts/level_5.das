%on_init = NEW_BLOCK `init, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
//    %ent = ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 4.0
    // World grid size
    %gridsz = COPY 16.0

    // World grid unit count;
    // this squared is # of new entities!
    %grid_units = COPY 128
    //%grid_units = COPY 256
    //%grid_units = COPY 192
    //%grid_units = COPY 16

    %unitsz: RealF = DIV %gridsz, %grid_units
    %unitsz_h: RealF = DIV %unitsz, 2.0

    // Default computation center
    %cx = COPY 0.0
    %cy = COPY 0.0
    %sz = DIV 4.0, 1.0

    // Zoom to specific spot
    %cx = COPY 0.390018071300935778312029
    %cy = COPY 0.138320184041392651089585
    %sz = DIV 1.0, 1024.0
    %sz = DIV %sz, 1048576.0
    //%sz = DIV %sz, 1024.0
    //%sz = DIV 1.0, 2.0

    %max_iters = COPY 5000

    %cellsz = DIV %sz, %grid_units

    // Note: don't try to use alpha here, else we really slow down from OIT sorting!
    %color_enum = NEW_SEQUENCE [`Srgba, `red, 1, `green, 1, `blue, 1]
    %shape = NEW_SEQUENCE [`Plane, x, %unitsz_h, y, %unitsz_h]
    %info =  NEW_SEQUENCE [`subdivisions, 2]
    %shape = NEW_STRUCT [`kind, %shape, `info, %info]

    // Spawn all the grid points, initializing them with
    // the coordinates to render.

    %tag = NEW_SEQUENCE [0]

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

    %pos = NEW_STRUCT [x: %wx, y: %wy, z: 1.0]

    %name = CALL_INTRINSIC `to_string, ["(", %x, ", ", %y, ")"]

    %rot = NEW_STRUCT [x: -90.0]
    %rot = NEW_SEQUENCE [`Deg, %rot]

    %new = CALL_INTRINSIC `bevy_spawn, [
        `offset, %pos,

        //`color, %color_enum,
        `unlit, true,
        `double_sided, true,
        //`color_texture, "textures/palette.png",

        `shape, %shape,
        `rotation, %rot,
        //`rigid_body, `Static,

        `CrosshairTargetable, nil,
        `Name, %name,

        `MeshTag, %tag,
    ]

    //CALL_INTRINSIC `bevy_call_func, [%new, `set_mesh_tag, 0]

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

        `pos: %pos,
    ]
    //CALL_INTRINSIC `print, [%data, "\n"]

    // Package ScriptCreationInfo data.
    %async = NEW_SEQUENCE [`Async]
    %sci = NEW_STRUCT [
        path: `scripts/mandel_img.das,
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
        `on_init: %on_init,
    ]
    CONSUME_FUEL 3
    RETURN %module
