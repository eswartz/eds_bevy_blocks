// Mandelbrot example, using one entity per grid square.

// This script generates worker entities to compute a single pixel
// of the Mandelbrot set.
%on_init = (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent = ARG 1
    %data = ARG 2

    // World grid position
    %gridcx = COPY 0.0
    %gridcy = COPY 4.0
    // World grid size (m)
    %gridsz = COPY 8.0

    %gridsz_h = DIV %gridsz, 2.0

    %num_workers: CntF = LOAD %data, `num_workers

    // Pixel grid unit count
    %grid_units: CntF = LOAD %data, `grid_units

    %cx: RealF = LOAD %data, `cx
    %cy: RealF = LOAD %data, `cy
    %sz: RealF = LOAD %data, `sz
    %max_iters: CntF = LOAD %data, `max_iters

    %cell_sz = DIV %sz, %grid_units

    // Create an entity to display the image.
    %shape = [`Cube, x, %gridsz, z, %gridsz, y, 0.01]
    %info =  []
    //%shape = [`Plane, x, %gridsz, y, %gridsz]
    //%info =  [`subdivisions, 8]
    //%shape: Struct = [`kind, %shape, `info, %info]

    %y = ADD %gridsz_h, 0.025
    %pos = [x: 0.0, y: %y, z: 5.0]

    %rot = [x: -90.0]
    %rot: Tuple = [`Deg, %rot]

    %imgsz = [`x, %grid_units, `y, %grid_units]
    %imgp = [`filter, `Nearest]
    %img_data = [`size, %imgsz, `format, `Rgba8Unorm, `params, %imgp]
    %img = SYSCALL `bevy_call_func, [`new_image, %img_data]

    %color = [`Srgba, `red, 2, `green, 2,  `blue, 2]

    // Spawn the billboard showing the image of the render.
    %img_ent = SYSCALL `bevy_call_func, [`spawn_entity,
        `parent, %ent,
        `Name, "Mandelbrot",

        `texture, %img,

        `offset, %pos,

        //`unlit, true,
        //`emissive, 0.05,
        //`double_sided, true,
        `cull_face, `None,
        `color, %color,

        `shape, %shape,
        `rotation, %rot,

        `CrosshairTargetable, nil,

        `rigid_body, `Static,
    ]

    // Compute the work.
    %worklist = []

    %grid_units_h = LSHR %grid_units, 1
    %y: IntF = SUB 0, %grid_units_h

y_loop:
    %cmp = LESS_I %y, %grid_units_h
    IF_TRUE %cmp, y_step, cells_done

y_step:
    %x: IntF = SUB 0, %grid_units_h

x_loop:
    %cmp = LESS_I %x, %grid_units_h
    IF_TRUE %cmp, x_step, y_next

x_step:
    // State for cell.
    %cellx = COPY %x
    %celly = COPY %y
    %cr: RealF = MUL %cellx, %cell_sz
    %cr: RealF = ADD %cr, %cx
    %ci: RealF = MUL %celly, %cell_sz
    %ci: RealF = ADD %ci, %cy

    %px = ADD %cellx, %grid_units_h
    %py = ADD %celly, %grid_units_h
    %coord = [
        `x: %px,
        `y: %py,
    ]

    // Parameters for the cell.
    %cell_data = [
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

    SYSCALL `sequence_append, [%worklist, %cell_data]
    %cell_data = COPY nil

x_next:
    %x = ADD %x, 1
    JUMP x_loop

y_next:
    %y = ADD %y, 1
    JUMP y_loop

cells_done:

    %i: CntF = COPY 0

make_worker_loop:
    %f = LESS_U %i, %num_workers
    IF_TRUE %f, make_worker, exit

make_worker:
    // Package ScriptCreationInfo data.
    %worker = SYSCALL `bevy_call_func, [`spawn_entity,
        `parent, %ent,
        `Name, "WORKER",
    ]

    %mode = [`Async]
    %sci = [
        path: `scripts/mandel_img.das,
        mode: %mode,
    ]
    %cell_data = [
        worklist: %worklist,
    ]
    SYSCALL `bevy_call_func, [%worker, `add_script, %sci, %cell_data]

    %i: CntF = ADD %i, 1
    JUMP make_worker_loop

exit:
    RETURN nil
}

///////

%sz = DIV 1.0, 1024.0
%sz = DIV %sz, 1048576.0
%sz = DIV %sz, 1024.0

%init_data = [
    // number of entities to chug through data.
    //`num_workers: 1024,
    //`num_workers: 128,
    `num_workers: 256,

    // Pixel grid unit count
    `grid_units: 64,
    `grid_units: 256,
    //`grid_units: 192,
    //`grid_units: 128,

    `cx: 0.390018071300935778312029,
    `cy: 0.138320184041392651089585,
    //`cx: -0.743_643_900_055,
    //`cy: 0.131_828_895_901,

    `sz: %sz,

    `max_iters: 2500,
]

%module = [
    `on_init: %on_init,

    `boom_mass: 500.0,

    `init_data: %init_data,
]
RETURN %module
