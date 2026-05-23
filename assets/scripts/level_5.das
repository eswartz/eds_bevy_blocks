// Mandelbrot example, using one entity per grid square.

// This script generates worker entities to compute a single pixel
// of the Mandelbrot set.
%on_init = NEW_BLOCK `init, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, CntF
    %step: CntF = LOAD %ctx, `tick_count, true

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 4.0
    // World grid size (m)
    %gridsz = COPY 8.0

    %gridsz_h = DIV %gridsz, 2.0

    // Pixel grid unit count;
    // this squared is # of pixels and new entities!
    //%grid_units = COPY 256
    %grid_units = COPY 192
    //%grid_units = COPY 64
    //%grid_units = COPY 512
    //%grid_units = COPY 128

    // Default computation center
    %cx = COPY 0.0
    %cy = COPY 0.0
    %sz = DIV 4.0, 1.0

    // Zoom to specific spot
    %cx = COPY 0.390018071300935778312029
    %cx = COPY 0.390014071300505778312029
    %cy = COPY 0.138320184041392651089585
    %sz = DIV 1.0, 1024.0
    //%sz = COPY 1.0
    %sz = DIV %sz, 1024.0
    %sz = DIV %sz, 1048576.0
    //%sz = DIV 1.0, 2.0

    %max_iters = COPY 1500

    %cell_sz = DIV %sz, %grid_units

    // Create an entity to display the image.
    %shape = NEW_SEQUENCE [`Plane, x, %gridsz, y, %gridsz]
    %info =  NEW_SEQUENCE [`subdivisions, %grid_units]
    %shape = NEW_STRUCT [`kind, %shape, `info, %info]

    %pos = NEW_STRUCT [x: 0.0, y: %gridsz_h, z: 5.0]

    %rot = NEW_STRUCT [x: -90.0]
    %rot = NEW_SEQUENCE [`Deg, %rot]

    %imgsz = NEW_SEQUENCE [`x, %grid_units, `y, %grid_units]
    %img_data = NEW_SEQUENCE [`size, %imgsz, `format, `Rgba8Unorm]
    %img = CALL_INTRINSIC `bevy_call_func, [`new_image, %img_data]

    //%color = NEW_SEQUENCE [`Srgba, `red, 1, `green, 1,  `blue, 1]

    %img_ent = CALL_INTRINSIC `bevy_spawn, [
        `Name, "Mandelbrot",

        `texture, %img,

        `offset, %pos,

        `unlit, true,
        `double_sided, true,
        //`color, %color,

        `shape, %shape,
        `rotation, %rot,

        `CrosshairTargetable, nil,

        `rigid_body, `Static,
    ]

    // Spawn all the grid points, initializing them with
    // the coordinates to render. These are POD entities.

    %grid_units_h = LSHR %grid_units, 1
    %y: IntF = SUB 0, %grid_units_h

y_loop:
    %cmp = LESS_SINT %y, %grid_units_h
    IF_TRUE %cmp, y_step, exit

y_step:
    %x: IntF = SUB 0, %grid_units_h

x_loop:
    %cmp = LESS_SINT %x, %grid_units_h
    IF_TRUE %cmp, x_step, y_next

x_step:
    %name = CALL_INTRINSIC `to_string, ["(", %x, ", ", %y, ")"]

    %worker = CALL_INTRINSIC `bevy_spawn, [`Name, %name]

    // State for cell.
    %cr: RealF = MUL %x, %cell_sz
    %cr: RealF = ADD %cr, %cx
    %ci: RealF = MUL %y, %cell_sz
    %ci: RealF = ADD %ci, %cy

    %px = ADD %x, %grid_units_h
    %py = ADD %y, %grid_units_h
    %coord = NEW_STRUCT [
        `x: %px,
        `y: %py,
    ]

    //CALL_INTRINSIC `print, [%coord, "\n"]
    // Gather parameters for the worker script.
    %data = NEW_STRUCT [
        `cr: %cr,
        `ci: %ci,
        `zr: 0.0,
        `zi: 0.0,
        `max_iters: %max_iters,
        `iters: 0,

        // target image
        `img: %img,
        // pixel (x,y) to fill
        `coord: %coord,
    ]

    // Package ScriptCreationInfo data.
    %async = NEW_SEQUENCE [`Async]
    %sci = NEW_STRUCT [
        path: `scripts/mandel_img.das,
        mode: %async,
    ]
    CALL_INTRINSIC `bevy_call_func, [%worker, `add_script, %sci, %data]
    //CONSUME_FUEL 1    // FIXME: stalls script

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
    ]
    CONSUME_FUEL 3
    RETURN %module
