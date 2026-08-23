pub mod orbit;
pub mod selection;

use bevy::prelude::*;

use crate::plugins::core::components::selection::Selectable;

/// 场景对象的默认颜色。
pub const OBJECT_COLOR: Color = Color::srgb(0.2, 0.5, 0.9);

/// 搭建最简三维场景雏形：一个相机、一个方向光源、三个对象（立方体）。
///
/// 成功标准：`cargo r` 后窗口中可以看到一排受光照的蓝色立方体，
/// 按住右键（或触控板双指）拖拽可 360° 环绕查看，
/// 左键拖拽可框选其中的若干对象。
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 相机：位于斜上方，看向原点。
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 方向光：模拟太阳，从左上方照向场景（不启用阴影，保持最简）。
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, -1.0, -0.6, 0.0)),
    ));

    // 三个对象：1x1x1 立方体沿 X 轴排开（底面贴 y=0），
    // 各自独立材质实例，选中高亮互不影响。
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    for x in [-2.0_f32, 0.0, 2.0] {
        commands.spawn((
            Selectable,
            Mesh3d(cube.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: OBJECT_COLOR,
                ..default()
            })),
            Transform::from_xyz(x, 0.5, 0.0),
        ));
    }
}
