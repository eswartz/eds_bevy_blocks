%on_init = NEW_BLOCK `on_init, (ctx, ent, data, ...) {

    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    STORE %data, `player, %pent
    STORE %data, `speed, 8.0

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    //%vec = NEW_STRUCT [x: -0.1, y: 0.0, z: -0.1]

    %curvel = SYSCALL `bevy_call_func, [%ent, `get_velocity]
    %curvx: RealF = LOAD %curvel, `x
    %curvy: RealF = LOAD %curvel, `y
    %curvz: RealF = LOAD %curvel, `z

    %pos = SYSCALL `bevy_call_func, [%ent, `get_position]
    %posx: RealF = LOAD %pos, `x
    %posy: RealF = LOAD %pos, `y
    %posz: RealF = LOAD %pos, `z
    %vx = DIVR %posx, -4.0
    %vy = DIVR %posy, -2.0
    %vz = DIVR %posz, -4.0

    // FIXME copysign/etc.
    //%vxa = ABSR %vx
    //%vxa = MINR %vxa, 10.0
    //%c = LESSR %vx, 0.0

    %speed: RealF = LOAD %data, `speed
    %vx = ADDR %curvx, %vx
    %vy = ADDR %curvy, %vy
    %vz = ADDR %curvz, %vz

    %vx = MINR %vx, %speed
    %vy = MINR %vy, %speed
    %vz = MINR %vz, %speed
    %rspeed = NEGR %speed
    %vx = MAXR %vx, %rspeed
    %vy = MAXR %vy, -1.0
    %vz = MAXR %vz, %rspeed

    %vel = NEW_STRUCT [x: %vx, y: %vy, z: %vz]

    //SYSCALL `print, [%vel, '\n']
    SYSCALL `bevy_call_func, [%ent, `set_velocity, %vel]

    SYSCALL `delay, [0.05]

    RETURN nil
}

%module = NEW_STRUCT [
    `on_init: %on_init,
    `on_update: %on_update,
]
RETURN %module
