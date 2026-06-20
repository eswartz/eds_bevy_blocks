%on_init = NEW_BLOCK `init, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent: CntF = ARG 1, false, CntF
    %data: Struct = ARG 2, false, Struct

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 1.0
    %gridcz = COPY 1.0

    // World grid size
    %gridsz = COPY 16.0

    %unitsz_h = COPY 0.25
    %unitsz = COPY 1.5

    %umul: CntF = MODU %ent, 32
    %umul = XORU %umul, 0xa
    %umul = MAXU %umul, 0x4
    %umul: RealF = CAST %umul, `coerce
    %umul: RealF = DIVR %umul, 0.5
    %umul: RealF = ABSR %umul
    %unitsz = MULR %unitsz, %umul
    %unitsz = MAXR %unitsz, 0.5
    %unitsz = MINR %unitsz, 5.0

    %grid_units: CntF = LOAD %data, `grid_units

    // Default computation center
    %cx = COPY 0.0
    %cy = COPY 0.0
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
    %wx = MUL %x, %gridsz
    %wx = ADD %wx, %gridcx

    %wy = MUL %y, %gridsz
    // %wy: RealF = NEG %wy    // top-down
    %wy = ADD %wy, %gridcy

    %ccr = DIVR %wx, 555555.0
    %ccg = DIVR %wy, 13333.0
    %ccb = MULR %ccr, %ccg
    %ccr = MODR %ccr, 1.0
    %ccg = MODR %ccg, 1.0
    %ccb = MODR %ccb, 1.0
    %hue = MULR %ccb, 360.0
    //%color_enum = NEW_SEQUENCE [`Srgba, `red, %ccr, `green, %ccg, `blue, %ccb, `alpha, 0.25]
    %color_enum = NEW_SEQUENCE [`Hsla, `hue, %hue, `saturation, 1, `lightness, 0.5, `alpha, 1.0]
    //SYSCALL `print, [%ccr, "/", %ccg, " "]

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
        `diffuse_transmission, 0.5,
        `specular_transmission, 0.5,
        `thickness, 0.5,

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

    STORE %data, `tick, 0
    RETURN nil
}

///////

%on_update = NEW_BLOCK `update, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent: CntF = ARG 1, false, CntF
    %data = ARG 2, false, Struct

//    //%step: CntF = LOAD %ctx, `tick_count, true
//    %tick: CntF = LOAD %data, `tick, false
//    %tick: CntF = ADDU %tick, 1
//    STORE %data, `tick, %tick

//    %f = MODU %tick, 2
//    //SYSCALL `print, [%f, ' ']
//    IF_ZERO %f, a, b
//a:
//    %vel = NEW_STRUCT [`x: 0.1, `y: -0.5, `z: -0.1]
//    JUMP c
//b:
//    %vel = NEW_STRUCT [`x: -0.1, `y: 0.5, `z: 0.1]
//    JUMP c
//c:
//    SYSCALL `bevy_call_func, [%ent, `set_velocity, %vel]
//    SYSCALL `delay, [2]

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    %ppos = SYSCALL `bevy_call_func, [%pent, `get_position, `World]
    %pposx: RealF = LOAD %ppos, `x
    %pposy: RealF = LOAD %ppos, `y
    %pposz: RealF = LOAD %ppos, `z

    %pos = SYSCALL `bevy_call_func, [%ent, `get_position, `World]
    %posx: RealF = LOAD %pos, `x
    %posy: RealF = LOAD %pos, `y
    %posz: RealF = LOAD %pos, `z

    %dx = SUBR %pposx, %posx
    %dx = DIVR %dx, 2
    %dy = SUBR %pposy, %posy
    %dy = ADDR %dy, 2.0     //
    %dy = DIVR %dy, 2
    %dz = SUBR %pposz, %posz
    %dz = DIVR %dz, 4

    //%pposx = ADDR %posx, %pposx
    //%pposy = ADDR %posy, %pposy
    //%pposz = ADDR %posz, %pposz
    //%pposy = ADDR %posy, %pposy
    //%ppos = NEW_STRUCT [`x: %pposx, `y: %pposy, `z: %pposz]
    //SYSCALL `bevy_call_func, [%ent, `set_velocity, %ppos]

    %vel = NEW_STRUCT [`x: %dx, `y: %dy, `z: %dz]
    SYSCALL `bevy_call_func, [%ent, `set_velocity, %vel]

    SYSCALL `delay, [0.5]

    RETURN nil
}

///////

    %module = NEW_STRUCT [
        `on_init: %on_init,
        `on_update: %on_update,
        `model: "models/smoodge.glb#Scene0",
        `grid_units: 8,
        //`grid_units: 16,
        `boom_mass: 10000.0,
        `block_mass: 2.5,
    ]
    RETURN %module
