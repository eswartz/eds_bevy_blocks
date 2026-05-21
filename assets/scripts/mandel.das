// This computes a given coordinate in the Mandelbrot set.

%on_update = NEW_BLOCK `on_update, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    //
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
    CONSUME_FUEL 1

    %zr: RealF = LOAD %data, `zr
    %zi: RealF = LOAD %data, `zi

loop:
    %t = LESS_UINT %iters, %max_iters
    IF_TRUE %t, calc, past_max

calc:
    %iters: CntF = ADD %iters, 1
    //CONSUME_FUEL 1

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

    //CALL_INTRINSIC `print, [%cr, "+", %ci, "i #", %iters, " = ", %zr, "+", %zi, "i\n"]
    IF_TRUE %t, loop, exit

exit:
    // finally finished here
    %iters = NEG %iters
    JUMP done

past_max:
    // too many iters

done:
    %itersz: RealF = DIV %iters, %max_iters
    %itersz: RealF = ABS %itersz
    %vec = NEW_STRUCT [x: 0, y: 0, z: %itersz]
    CALL_INTRINSIC `bevy_call_func, [%ent, `translate, %vec]

    //CALL_INTRINSIC `print, [%cr, "+", %ci, "i out at ", %iters, "\n"]
    STORE %data, `iters, %iters
    STORE %data, `zr, %zr
    STORE %data, `zi, %zi

skip:
    RETURN nil
}

%module = NEW_STRUCT [
    `on_update: %on_update,
]
RETURN %module
