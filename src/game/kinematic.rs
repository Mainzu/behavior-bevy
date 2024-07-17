use bevy::prelude::*;

pub mod components {
    use super::*;

    /// A world position.
    #[derive(Debug, Default, Component)]
    pub struct Position(pub Vec2);

    /// Prerequisite: [`Position`]
    #[derive(Debug, Default, Component)]
    pub struct Velocity(pub Vec2);

    /// Acceleration accumulator.
    /// Should be applied at the end of every frame.
    ///
    /// Prerequisite: [`Velocity`]
    #[derive(Debug, Default, Component)]
    pub struct Acceleration(pub Vec2);
    impl Acceleration {
        pub fn accumulate(&mut self, acc: Vec2) {
            self.0 += acc;
        }
        pub fn reset(&mut self) {
            self.0 = Vec2::ZERO;
        }
    }

    /// An acceleration that can never increase an entity's absolute velocity.
    /// Should be applied before any other accelerations at the end of every frame.
    ///
    /// Prerequisite: [`Velocity`]
    #[derive(Debug, Default, Component)]
    pub struct Dampening {
        pub max_acc: f32,
    }
    impl Dampening {
        pub fn accumulate(&mut self, acc: f32) {
            self.max_acc += acc;
        }
        pub fn reset(&mut self) {
            self.max_acc = 0.
        }
    }

    #[derive(Debug, Component)]
    pub struct Mass(pub f32);
    impl Default for Mass {
        fn default() -> Self {
            Mass(1.)
        }
    }

    /// Determine the parameters for a self-moving entity.
    ///
    /// Prerequisite: [`Acceleration`]
    #[derive(Debug, Component)]
    pub struct SelfMoving {
        /// Maximum self-acceleration.
        pub accel: f32,
    }

    /// Action: decelerate to zero velocity.
    ///
    /// Prerequisite: [`SelfMoving`]
    #[derive(Debug, Default, Component)]
    pub struct Decelerating;

    /// Action: move to a destination.
    ///
    /// Prerequisite: [`SelfMoving`]
    #[derive(Debug, Default, Component)]
    pub struct MovingTo {
        pub dest: Vec2,
    }

    /// Action: follow a target.
    ///
    /// Prerequisite: [`MovingTo`]
    #[derive(Debug, Component)]
    pub struct Following {
        pub target: Entity,
    }

    /// Action: move in a direction.
    #[derive(Debug, Default, Component)]
    pub struct MovingIn {
        pub dir: Vec2,
    }

    /// This component implies that the entity's cross-section size
    /// is the same in all directions, i.e. it is symmetric.
    #[derive(Debug, Component)]
    pub struct CrossSectionSize(pub f32);
    impl Default for CrossSectionSize {
        fn default() -> Self {
            Self(1.)
        }
    }

    #[derive(Debug, Component)]
    pub struct ExperienceDrag {
        pub coeff: f32,
    }
    impl Default for ExperienceDrag {
        fn default() -> Self {
            Self { coeff: 1. }
        }
    }
}

pub mod bundles {
    use super::*;
    use components::*;

    #[derive(Debug, Default, Bundle)]
    pub struct FullKinematic {
        pub position: Position,
        pub velocity: Velocity,
        pub acceleration: Acceleration,
        pub dampening: Dampening,
        pub mass: Mass,
        pub experience_drag: ExperienceDrag,
    }

    #[derive(Debug, Default, Bundle)]
    pub struct SymmeticFullKinematic {
        pub position: Position,
        pub velocity: Velocity,
        pub acceleration: Acceleration,
        pub dampening: Dampening,
        pub mass: Mass,
        pub cross_section_size: CrossSectionSize,
        pub experience_drag: ExperienceDrag,
    }
}

mod resources {
    use super::*;

    #[derive(Debug, Resource)]
    pub struct FluidDensity(pub f32);
}

mod systems {
    use super::*;
    use components::*;
    use resources::*;

    pub fn sync_pos_transform(mut query: Query<(&Position, &mut Transform)>) {
        for (pos, mut transform) in query.iter_mut() {
            transform.translation = pos.0.extend(transform.translation.z);
        }
    }

    /// Update position and velocity assuming constant acceleration.
    pub fn update_kinematic(
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

            pos.0 += dp;
            vel.0 += dv;
            acc.reset();
            if let Some(mut dampening) = dampening {
                dampening.reset();
            }
        }
    }

    /// Update position of non-accelerating entities.
    pub fn update_movement(
        mut query: Query<(&mut Position, &Velocity), Without<Acceleration>>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mut pos, vel) in query.iter_mut() {
            pos.0 += vel.0 * dt;
        }
    }

    pub fn update_moving_to_dest(
        mut query: Query<(&Position, &mut Acceleration, &SelfMoving, &MovingTo)>,
    ) {
        for (pos, mut acc, self_moving, &MovingTo { dest }) in query.iter_mut() {
            let da = (dest - pos.0).normalize_or_zero() * self_moving.accel;

            acc.accumulate(da);
        }
    }

    pub fn update_following(
        mut query: Query<(&mut MovingTo, &Following)>,
        positions: Query<&Position, Changed<Position>>,
    ) {
        for (mut moving_to, &Following { target }) in query.iter_mut() {
            if let Ok(pos) = positions.get(target) {
                moving_to.dest = pos.0;
            }
        }
    }

    pub fn update_moving_in_dir(mut query: Query<(&mut Acceleration, &SelfMoving, &MovingIn)>) {
        for (mut acc, self_moving, &MovingIn { dir }) in query.iter_mut() {
            acc.accumulate(dir * self_moving.accel);
        }
    }

    pub fn update_decelerating(
        mut query: Query<(&mut Dampening, &SelfMoving), With<Decelerating>>,
    ) {
        for (mut damp, self_moving) in query.iter_mut() {
            damp.accumulate(self_moving.accel);
        }
    }

    pub fn update_drag_symmetric(
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

            damp.accumulate(accel);
        }
    }

    // TODO: Implement drag for asymmetric entities.
}

pub struct KinematicPlugin;

impl Plugin for KinematicPlugin {
    fn build(&self, app: &mut App) {
        use resources::*;
        use systems::*;

        let update_ordered = (
            update_decelerating,
            update_kinematic,
            update_movement,
            update_moving_in_dir,
            update_following,
            update_moving_to_dest,
            update_drag_symmetric,
            sync_pos_transform,
        );

        app.insert_resource(FluidDensity(0.001))
            .add_systems(Update, update_ordered);
    }
}

pub mod prelude {
    pub use super::bundles::*;
    pub use super::components::*;
    pub use super::resources::*;

    pub use super::KinematicPlugin;
}
