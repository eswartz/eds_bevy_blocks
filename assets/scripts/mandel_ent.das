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
    //SYSCALL `bevy_call_func, [%ent, `set_palette_material]
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

    %depth_size: RealF = LOAD %data, depth_size

    // current count; 0 means not started,
    // > 0 means still going;
    // < 0 means completed at -iters.
    %iters_signed: IntF = LOAD %data, `iters
    %iters: CntF = CAST %iters_signed, `coerce

    // < 0 means finished @ -iters
    %t = LESS_I %iters_signed, 0
    IF_TRUE %t, face_player, check_high

check_high:
    // stop if already past limit
    %t = LESS_U %iters, %max_iters
    IF_TRUE %t, enter, face_player

enter:
    %zr: RealF = LOAD %data, `zr
    %zi: RealF = LOAD %data, `zi

loop:
    %t = LESS_U %iters, %max_iters
    IF_TRUE %t, calc, past_max

calc:
    %iters: CntF = ADDU %iters, 1
//    %off = MODU %iters, 256
//    IF_ZERO %off, wait_a_bit, do_calc
//
//wait_a_bit:
//`    SYSCALL `delay, [0.0]

do_calc:
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
    %done = COPY true

    JUMP done

past_max:
    // too many iters
    %done = COPY false

done:
    %iters_base: RealF = SUB %iters, %range_min
    %iters_size = SUB %max_iters, %range_min

    %iters_pos: RealF = DIV %iters, %iters_size
    %depth: RealF = DIV %iters, %iters_size

    %depth: RealF = MULR %depth, %depth_size

    //SYSCALL `delay, [0.05]

    %pos = LOAD %data, `pos
    %posx = LOAD %pos, `x
    %posy = LOAD %pos, `y
    %posz = LOAD %pos, `z
    %posz = ADD %posz, %depth
    %vec = NEW_STRUCT [x: %posx, y: %posy, z: %posz]
    //%parent = LOAD %data, `parent
    //%coord = NEW_SEQUENCE [`RelativeTo, %parent]
    //SYSCALL `bevy_call_func, [%ent, `set_position, %vec, %coord]
    SYSCALL `bevy_call_func, [%ent, `set_position, %vec]

    //SYSCALL `print, [%cr, "+", %ci, "i out at ", %piters, "\n"]
    SYSCALL `bevy_call_func, [%ent, `set_mesh_tag, %iters]

    IF_TRUE %done, negate_iters, store_iters
negate_iters:
    // signed -> unsigned
    %iters = SUBU 0, %iters

store_iters:
    STORE %data, `iters, %iters
    STORE %data, `zr, %zr
    STORE %data, `zi, %zi

    // Give some time before watching user.
    %look_at = LOAD %data, look_at_player
    IF_TRUE %look_at, delay_before_look_at, unload

delay_before_look_at:
    %delay: RealF = LOAD %data, `delay_before_look_at, true
    SYSCALL `delay, [%delay]
    JUMP face_player

unload:
    %type = LOAD %ctx, `type_id, false
    SYSCALL `bevy_call_func, [%ent, `remove_script, %type]
//    SYSCALL `print, [%ent, "👋"]

    // STOP

face_player:
    // Always face the viewer.
    %pent = LOAD %data, `player

    %delay: RealF = LOAD %data, `look_at_player_delay_secs, true
    SYSCALL `delay, [%delay]

    %la = SYSCALL `bevy_call_func, [%ent, `look_at, %pent, 1.0]

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
