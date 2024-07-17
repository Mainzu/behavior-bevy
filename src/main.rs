use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle, Wireframe2dPlugin},
};

mod game;
mod mouse;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Wireframe2dPlugin))
        .add_plugins(mouse::MousPlugin)
        .add_plugins(game::GamePlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, exit_on_esc)
        // .add_systems(Update, toggle_wireframe)
        .run();
}

fn exit_on_esc(mut exit: EventWriter<AppExit>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // commands
    //     .spawn((
    //         LocalPlayerControlled,
    //         Position(Vec2::ZERO),
    //         Velocity(Vec2::ZERO),
    //         Acceleration(Vec2::ZERO),
    //         Dampening { max_acc: 0. },
    //         Mass(1.),
    //         CrossSectionSize(10.),
    //         ExperienceDrag { coeff: 1. },
    //         SelfMoving { accel: 2500. },
    //     ))
    //     .insert(MaterialMesh2dBundle {
    //         mesh: Mesh2dHandle(meshes.add(Circle::new(5.))),
    //         material: materials.add(Color::linear_rgb(0.4, 0.0, 0.6)),
    //         transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //         ..default()
    //     });

    let background_image = asset_server.load("temp/background.png");

    // Spawn the background sprite
    commands.spawn(SpriteBundle {
        texture: background_image,
        transform: Transform::from_xyz(0.0, 0.0, -1.0),
        ..default()
    });
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
