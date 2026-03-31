use bevy::prelude::*;

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct Health {
    pub blood: u8,
    pub head: u8,
    pub chest: u8,
    pub stomach: u8,
    pub right_arm: u8,
    pub left_arm: u8,
    pub right_leg: u8,
    pub left_leg: u8,
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct HitRate {
    pub head: u8,
    pub chest: u8,
    pub stomach: u8,
    pub right_arm: u8,
    pub left_arm: u8,
    pub right_leg: u8,
    pub left_leg: u8,
}

#[derive(Clone, Copy, Component, Debug, Default)]
pub struct BooldRate(u64);
