use bevy::prelude::*;

use super::allegience::prelude::*;
use super::kinematic::prelude::*;

pub mod components {
    use std::time::Duration;

    use super::*;

    /// Tags an entity as a unit
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
    pub struct Unit;

    #[derive(Debug, Clone, Copy, PartialEq, Default, Component)]
    pub struct HP {
        pub value: f32,
        pub max: f32,
    }

    impl HP {
        pub fn new(value: f32, max: f32) -> Self {
            Self { value, max }
        }
        pub fn full(max: f32) -> Self {
            Self { value: max, max }
        }

        pub fn is_dead(&self) -> bool {
            self.value <= 0.
        }

        pub fn is_alive(&self) -> bool {
            self.value > 0.
        }

        pub fn heal(&mut self, amount: f32) {
            self.value = (self.value + amount).min(self.max);
        }

        pub fn refill(&mut self) {
            self.value = self.max;
        }
    }

    /// Radius for collision
    #[derive(Debug, Clone, Copy, PartialEq, Default, Component)]
    pub struct Radius(pub f32);

    #[derive(Debug, Clone, PartialEq, Eq, Component)]
    pub struct Invulnerability {
        pub duration: Timer,
    }

    impl Invulnerability {
        pub fn new(duration: Duration) -> Self {
            Self {
                duration: Timer::new(duration, TimerMode::Once),
            }
        }
    }
    impl Default for Invulnerability {
        fn default() -> Self {
            Self::new(Duration::ZERO)
        }
    }
}

pub mod bundles {
    use super::*;

    use components::*;

    #[derive(Debug, Default, Bundle)]
    pub struct UnitBundleWithoutFaction {
        pub unit: Unit,
        pub hp: HP,
        pub invulnerability: Invulnerability,
        pub radius: Radius,
        pub kinematic: SymmeticFullKinematic,
    }
    impl UnitBundleWithoutFaction {
        pub fn new(hp: HP, radius: Radius) -> Self {
            Self {
                hp,
                radius,
                kinematic: SymmeticFullKinematic {
                    cross_section_size: CrossSectionSize(2. * radius.0),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Default, Bundle)]
    pub struct UnitBundleWithFaction {
        pub unit: UnitBundleWithoutFaction,
        pub faction: Faction,
    }
    impl UnitBundleWithFaction {
        pub fn new(faction: Faction, hp: HP, radius: Radius) -> Self {
            Self {
                unit: UnitBundleWithoutFaction::new(hp, radius),
                faction,
            }
        }
    }
}

pub mod events {
    use super::*;

    #[derive(Debug, Event)]
    pub struct UnitSpawned(pub Entity);

    #[derive(Debug, Event)]
    pub struct UnitDied(pub Entity);

    // #[derive(Debug, Event)]
    // pub struct UnitWasHit(pub Entity);

    // #[derive(Debug, Event)]
    // pub struct UnitDidHit(pub Entity);
}

pub mod systems {
    use super::*;
    use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

    use components::*;
    use events::*;

    pub fn tick_down_invulnerability(mut query: Query<&mut Invulnerability>, time: Res<Time>) {
        for mut invuln in query.iter_mut() {
            invuln.duration.tick(time.delta());
        }
    }

    // TODO: May change this to an observer
    pub fn add_sprite_to_units(
        mut commands: Commands,
        query: Query<(Entity, &Faction, &Position, &Radius), Added<Unit>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        for (entity, faction, pos, &Radius(radius)) in query.iter() {
            commands.entity(entity).insert(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius })),
                material: materials.add(faction.color()),
                transform: Transform::from_translation(pos.0.extend(0.0)),
                ..default()
            });
        }
    }

    pub fn trigger_unit_spawned_event(
        query: Query<Entity, Added<Unit>>,
        mut events: EventWriter<UnitSpawned>,
    ) {
        for entity in query.iter() {
            events.send(UnitSpawned(entity));
        }
    }

    pub fn trigger_unit_died_event(
        query: Query<(Entity, &HP), Changed<HP>>,
        mut events: EventWriter<UnitDied>,
    ) {
        for (entity, hp) in query.iter() {
            if hp.is_dead() {
                events.send(UnitDied(entity));
            }
        }
    }
}

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        use events::*;
        use systems::*;

        app.add_event::<UnitSpawned>()
            .add_event::<UnitDied>()
            .add_systems(Update, add_sprite_to_units)
            .add_systems(Update, trigger_unit_spawned_event)
            .add_systems(Update, trigger_unit_died_event)
            .add_systems(Update, tick_down_invulnerability);
    }
}

pub mod prelude {
    pub use super::bundles::*;
    pub use super::components::*;
    pub use super::events::*;

    pub use super::UnitPlugin;
}
