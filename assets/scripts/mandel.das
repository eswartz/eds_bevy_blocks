// This computes a given coordinate in the Mandelbrot set.

%on_update = NEW_BLOCK `on_update, (ctx, ent, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64

    RETURN nil
}

%module = NEW_STRUCT [
    `on_update: %on_update,
]
RETURN %module
