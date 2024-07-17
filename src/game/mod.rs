use bevy::{app::PluginGroupBuilder, prelude::*};

pub mod allegience;
pub mod camera;
pub mod kinematic;
pub mod player;
pub mod unit;

#[derive(Default)]
pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>();

        group
            .add(kinematic::KinematicPlugin)
            .add(allegience::AllegiencePlugin::default())
            .add(unit::UnitPlugin)
            .add(player::PlayerPlugin)
            .add(camera::CameraPlugin)
    }
}
