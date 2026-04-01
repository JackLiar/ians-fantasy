use bevy::log::info;
use bevy::prelude::*;

use crate::plugins::core::components::creature::{Human, Hunger, HungerRate, Name};

pub fn update_hunger(
    time: Res<Time>,
    mut query: Query<(&mut Hunger, &HungerRate, &Name), With<Human>>,
) {
    for (mut hunger, rate, name) in &mut query {
        if hunger.value == 0 {
            continue;
        }
        hunger.accumulator += time.delta_secs();
        let interval = 60.0 / rate.0;

        while hunger.accumulator >= interval {
            hunger.accumulator -= interval;
            hunger.value = hunger.value.saturating_sub(1);
        }

        info!("[{:?}]: {} {}", name, hunger.value, rate.0);
    }
}

pub fn spawn_human() -> impl Bundle {
    (
        Human,
        Name {
            real_name: "Ian".to_string(),
            nick_name: "Ian".to_string(),
        },
        Hunger {
            value: 100,
            accumulator: 0.0,
        },
        HungerRate(1.0),
    )
}

pub fn add_creature(mut commands: Commands) {
    commands.spawn(spawn_human());
}
