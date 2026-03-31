use bevy::prelude::*;

#[derive(Component)]
pub struct Hunger(pub u8);

#[derive(Component)]
pub struct HungerRate(pub u64);

#[derive(Component)]
pub struct BodyTemperature(pub u8);

#[derive(Component)]
pub struct Human;

#[derive(Component, Debug, Default)]
pub struct Name {
    pub real_name: String,
    pub nick_name: String,
}
