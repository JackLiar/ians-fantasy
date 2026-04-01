use bevy::prelude::*;

#[derive(Component)]
pub struct Hunger {
    pub value: u8,
    pub accumulator: f32,
}

#[derive(Component)]
pub struct HungerRate(pub f32);

#[derive(Component)]
pub struct BodyTemperature(pub u8);

#[derive(Component)]
pub struct Human;

#[derive(Component, Debug, Default)]
pub struct Name {
    pub real_name: String,
    pub nick_name: String,
}
