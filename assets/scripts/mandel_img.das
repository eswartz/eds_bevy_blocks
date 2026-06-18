// This computes a given coordinate in the Mandelbrot set
// by taking an available entry from a `worklist.

%on_update = NEW_BLOCK `on_update, (ctx, ent, ent_data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %ent_data = ARG 2, false, Struct

    // see if we have any work
    %work = LOAD %ent_data, `worklist

work_loop:
    %data = SYSCALL `sequence_pop, [%work]
    IF_NIL %data, skip, work

work:
    //SYSCALL `print, [%ent, "> ", %data, "\n"]
    // center
    %cr: RealF = LOAD %data, `cr
    %ci: RealF = LOAD %data, `ci

    // stop limit
    %max_iters: CntF = LOAD %data, `max_iters

    // current count; 0 means not started,
    // > 0 means still going;
    // < 0 means completed at -iters.
    %iters_s: IntF = LOAD %data, `iters
    %iters: CntF = CAST %iters_s, `coerce

    // < 0 means finished @ -iters
    %t = LESS_I %iters_s, 0
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
    %itersr: RealF = CAST %iters, `convert
    %max_itersr: RealF = CAST %max_iters, `convert
    %depth: RealF = DIVR %itersr, %max_itersr

    %iters = SUBU 0, %iters
    JUMP done

past_max:
    // too many iters
    %depth = COPY 0.0

done:
    %coord = LOAD %data, `coord
    %px = LOAD %coord, `x
    %py = LOAD %coord, `y

    //SYSCALL `print, [%px, "x", %py, %iters, " @", %depth, "\n"]

    %c: RealF = MULR %depth, 16.0
    %c = MODR %c, 1.0
    //%color_enum = NEW_SEQUENCE [`Hsva, `hue, 0, `saturation, 0, `value, %c, `alpha, 1]
    %color_enum = NEW_SEQUENCE [`Srgba, `red, %c, `green, %c, `blue, %c]

    %img_handle = LOAD %data, `img

    %img = NEW_SEQUENCE [`Handle, %img_handle]
    SYSCALL `bevy_call_func, [
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
    //`on_init: %on_init,
    `on_update: %on_update,
]
RETURN %module
