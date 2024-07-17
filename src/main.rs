use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, Wireframe2dPlugin},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, exit_on_esc)
        .add_systems(Update, update_mouse_position)
        .add_systems(Update, update_camera_focus)
        // .add_systems(Update, toggle_wireframe)
        .add_systems(Update, update_kinematic)
        .add_systems(Update, update_movement)
        .add_systems(Update, update_decelerating)
        .add_systems(Update, (update_following, update_moving_to_dest))
        .add_systems(Update, update_moving_in_dir)
        .add_systems(Update, update_drag_symmetric)
        .add_systems(Update, sync_pos_transform)
        .add_systems(Update, update_player_controlled)
        .run();
}
//     commands.spawn(
//         TextBundle::from_section("Press space to toggle wireframes", TextStyle::default())
//             .with_style(Style {
//                 position_type: PositionType::Absolute,
//                 top: Val::Px(12.0),
//                 left: Val::Px(12.0),
//                 ..default()
//             }),
//     );
// }

// fn toggle_wireframe(
//     mut wireframe_config: ResMut<Wireframe2dConfig>,
//     keyboard: Res<ButtonInput<KeyCode>>,
// ) {
//     if keyboard.just_pressed(KeyCode::Space) {
//         wireframe_config.global = !wireframe_config.global;
//     }
// }

fn exit_on_esc(mut exit: EventWriter<AppExit>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(FluidDensity(0.01));
    commands.insert_resource(MousePosition(Vec2::ZERO));

    commands
        .spawn((
            PlayerControlled,
            Position(Vec2::ZERO),
            Velocity(Vec2::ZERO),
            Acceleration(Vec2::ZERO),
            Dampening { max_acc: 0. },
            Mass(1.),
            CrossSectionSize(10.),
            ExperienceDrag { coeff: 1. },
            SelfMoving { accel: 2500. },
        ))
        .insert(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle::new(5.))),
            material: materials.add(Color::linear_rgb(0.4, 0.0, 0.6)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });

    let background_image = asset_server.load("background.png");

    // Spawn the background sprite
    commands.spawn(SpriteBundle {
        texture: background_image,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}

#[derive(Resource, Default)]
struct MousePosition(pub Vec2);

fn update_mouse_position(mut mouse_pos: ResMut<MousePosition>, windows: Query<&Window>) {
    let window = windows.get_single().unwrap();
    if let Some(cursor_position) = window.cursor_position() {
        mouse_pos.0 = cursor_position;
    }
}

/// A world position.
#[derive(Debug, Component)]
struct Position(pub Vec2);

/// Prerequisite: [`Position`]
#[derive(Debug, Component)]
struct Velocity(pub Vec2);

/// Acceleration accumulator.
/// Should be applied at the end of every frame.
///
/// Prerequisite: [`Velocity`]
#[derive(Debug, Component)]
struct Acceleration(pub Vec2);

/// An acceleration that can never increase an entity's absolute velocity.
/// Should be applied before any other accelerations at the end of every frame.
///
/// Prerequisite: [`Velocity`]
#[derive(Debug, Component)]
struct Dampening {
    pub max_acc: f32,
}

#[derive(Debug, Component)]
struct Mass(pub f32);

/// Update position and velocity assuming constant acceleration.
fn update_kinematic(
    mut query: Query<(
        &mut Position,
        &mut Velocity,
        &mut Acceleration,
        Option<&mut Dampening>,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut pos, mut vel, mut acc, dampening) in query.iter_mut() {
        let max_damp_acc = dampening.as_ref().map_or(0., |d| d.max_acc);
        let max_damp_dv = max_damp_acc * dt;

        let v = vel.0.length();
        let damp_dv = max_damp_dv.min(v);

        let dv = acc.0 * dt - vel.0.normalize_or_zero() * damp_dv;
        // vel.0 * dt + 0.5 * dv * dt
        let dp = dt * (vel.0 + 0.5 * dv);

        // if v != 0. {
        //     vel.0 *= 1. - damp_dv / v;
        //     info!("damp_dv / v: {}", damp_dv / v);
        // }

        pos.0 += dp;
        vel.0 += dv;
        acc.0 = Vec2::ZERO;
        if let Some(mut dampening) = dampening {
            dampening.max_acc = 0.;
        }
    }
}

/// Update position of non-accelerating entities.
fn update_movement(
    mut query: Query<(&mut Position, &Velocity), Without<Acceleration>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0 * dt;
    }
}

/// Determine the parameters for a self-moving entity.
///
/// Prerequisite: [`Acceleration`]
#[derive(Debug, Component)]
struct SelfMoving {
    /// Maximum self-acceleration.
    pub accel: f32,
}

