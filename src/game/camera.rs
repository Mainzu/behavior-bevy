use bevy::prelude::*;

use super::kinematic::prelude::*;
use super::player::prelude::*;

use crate::mouse::MousePosition;

pub mod systems {
    use super::*;

    pub fn setup_camera(mut commands: Commands) {
        commands.spawn(Camera2dBundle::default());
    }

    pub fn update_camera_focus(
        mut camera: Query<(&Camera, &mut Transform, &GlobalTransform), With<Camera2d>>,
        player: Query<&Position, With<LocalPlayer>>,
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
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;

        assert!(
            app.world().contains_resource::<MousePosition>(),
            "Missing resource: MousePosition"
        );

        app.add_systems(Startup, setup_camera)
            .add_systems(Update, update_camera_focus);
    }
}
