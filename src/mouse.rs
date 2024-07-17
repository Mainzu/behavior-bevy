//! Maintain mouse position in a resource

use bevy::prelude::*;

#[derive(Resource)]
pub struct MousePosition(pub Vec2);

impl FromWorld for MousePosition {
    fn from_world(world: &mut World) -> Self {
        let window = world.query::<&Window>().single(world);

        Self(
            window
                .cursor_position()
                .unwrap_or_else(|| window.size() / 2.0),
        )
    }
}

pub fn update_mouse_position(windows: Query<&Window>, mut mouse_pos: ResMut<MousePosition>) {
    let window = windows.get_single().unwrap();
    if let Some(cursor_position) = window.cursor_position() {
        mouse_pos.0 = cursor_position;
    }
}

pub struct MousPlugin;
impl Plugin for MousPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePosition>()
            .add_systems(Update, update_mouse_position);
    }
}
