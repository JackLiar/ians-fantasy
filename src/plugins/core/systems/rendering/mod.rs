pub mod orbit;

use bevy::prelude::*;

/// 搭建最简三维场景雏形：一个相机、一个方向光源、一个对象（立方体）。
///
/// 成功标准：`cargo r` 后窗口中可以看到原点处一个受光照的蓝色立方体。
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

    // 三维空间中的唯一对象：一个 1x1x1 的蓝色立方体，位于原点。
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.5, 0.9),
            ..default()
        })),
    ));
}
