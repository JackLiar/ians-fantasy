use bevy::prelude::*;

/// 可被鼠标框选的对象标记。
#[derive(Component)]
pub struct Selectable;

/// 当前处于选中状态的对象标记。
#[derive(Component)]
pub struct Selected;
