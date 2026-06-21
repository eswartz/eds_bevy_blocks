%on_init = NEW_BLOCK `on_init, (ctx, ent, data, ...) {

    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    STORE %data, `player, %pent
    STORE %data, `speed, 4.0
    STORE %data, `hadjust, 2.0
    STORE %data, `vadjust, 1.0

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx, ent, data, ...) {
    %ctx = ARG 0, false, Struct
    %ent = ARG 1, false, Cnt64
    %data = ARG 2, false, Struct

    //%vec = NEW_STRUCT [x: -0.1, y: 0.0, z: -0.1]

    %pos = SYSCALL `bevy_call_func, [%ent, `get_position, `World]
    //SYSCALL `print, ["from ", %pos, '\n']
    %posx: RealF = LOAD %pos, `x
    %posy: RealF = LOAD %pos, `y
    %posz: RealF = LOAD %pos, `z

    %pent: CntF = LOAD %data, `parent
    %ppos = SYSCALL `bevy_call_func, [%pent, `get_position, `World]
    //SYSCALL `print, ["to ", %ppos, '\n']
    %pposx: RealF = LOAD %ppos, `x
    %pposy: RealF = LOAD %ppos, `y
    %pposz: RealF = LOAD %ppos, `z

    %vx = SUBR %pposx, %posx
    %vy = SUBR %pposy, %posy
    %vz = SUBR %pposz, %posz

    %hadjust: RealF = LOAD %data, `hadjust
    %vadjust: RealF = LOAD %data, `vadjust
    %vx = MULR %vx, %hadjust
    //%vx = ADDR %vx, 2.0
    %vy = MULR %vy, %vadjust
    %vz = MULR %vz, %hadjust
    //%vz = ADDR %vz, -2.0

    // FIXME copysign/etc.
    //%vxa = ABSR %vx
    //%vxa = MINR %vxa, 10.0
    //%c = LESSR %vx, 0.0

    //%curvel = SYSCALL `bevy_call_func, [%ent, `get_velocity]
    //%curvx: RealF = LOAD %curvel, `x
    //%curvy: RealF = LOAD %curvel, `y
    //%curvz: RealF = LOAD %curvel, `z

    %speed: RealF = LOAD %data, `speed
    //%vx = ADDR %curvx, %vx
    //%vy = ADDR %curvy, %vy
    //%vz = ADDR %curvz, %vz
    %vx = DIVR %vx, 2
    %vy = DIVR %vy, 2
    %vz = DIVR %vz, 2

    %vx = MINR %vx, %speed
    %vy = MINR %vy, %speed
    %vz = MINR %vz, %speed
    %rspeed = NEGR %speed
    %vx = MAXR %vx, %rspeed
    %vy = MAXR %vy, -2.0
    %vz = MAXR %vz, %rspeed

    %vel = NEW_STRUCT [x: %vx, y: %vy, z: %vz]

    //SYSCALL `print, [%vel, '\n']
    SYSCALL `bevy_call_func, [%ent, `set_velocity, %vel]

    //%d: RealF = CAST %ent, `coerce
    %d = LOAD %data, `num
    //SYSCALL `print, [%d, '\n']

    %d: RealF = CAST %d, `coerce
    %d = DIVR %d, 17.0
    %d = MODR %d, 1.0
    SYSCALL `delay, [%d]

    RETURN nil
}

%module = NEW_STRUCT [
    `on_init: %on_init,
    `on_update: %on_update,
]
RETURN %module
