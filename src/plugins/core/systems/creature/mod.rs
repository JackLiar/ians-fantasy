use bevy::prelude::*;

use crate::plugins::core::components::creature::{Human, Hunger, HungerRate, Name};
use crate::plugins::core::resources::MyTimer;

pub fn update_hunger(
    time: Res<Time>,
    mut timer: ResMut<MyTimer>,
    mut query: Query<(&mut Hunger, &HungerRate, &Name), With<Human>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut hunger, rate, name) in &mut query {
            hunger.0 = hunger.0.saturating_sub(rate.0 as u8);
            println!("[{:?}]: {:?}", name, hunger.0);
        }
    }
}

pub fn spawn_human() -> impl Bundle {
    (
        Human,
        Name {
            real_name: "Ian".to_string(),
            nick_name: "Ian".to_string(),
        },
        Hunger(100),
        HungerRate(100),
    )
}

pub fn add_creature(mut commands: Commands) {
    commands.spawn(spawn_human());
    commands.spawn(spawn_human());
    commands.spawn(spawn_human());
}
