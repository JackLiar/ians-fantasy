use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use systems::creature::{add_creature, update_hunger};
use systems::rendering::orbit::{
    auto_camera, focus_camera, orbit_camera, pan_camera, FocusTransition, OrbitCameraSettings,
    PanState,
};
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
            .init_resource::<PanState>()
            .init_resource::<FocusTransition>()
            .init_resource::<SelectionState>()
            .add_systems(
                Startup,
                (add_creature, setup_scene, spawn_selection_box),
            )
            .add_systems(
                Update,
                // 链内串行：平移/自动调整/键盘先更新 target 与相机，环绕再摆放相机，
                // 聚焦过渡最后检测"是否被手动接管"，框选读取 PanState 让位。
                (
                    update_hunger,
                    (
                        pan_camera,
                        auto_camera,
                        orbit_camera,
                        focus_camera,
                        selection_box_system,
                    )
                        .chain(),
                ),
            );
    }
}
