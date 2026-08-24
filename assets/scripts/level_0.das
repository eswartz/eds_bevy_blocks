/// The plastic green field, "Please Don't Disturb",
/// vast arena of dead architecture...
%make_on_update = NEW_BLOCK `make_on_update, () {
    LOCAL cnt, false, CntF

    %update_cb = NEW_BLOCK `update_cb, (ctx: Struct, ent: Cnt64, ...) {
        %ctx = ARG 0
        %ent = ARG 1
        %step: CntF = LOAD %ctx, `tick_count, true

//        SYSCALL `print, ["<", %step]

        %t: CntF = CAST %ent, `coerce
        //%tns: CntF = LSHR %t, 32
        //%tns2: CntF = ADD %t, %tns
        //%tn: CntF = MOD %tns2, 0xa00
        %tn: CntF = MOD %t, 0xa00
        %tnc: CntF = ADD %tn, 0x1f400
        %ts = SYSCALL char_to_string, [%tnc]

        %task = GET_SYSTEM task

        %outer = GET_FRAME 1
        %cnt = LOAD %outer, cnt
        %new = ADD %cnt, 1
        STORE %outer, cnt, %new

        //SYSCALL spin, [50]
        //SYSCALL print, [%ts]             // uncomment for fun
        //SYSCALL print, [%ts]

        //SYSCALL spin, [10]
        _ = DIV 0, 0
//        SYSCALL print, [%ts]

        //SYSCALL `print, []
        RETURN nil
    }

    %frame = GET_FRAME
    STORE %frame, cnt, 0

    %clos = CLOSURE %update_cb
    RETURN %clos
}

///////

%on_update = CALL %make_on_update, []

%on_init = NEW_BLOCK `init, (ctx: Struct, ent: Cnt64, ...) {
    %ent = ARG 1

    %scene_offs = [x: 20.0, y: -0.5, z: 5.0]
    %scene_offs = [x: 15.0, y: -0.5, z: -20.0]

    %scene_rot = [x: 0.0, y: 0.0, z: 0.0]
    %scene_rot: Tuple = [
        `Degrees, %scene_rot,
    ]
    %scene = [
        //`Model, "maps/school_gym.glb#Scene0",
        `Model, "maps/classroom.glb#Scene0",
    ]

    SYSCALL `bevy_spawn, [
        //`parent, %ent,
        `shape, %scene,
        `offset, %scene_offs,
       // `rot, %scene_rot,
    ]

    RETURN nil
}

    %boom = [
        mass: 250.0,
        x: 0.125,
        y: 0.125,
        shape: `Sphere,
    ]

    %module = [
        boom: %boom,
        on_init: %on_init,

        block_size: 0.5,
        half_side_length: 3,
        on_update: %on_update,
        block_mass: 5.0,
        static: false,
    ]
    RETURN %module
