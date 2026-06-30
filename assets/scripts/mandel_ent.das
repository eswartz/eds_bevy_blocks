// This computes a given coordinate in the Mandelbrot set.

%on_init = NEW_BLOCK `on_init, (ctx: Struct, ent: Cnt64, data: Struct, ...) {

    %ent = ARG 1
    %data = ARG 2

    // // HACK: try to avoid having them all pile up at start
    //%v = MODU %ent, 100
    //%v: RealF = CAST %v, `coerce
    //%v = DIVR %v, 10.0
    //SYSCALL `delay, [%v]

    //SYSCALL `delay, [1.0]
    SYSCALL `bevy_call_func, [%ent, `set_palette_material]
    //SYSCALL `print, ["New block: ", %ent, "\n"]

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    STORE %data, `player, %pent

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent = ARG 1
    %data = ARG 2

    //SYSCALL `print, ["Update: ", %ent, "\n"]
    //    `cr: %x,
    //    `ci: %y,
    //    `zr: %x,
    //    `zi: %y,
    //    `max_iters: 1000,
    //    `iters: 0,

    // center
    %cr: RealF = LOAD %data, `cr
    %ci: RealF = LOAD %data, `ci

    // stop limit
    %max_iters: CntF = LOAD %data, `max_iters
    %range_min: CntF = LOAD %data, `range_min, true
    %range_size: CntF = LOAD %data, `range_size, true

    // current count; 0 means not started,
    // > 0 means still going;
    // < 0 means completed at -iters.
    %iters_signed: IntF = LOAD %data, `iters
    %iters: CntF = CAST %iters_signed, `coerce

    // < 0 means finished @ -iters
    %t = LESS_I %iters_signed, 0
    IF_TRUE %t, skip, check_high

check_high:
    // stop if already past limit
    %t = LESS_U %iters, %max_iters
    IF_TRUE %t, enter, skip

enter:
    %zr: RealF = LOAD %data, `zr
    %zi: RealF = LOAD %data, `zi

loop:
    %t = LESS_U %iters, %max_iters
    IF_TRUE %t, calc, past_max

calc:
    %iters: CntF = ADDU %iters, 1

    // advance z = z²+c
    // (a+bi) * (c+di) = ac - bd + (ad+bc)i
    // (zr+zi) * (zr+zi) = zr² - zi² + (2*zr*zi)

    %zr2: RealF = MULR %zr, %zr
    %zi2: RealF = MULR %zi, %zi
    %zrzi: RealF = MULR %zr, %zi

    %newzr: RealF = SUBR %zr2, %zi2
    %newzi: RealF = MULR %zrzi, 2.0
    %zr: RealF = ADDR %newzr, %cr
    %zi: RealF = ADDR %newzi, %ci

    // check limit (|z| < 2)
    %magr: RealF = MULR %zr, %zr
    %magi: RealF = MULR %zi, %zi
    %mag: RealF = ADDR %magr, %magi
    %t = LESS_R %mag, 4.0

    //SYSCALL `print, [%cr, "+", %ci, "i #", %iters, " = ", %zr, "+", %zi, "i\n"]
    IF_TRUE %t, loop, exit

exit:
    // finally finished here
    %iters_base: RealF = SUB %iters, %range_min
    %depth: RealF = DIV %iters, %range_size

    %depth_size: RealF = LOAD %data, depth
    %depth: RealF = MULR %depth, %depth_size

    // signed -> unsigned
    %iters = SUBU 0, %iters
    JUMP done

past_max:
    // too many iters
    %iters_base: RealF = SUB %max_iters, %range_min
    %depth: RealF = DIV %iters, %range_size

    %depth_size: RealF = LOAD %data, depth
    %depth: RealF = MULR %depth, %depth_size

done:
    //SYSCALL `delay, [0.05]

    %pos = LOAD %data, `pos
    %posx = LOAD %pos, `x
    %posy = LOAD %pos, `y
    %posz = LOAD %pos, `z
    %posz = ADD %posz, %depth
    %vec = NEW_STRUCT [x: %posx, y: %posy, z: %posz]
    SYSCALL `bevy_call_func, [%ent, `set_position, %vec]

    //%piters = SUBI 0, %iters
    //SYSCALL `print, [%cr, "+", %ci, "i out at ", %piters, "\n"]
    SYSCALL `bevy_call_func, [%ent, `set_mesh_tag, %iters]

    STORE %data, `iters, %iters
    STORE %data, `zr, %zr
    STORE %data, `zi, %zi

//    JUMP skip

    // Unload!
    %type = LOAD %ctx, `type_id, false
    SYSCALL `bevy_call_func, [%ent, `remove_script, %type]
//    SYSCALL `print, [%ent, "👋"]

skip:
    // Always face the viewer.
    %pent = LOAD %data, `player

//    %ppos = SYSCALL `bevy_call_func, [%pent, `get_position]
//    //SYSCALL `print, [%ppos, "\n"]

//    //%plook = SYSCALL `bevy_call_func, [%ent, `get_player_look]
//    //SYSCALL `bevy_call_func, [%ent, `set_rotation, %plook]

    %la = SYSCALL `bevy_call_func, [%ent, `look_at, %pent, 0.5]

    SYSCALL `delay, [0.05]
//    //SYSCALL `print, [%pent, " @", %ppos, "=", %la, "\n"]

    RETURN nil
}

%init_data = NEW_STRUCT [

]

%module = NEW_STRUCT [
    `on_init: %on_init,
    `on_update: %on_update,
    `init_data: %init_data,
]
RETURN %module
