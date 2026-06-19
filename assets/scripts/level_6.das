%on_init = NEW_BLOCK `init, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent: CntF = ARG 1, false, CntF
    %data = ARG 2, false, CntF

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 1.0
    %gridcz = COPY 1.0

    // World grid size
    %gridsz = COPY 16.0

    %unitsz_h = COPY 0.5
    %unitsz = COPY 1.5

    %umul: CntF = MODU %ent, 32
    %umul: RealF = CAST %umul, `coerce
    %umul: RealF = DIVR %umul, 0.5
    %umul: RealF = ABSR %umul
    %unitsz = MULR %unitsz, %umul

    %grid_units: CntF = LOAD %data, `grid_units

    // Default computation center
    %cx = COPY 0.0
    %cy = COPY 1.0
    %cz = COPY 0.0

//    %shape = NEW_SEQUENCE [`Cube, x, %unitsz_h, y, %unitsz_h, z, %unitsz_h]
    %shape = NEW_SEQUENCE [`Sphere, %unitsz_h]
    %info =  NEW_SEQUENCE [`subdivisions, 1]
    %shape: Struct = NEW_STRUCT [`kind, %shape, `info, %info]

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
    %wx = MUL %x, %unitsz
    %wx = ADD %wx, %gridcx

    %wy = MUL %y, %unitsz
    %wy: RealF = NEG %wy    // top-down
    %wy = ADD %wy, %gridcy

    %ccr = MODR %wx, 4.0
    %ccr = DIVR %ccr, 2.0
    %ccr = ABSR %ccr
    %ccg = MODR %wy, 4.0
    %ccg = DIVR %ccg, 2.0
    %ccg = ABSR %ccg
    %ccb = MULR %ccr, %ccg
    %hue = MULR %ccb, 360.0
    //%color_enum = NEW_SEQUENCE [`Srgba, `red, %ccr, `green, %ccg, `blue, %ccb, `alpha, 0.25]
    %color_enum = NEW_SEQUENCE [`Hsla, `hue, %hue, `saturation, 1, `lightness, 0.5, `alpha, 1.0]
    //SYS_CALL `print, [%color_enum, "\n"]

    %pos = NEW_STRUCT [x: %wx, z: %wy, y: 2.0]

    %name = SYSCALL `to_string, ["(", %x, ", ", %y, ")"]

    //%mass: RealF = LOAD %data, `block_mass
    %mass = COPY 50.0

    %new = SYSCALL `bevy_spawn, [
        `parent, %ent,
        `scriptable, true,

        `offset, %pos,

        `color, %color_enum,
        //`alpha_mode, `Premultiplied,
        `alpha_mode, `Add,
        //`emissive, 1,
        `roughness, 0.0,
        `cull_face, `None,

        `shape, %shape,
        `rigid_body, `Dynamic,
        `mass, %mass,
        `friction, 0.0,

        `CrosshairTargetable, nil,
        `Name, %name,
    ]

    // State for cell.
    %data = NEW_STRUCT [
        `pos: %pos,
    ]

    // Package ScriptCreationInfo data.
    %sci = NEW_STRUCT [
        path: `scripts/automaton.das,
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

    %module = NEW_STRUCT [
        `on_init: %on_init,
        //`grid_units: 40,
        `grid_units: 20,
        `boom_mass: 10000.0,
        `block_mass: 25.0,
    ]
    RETURN %module
