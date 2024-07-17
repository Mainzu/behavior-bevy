use bevy::prelude::*;

use super::allegience::prelude::*;
use super::kinematic::prelude::*;
use super::unit::prelude::*;

pub mod components {
    use super::*;

    /// Tags an entity as a player
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
    pub struct Player;

    /// Tags an entity as the local player
    ///
    /// Prerequisite: [`Player`]

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
    pub struct LocalPlayer;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
    /// Prerequisite: [`SelfMoving`]
    pub struct LocalPlayerControlled;
}

pub mod bundles {
    use super::*;

    use components::*;

    #[derive(Debug, Default, Bundle)]
    pub struct PlayerBundle {
        pub player: Player,
        pub unit: UnitBundleWithFaction,
    }

    #[derive(Debug, Default, Bundle)]
    pub struct LocalPlayerBundle {
        pub local_player: LocalPlayer,
        pub local_player_controlled: LocalPlayerControlled,
        pub player: PlayerBundle,
    }
    impl LocalPlayerBundle {
        pub fn new(player: PlayerBundle) -> Self {
            Self {
                local_player: LocalPlayer,
                local_player_controlled: LocalPlayerControlled,
                player,
            }
        }
    }
}

pub mod systems {
    use super::*;
    use bundles::*;
    use components::*;

    // TODO: Make this configurable
    pub fn spawn_local_player(mut commands: Commands) {
        commands
            .spawn(LocalPlayerBundle::new(PlayerBundle {
                unit: UnitBundleWithFaction::new(Faction::A, HP::full(100.), Radius(5.)),
                ..Default::default()
            }))
            .insert(SelfMoving { accel: 2000. });
    }

    pub fn update_local_player_controlled(
        mut commands: Commands,
        query: Query<Entity, (With<SelfMoving>, With<LocalPlayerControlled>)>,
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
            let dir = Vec2::new(x as f32, y as f32).normalize_or_zero();

            for entity in query.iter() {
                commands
                    .entity(entity)
                    .remove::<Decelerating>()
                    .insert(MovingIn { dir });
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        use bevy::input;

        use systems::*;

        assert!(
            app.world()
                .contains_resource::<input::ButtonInput<KeyCode>>(),
            "Missing resource: ButtonInput<KeyCode>"
        );

        app.add_systems(Startup, spawn_local_player)
            .add_systems(Update, update_local_player_controlled);
    }
}

pub mod prelude {
    pub use super::components::*;

    pub use super::PlayerPlugin;
}
