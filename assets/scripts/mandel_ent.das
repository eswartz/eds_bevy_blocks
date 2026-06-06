// This computes a given coordinate in the Mandelbrot set.

%on_init = NEW_BLOCK `on_init, (ctx, ent, data, ...) {

    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct
    //SYSCALL `delay, [1.0]
    SYSCALL `bevy_call_func, [%ent, `set_palette_material]
    //SYSCALL `print, ["New block: ", %ent, "\n"]

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    STORE %data, `player, %pent

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

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

    // current count; 0 means not started,
    // > 0 means still going;
    // < 0 means completed at -iters.
    %iters: IntF = LOAD %data, `iters

    // < 0 means finished @ -iters
    %t = LESS_SINT %iters, 0
    IF_TRUE %t, skip, check_high

check_high:
    // stop if already past limit
    %t = LESS_SINT %iters, %max_iters
    IF_TRUE %t, enter, skip

enter:
    %zr: RealF = LOAD %data, `zr
    %zi: RealF = LOAD %data, `zi

loop:
    %t = LESS_UINT %iters, %max_iters
    IF_TRUE %t, calc, past_max

calc:
    %iters: CntF = ADD %iters, 1

    // advance z = z²+c
    // (a+bi) * (c+di) = ac - bd + (ad+bc)i
    // (zr+zi) * (zr+zi) = zr² - zi² + (2*zr*zi)

    %zr2: RealF = MUL %zr, %zr
    %zi2: RealF = MUL %zi, %zi
    %zrzi: RealF = MUL %zr, %zi

    %newzr: RealF = SUB %zr2, %zi2
    %newzi: RealF = MUL %zrzi, 2.0
    %zr: RealF = ADD %newzr, %cr
    %zi: RealF = ADD %newzi, %ci

    // check limit (|z| < 2)
    %magr: RealF = MUL %zr, %zr
    %magi: RealF = MUL %zi, %zi
    %mag: RealF = ADD %magr, %magi
    %t = LESS_REAL %mag, 4.0

    //SYSCALL `print, [%cr, "+", %ci, "i #", %iters, " = ", %zr, "+", %zi, "i\n"]
    IF_TRUE %t, loop, exit

exit:
    // finally finished here
    %depth: RealF = DIV %iters, %max_iters

    %iters = NEG %iters
    JUMP done

past_max:
    // too many iters
    %depth = COPY 1.0

done:
    //SYSCALL `delay, [0.05]

    %pos = LOAD %data, `pos
    %posx = LOAD %pos, `x
    %posy = LOAD %pos, `y
    %posz = LOAD %pos, `z
    %posz = ADD %posz, %depth
    %vec = NEW_STRUCT [x: %posx, y: %posy, z: %posz]
    SYSCALL `bevy_call_func, [%ent, `set_position, %vec]

    //SYSCALL `print, [%cr, "+", %ci, "i out at ", %iters, "\n"]
    SYSCALL `bevy_call_func, [%ent, `set_mesh_tag, %iters]

    STORE %data, `iters, %iters
    STORE %data, `zr, %zr
    STORE %data, `zi, %zi

    // // Unload!
    //%type = LOAD %ctx, `type_id, false
    //SYSCALL `bevy_call_func, [%ent, `remove_script, %type]

skip:
    // Always face the viewer.
    %pent = LOAD %data, `player

//    %ppos = SYSCALL `bevy_call_func, [%pent, `get_position]
//    //SYSCALL `print, [%ppos, "\n"]

//    //%plook = SYSCALL `bevy_call_func, [%ent, `get_player_look]
//    //SYSCALL `bevy_call_func, [%ent, `set_rotation, %plook]

    %la = SYSCALL `bevy_call_func, [%ent, `look_at, %pent, 0.5]
//    //SYSCALL `print, [%pent, " @", %ppos, "=", %la, "\n"]

    RETURN nil
}

%module = NEW_STRUCT [
    `on_init: %on_init,
    `on_update: %on_update,
]
RETURN %module
