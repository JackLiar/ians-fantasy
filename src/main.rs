use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;

use crate::plugins::core::CorePlugin;

mod plugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info".to_string(),
            level: Level::INFO,
            ..Default::default()
        }))
        .add_plugins(CorePlugin)
        .run();
}
