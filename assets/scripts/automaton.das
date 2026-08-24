%on_init = NEW_BLOCK `on_init, (ctx: Struct, ent: Cnt64, data: Struct, ...) {

    %ent = ARG 1
    %data = ARG 2

    %pent = SYSCALL `bevy_call_func, [%ent, `get_player_entity]
    STORE %data, `player, %pent
    STORE %data, `speed, 4.0
    STORE %data, `hadjust, 2.0
    STORE %data, `vadjust, 1.0

    RETURN nil
}

%on_update = NEW_BLOCK `on_update, (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent = ARG 1
    %data = ARG 2

    //%vec = [x: -0.1, y: 0.0, z: -0.1]

    %pos = SYSCALL `bevy_call_func, [%ent, `get_position, `World]
    //SYSCALL `print, ["from ", %pos, '\n']
    %posx: RealF = LOAD %pos, `x
    %posy: RealF = LOAD %pos, `y
    %posz: RealF = LOAD %pos, `z

    %pent: CntF = LOAD %data, `parent
    %goal_pos = SYSCALL `bevy_call_func, [%pent, `get_position, `World]
    //SYSCALL `print, ["to ", %goal_pos, '\n']
    %goal_posx: RealF = LOAD %goal_pos, `x
    %goal_posy: RealF = LOAD %goal_pos, `y
    %goal_posz: RealF = LOAD %goal_pos, `z

    // Directional distance from our goal.
    %dx = SUBR %goal_posx, %posx
    %dy = SUBR %goal_posy, %posy
    %dz = SUBR %goal_posz, %posz

    %hadjust: RealF = LOAD %data, `hadjust
    %vadjust: RealF = LOAD %data, `vadjust
    %dx = MULR %dx, %hadjust
    %dy = MULR %dy, %vadjust
    %dz = MULR %dz, %hadjust

    // FIXME CLAMP? or copysign? or oh utility libraries...
    //%dxa = ABSR %dx
    //%dxa = MINR %vda, 10.0
    //%c = LESSR %dx, 0.0

    //%curvel = SYSCALL `bevy_call_func, [%ent, `get_velocity]
    //%curvx: RealF = LOAD %curvel, `x
    //%curvy: RealF = LOAD %curvel, `y
    //%curvz: RealF = LOAD %curvel, `z

    //%vx = ADDR %curvx, %vx
    //%vy = ADDR %curvy, %vy
    //%vz = ADDR %curvz, %vz
    %vx = DIVR %dx, 4
    %vy = DIVR %dy, 2   // fight gravity and dip!
    %vz = DIVR %dz, 4

    // No hurry when far, but zips when close!
    //%vx = DIVR 2, %dx
    //%vy = DIVR 2, %dy
    //%vz = DIVR 2, %dz

    // Clamp to the maximum allowed speed.
    %speed: RealF = LOAD %data, `speed

    %vx = MINR %vx, %speed
    %vy = MINR %vy, %speed
    %vz = MINR %vz, %speed
    // ... in both directions. Maybe CLAMP...
    %rspeed = NEGR %speed
    %vx = MAXR %vx, %rspeed
    %vy = MAXR %vy, -2.0
    %vz = MAXR %vz, %rspeed

    %vel = [x: %vx, y: %vy, z: %vz]

    //SYSCALL `print, [%vel, '\n']
    //SYSCALL `bevy_call_func, [%ent, `set_velocity, %vel]
    SYSCALL `bevy_call_func, [%ent, `insert_components,
        `LinearVelocity, %vel,
    ]

    //%d: RealF = CAST %ent, `coerce
    %d = LOAD %data, `num
    //SYSCALL `print, [%d, '\n']

    %d: RealF = CAST %d, `coerce
    %d = DIVR %d, 17.0
    //%d = MODR %d, 1.0
    %d = MODR %d, 0.5
    %d = ADDR %d, 0.05

    SYSCALL `delay, [%d]

    RETURN nil
}

%on_update_temp = NEW_BLOCK `on_update, (ctx: Struct, ent: Cnt64, data: Struct, ...) {
    %ctx = ARG 0
    %ent = ARG 1
    %data = ARG 2

    %vel = [x: 0, y: 0.1, z: 0]

    SYSCALL `bevy_call_func, [%ent, `insert_components,
        `LinearVelocity, %vel,
    ]

    SYSCALL `delay, [0.05]

    RETURN nil
}


%module = [
    `on_init: %on_init,
    `on_update: %on_update,
    //`on_update: %on_update_temp,
]
RETURN %module
