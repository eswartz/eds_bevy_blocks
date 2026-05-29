// This computes a given coordinate in the Mandelbrot set
// by taking an available entry from a `worklist.

%on_init = NEW_BLOCK `on_init, (ctx, ent, ...) {

    %ent = ARG 1, false, Cnt64
    //CALL_INTRINSIC `delay, [1.0]
    CALL_INTRINSIC `bevy_call_func, [%ent, `set_palette_material]
    //CALL_INTRINSIC `print, ["New block: ", %ent, "\n"]

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx, ent, ent_data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %ent_data = ARG 2, false, Struct

    // see if we have any work
    %work = LOAD %ent_data, `worklist

work_loop:
    %data = CALL_INTRINSIC `sequence_pop, [%work]
    IF_NIL %data, skip, work

work:
    //CALL_INTRINSIC `print, [%ent, "> ", %data, "\n"]
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
    //%f = AND %iters, 0x3f
    //%f = EQUAL_UINT %f, 0
    //IF_TRUE %f, fuel, calc_do
//fuel:

//calc_do:
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
    %depth: RealF = DIV %iters, %max_iters

    %iters = NEG %iters
    JUMP done

past_max:
    // too many iters
    %depth = COPY 0.0

done:
    %coord = LOAD %data, `coord
    %px = LOAD %coord, `x
    %py = LOAD %coord, `y

    //CALL_INTRINSIC `print, [%px, "x", %py, %iters, " @", %depth, "\n"]

    %c: RealF = MUL %depth, 16.0
    %c: RealF = MOD %c, 1.0
    //%color_enum = NEW_SEQUENCE [`Hsva, `hue, 0, `saturation, 0, `value, %c, `alpha, 1]
    %color_enum = NEW_SEQUENCE [`Srgba, `red, %c, `green, %c, `blue, %c]

    %img_handle = LOAD %data, `img

    %img = NEW_SEQUENCE [`Handle, %img_handle]
    CALL_INTRINSIC `bevy_call_func, [
        %ent,
        set_image_pixel,
        %img_handle,
        %px,
        %py,
        %color_enum,
    ]

    STORE %data, `iters, %iters
    STORE %data, `zr, %zr
    STORE %data, `zi, %zi

    JUMP work_loop

skip:
    RETURN nil
}

%module = NEW_STRUCT [
    `on_init: %on_init,
    `on_update: %on_update,
]
RETURN %module
