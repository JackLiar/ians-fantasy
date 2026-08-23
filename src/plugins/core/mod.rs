use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use systems::creature::{add_creature, update_hunger};
use systems::rendering::orbit::{orbit_camera, OrbitCameraSettings};
use systems::rendering::selection::{
    selection_box_system, spawn_selection_box, SelectionState,
};
use systems::rendering::setup_scene;

use crate::plugins::core::resources::TimeScale;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeScale(1))
            .init_resource::<OrbitCameraSettings>()
            .init_resource::<SelectionState>()
            .add_systems(
                Startup,
                (add_creature, setup_scene, spawn_selection_box),
            )
            .add_systems(
                Update,
                (update_hunger, orbit_camera, selection_box_system),
            );
    }
}
