%on_init = NEW_BLOCK `on_init, (ctx, ent, data, ...) {

    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    STORE %data, `player, %pent

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    //%vec = NEW_STRUCT [x: -0.1, y: 0.0, z: -0.1]

    %pos = SYSCALL `bevy_call_func, [%ent, `get_position]
    %posx: RealF = LOAD %pos, `x
    %posy: RealF = LOAD %pos, `y
    %posz: RealF = LOAD %pos, `z
    %posx = DIVR %posx, -4.0
    %posy = DIVR %posy, -2.0
    %posz = DIVR %posz, -4.0
    %vec = NEW_STRUCT [x: %posx, y: %posy, z: %posz]

    //%pos = SYSCALL `bevy_call_func, [%ent, `translate, %vec]
    SYSCALL `bevy_call_func, [%ent, `add_velocity, %vec]

    SYSCALL `delay, [0.01]

    RETURN nil
}

%module = NEW_STRUCT [
    `on_init: %on_init,
    `on_update: %on_update,
]
RETURN %module
