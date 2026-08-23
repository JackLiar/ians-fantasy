pub mod orbit;
pub mod selection;

use bevy::prelude::*;

use crate::plugins::core::components::selection::Selectable;

/// 场景对象的默认颜色。
pub const OBJECT_COLOR: Color = Color::srgb(0.2, 0.5, 0.9);

/// 地面平面的颜色。
pub const GROUND_COLOR: Color = Color::srgb(0.55, 0.62, 0.5);

/// 地面尺寸（x 向宽、z 向深），顶面位于 y=0。
///
/// 供 `setup_scene`（生成地面网格）与 `orbit::pan_camera`（判定左键是否
/// 按在地面上）共用。
pub const GROUND_SIZE: Vec2 = Vec2::new(12.0, 8.0);

/// `Selectable` 对象（1x1x1 立方体）的半尺寸，
/// 供 `orbit::pan_camera` 的射线命中测试（双击聚焦）使用。
pub const OBJECT_HALF_EXTENTS: Vec3 = Vec3::new(0.5, 0.5, 0.5);

/// 搭建最简三维场景雏形：一个相机、一个方向光源、一个可视化地面、
/// 三个落在地面上的对象（立方体）。
///
/// 成功标准：`cargo r` 后窗口中可以看到一排受光照的蓝色立方体
/// 落在绿色地面上，按住右键（或触控板双指）拖拽可 360° 环绕查看，
/// 在地面上按住左键拖拽可平移相机，在天空区域左键拖拽可框选其中的若干对象，
/// 双击立方体可让相机平滑聚焦到它，WASD/方向键移动相机、Q/E 旋转相机、
/// 光标贴近窗口边缘时相机随之移动（详见 `orbit` 模块）。
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

    // 可视化地面：一个很薄的立方体，顶面正好位于 y=0。
    // 立方体底面（y=0）与其共面，视觉上"落在"地面上；
    // 不挂 Selectable，不参与框选。
    let ground = meshes.add(Cuboid::new(GROUND_SIZE.x, 0.1, GROUND_SIZE.y));
    commands.spawn((
        Mesh3d(ground),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GROUND_COLOR,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.05, 0.0),
    ));

    // 三个对象：1x1x1 立方体沿 X 轴排开（底面贴 y=0，即落在地面上），
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
