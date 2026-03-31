use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use resources::MyTimer;
use systems::creature::{add_creature, update_hunger};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MyTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(Startup, add_creature)
            .add_systems(Update, update_hunger);
    }
}