/// Action: decelerate to zero velocity.
///
/// Prerequisite: [`SelfMoving`]
#[derive(Debug, Component)]
struct Decelerating;

fn update_decelerating(mut query: Query<(&mut Dampening, &SelfMoving), With<Decelerating>>) {
    for (mut damp, self_moving) in query.iter_mut() {
        damp.max_acc += self_moving.accel;
    }
}

/// Action: move to a destination.
///
/// Prerequisite: [`SelfMoving`]
#[derive(Debug, Component)]
struct MovingTo {
    pub dest: Vec2,
}

/// Action: follow a target.
///
/// Prerequisite: [`MovingTo`]
#[derive(Debug, Component)]
struct Following {
    pub target: Entity,
}

fn update_moving_to_dest(mut query: Query<(&Position, &mut Acceleration, &SelfMoving, &MovingTo)>) {
    for (pos, mut acc, self_moving, &MovingTo { dest }) in query.iter_mut() {
        let da = (dest - pos.0).normalize_or_zero() * self_moving.accel;

        acc.0 += da;
    }
}

fn update_following(
    mut query: Query<(&mut MovingTo, &Following)>,
    positions: Query<&Position, Changed<Position>>,
) {
    for (mut moving_to, &Following { target }) in query.iter_mut() {
        if let Ok(pos) = positions.get(target) {
            moving_to.dest = pos.0;
        }
    }
}

/// Action: move in a direction.
#[derive(Debug, Component)]
struct MovingIn {
    pub dir: Vec2,
}

fn update_moving_in_dir(mut query: Query<(&mut Acceleration, &SelfMoving, &MovingIn)>) {
    for (mut acc, self_moving, &MovingIn { dir }) in query.iter_mut() {
        acc.0 += dir * self_moving.accel;
    }
}

/// This component implies that the entity's cross-section size
/// is the same in all directions, i.e. it is symmetric.
#[derive(Debug, Component)]
struct CrossSectionSize(pub f32);

#[derive(Debug, Component)]
struct ExperienceDrag {
    pub coeff: f32,
}

#[derive(Debug, Resource)]
struct FluidDensity(pub f32);

fn update_drag_symmetric(
    mut query: Query<(
        &Velocity,
        &Mass,
        &mut Dampening,
        &CrossSectionSize,
        &ExperienceDrag,
    )>,
    fluid_density: Res<FluidDensity>,
) {
    // F_d = \frac{1}{2} \rho v^2 A C_d
    for (vel, mass, mut damp, cross_section_size, drag) in query.iter_mut() {
        let force =
            0.5 * fluid_density.0 * vel.0.length_squared() * cross_section_size.0 * drag.coeff;
        let accel = force / mass.0;
        damp.max_acc += accel;
    }
}

// TODO: Implement drag for asymmetric entities.

#[derive(Debug, Component)]
struct PlayerControlled;

fn update_player_controlled(
    mut commands: Commands,
    query: Query<Entity, (With<SelfMoving>, With<PlayerControlled>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let w = keyboard.pressed(KeyCode::KeyW);
    let a = keyboard.pressed(KeyCode::KeyA);
    let s = keyboard.pressed(KeyCode::KeyS);
    let d = keyboard.pressed(KeyCode::KeyD);

    let none = !w && !a && !s && !d;

    if none {
        for entity in query.iter() {
            commands
                .entity(entity)
                .remove::<MovingIn>()
                .insert(Decelerating);
        }
    } else {
        let x = d as i8 - a as i8;
        let y = w as i8 - s as i8;
        let dir = Vec2::new(x as f32, y as f32).normalize(); // It's definitely not zero

        for entity in query.iter() {
            commands
                .entity(entity)
                .remove::<Decelerating>()
                .insert(MovingIn { dir });
        }
    }
}

fn sync_pos_transform(mut query: Query<(&Position, &mut Transform)>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = pos.0.extend(transform.translation.z);
    }
}

fn update_camera_focus(
    mut camera: Query<(&Camera, &mut Transform, &GlobalTransform), With<Camera2d>>,
    player: Query<&Position, With<PlayerControlled>>,
    mouse: Res<MousePosition>,
) {
    let (camera, mut transform, global) = camera.single_mut();
    let player_pos = player.single().0;
    let mouse_viewport_pos = mouse.0;

    let Some(mouse_world) = camera.viewport_to_world_2d(global, mouse_viewport_pos) else {
        return;
    };

    let mid_point = player_pos.lerp(mouse_world, 0.2);

    transform.translation = mid_point.extend(transform.translation.z);
}
