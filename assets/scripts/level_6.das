%on_init = NEW_BLOCK `init, (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent: CntF = ARG 1
    %data: Struct = ARG 2

    %entstr = SYSCALL `bevy_call_func, [`ent_to_string, %ent]
    SYSCALL `print, ["SMOODGE: ", %entstr, '\n']

    %pos = NEW_STRUCT [`x: 0, `y: 2, `z: 0]

    %mesh_mat = NEW_STRUCT [
        `mesh: "models/smoodge.glb#Mesh0/Primitive0",
        `material: "models/smoodge.glb#Material0",
    ]
    %model = NEW_SEQUENCE [
        `MeshMaterial, %mesh_mat,
        //`Model, "models/smoodge.glb#Scene0",
    ]
    %collider_shape = NEW_SEQUENCE [`Cube, x, 1.0, y, 1.0, z, 1.0]
    %top_ent = SYSCALL `bevy_spawn, [
        `parent, %ent,
        `Name, "WATCH",

        `offset, %pos,

        `shape, %model,

        `CrosshairTargetable, nil,
        `Visibility, `Inherited,

        `rigid_body, `Dynamic,
        `collider_shape, %collider_shape,
    ]

    STORE %data, `tick, 0
    STORE %data, `smoodge, %top_ent

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 5.0
    %gridcz = COPY 1.0

    // World grid size
    %gridsz = COPY 4.0

    %unitsz_h = COPY 0.25
    %unitsz = COPY 1.0

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

    %ccr = MULR %wx, 555.555
    %ccg = MULR %wy, 3333.3333
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

    %mass: RealF = LOAD %data, `block_mass

    %stone = NEW_SEQUENCE [`Model, "models/stone.glb#Scene0"]
    %sphere_shape = NEW_SEQUENCE [`Sphere, 1.0]
    %sphere_collider_shape = NEW_SEQUENCE [`Sphere, 0.6]

    %rx: RealF = CAST %x, `coerce
    %rx = SINR %rx
    %rx = MULR %rx, 3.33333
    %ry: RealF = CAST %y, `coerce
    %ry = COSR %ry
    %ry = MULR %ry, 2.22222

    %rot = NEW_SEQUENCE [`Rad, %rx, %ry, 0.0]
    //SYSCALL `print, [%rot, '\n']

    %sx: RealF = CAST %ccg, `coerce
    %sx = ADDR %sx, 2.0
    %sx = MULR %sx, 0.5
    %sx = MAXR %sx, 0.5
    %sx = MINR %sx, 1.5

    %sx = MULR %sx, 0.1     // if not parented
    %scale = NEW_STRUCT [`x, %sx, `y, %sx, `z, %sx]
    //SYSCALL `print, [%scale, '\n']
    //SYSCALL `print, [%color_enum, '\n']

    %density = NEW_SEQUENCE [100.0]
    %new = SYSCALL `bevy_spawn, [
      //  `parent, %ent,
        `scriptable, true,

        `offset, %pos,
        `rotation, %rot,
        `scale, %scale,

        `color, %color_enum,
        `color_texture, "textures/bubble.webp",
        `normal_texture, "textures/bubble_normal.webp",
        //`alpha_mode, `Add,
        `emissive, 0.0,
        `roughness, 0.0,
        //`diffuse_transmission, 0.5,
        //`specular_transmission, 0.5,
        //`thickness, 0.5,
        //`cull_face, `None,

        //`shape, %stone,
        `shape, %sphere_shape,
        `collider_shape, %sphere_collider_shape,
        `rigid_body, `Dynamic,
        `mass, %mass,
        `friction, 1.0,
        `AngularDamping, 0.5,
        //`ColliderDensity, %density,
        //`CollisionMargin, 0.01,

        // `CrosshairTargetable, nil,
        `Name, %name,
        `Visibility, `Inherited,
    ]

    // State for cell.
    %num: IntF = CAST %ent, `coerce
    //%num = ADDI %x, %y
    %num = ABSI %num
    %cell_data = NEW_STRUCT [
        `pos: %pos,
        `parent: %top_ent,
        `num: %num,
    ]

    // Package ScriptCreationInfo data.
    %sci = NEW_STRUCT [
        path: `scripts/automaton.das,
        mode: `Async,
    ]
    SYSCALL `bevy_call_func, [%new, `add_script, %sci, %cell_data]

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

%on_update = NEW_BLOCK `update, (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent: CntF = ARG 1
    %data = ARG 2

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    %ppos = SYSCALL `bevy_call_func, [%pent, `get_position, `World]

    %pposx: RealF = LOAD %ppos, `x
    %pposy: RealF = LOAD %ppos, `y

    %dpy: CntF = LOAD %ctx, `tick_count, true
    //%dpy = MODU %dpy, 256
    //%dpy: IntF = SUBI %dpy, 64
    %dpy: RealF = CAST %dpy, `coerce
    %dpy = MULR %dpy, 0.01
    %dpy = SINR %dpy
    %dpy = MULR %dpy, 2.0

    %dpy = ADDR %dpy, 2.5

    %pposy = ADDR %pposy, %dpy
    %pposz: RealF = LOAD %ppos, `z

    %top_ent = LOAD %data, `smoodge
    %pos = SYSCALL `bevy_call_func, [%top_ent, `get_position, `World]
    IF_NIL %pos, ignore, update
update:
    //SYSCALL `print, [%ent, " ", %pos, '\n']

    //%entstr = SYSCALL `bevy_call_func, [`ent_to_string, %ent]
    //SYSCALL `print, ["MAIN: ", %entstr, '\n']

    %posx: RealF = LOAD %pos, `x
    %posy: RealF = LOAD %pos, `y
    %posz: RealF = LOAD %pos, `z

    %dx = SUBR %pposx, %posx
    %dx = DIVR %dx, 1.5
    %dy = SUBR %pposy, %posy
    %dy = DIVR %dy, 2

    %dy = ADDR %dy, 0.5     // offset

    %dz = SUBR %pposz, %posz
    %dz = DIVR %dz, 1.25

    %old = SYSCALL `bevy_call_func, [%top_ent, `look_at, %pent, 0.5]

    %vel = NEW_STRUCT [`x: %dx, `y: %dy, `z: %dz]
    //SYSCALL `print, [%vel, '\n']
    SYSCALL `bevy_call_func, [%top_ent, `set_velocity, %vel]
    //SYSCALL `print, [%old, '\n']
ignore:

    // This is a single entity. No sleep is fine. Look at automaton for all the balls!
    //SYSCALL `delay, [0.05]

    RETURN nil
}

///////

    %init_data = NEW_STRUCT [
        //`model: "models/smoodge.glb#Scene0",
        //`grid_units: 4,
        `grid_units: 20,
        //`grid_units: 1,
        `boom_mass: 5000.0,
        `block_mass: 0.5,
    ]
    %module = NEW_STRUCT [
        `on_init: %on_init,
        `on_update: %on_update,
        `init_data: %init_data,
    ]
    RETURN %module
