//! 鼠标框选：左键拖拽在屏幕上画出矩形，松开后选中投影落入矩形内的对象。
//!
//! 与相机手势的分工（按下点分类，见 `orbit::pan_camera`）：左键按在
//! `Selectable` 对象上归对象所有（双击可聚焦相机），框选对其让位；
//! 其余（地面或天空）归框选所有。
//!
//! Bevy 0.19 要点（均已对照 registry 源码确认）：
//! - UI 的 `Node` 组件用 `left/top/width/height: Val`（逻辑像素）定位，
//!   布局系统 `ui_layout_system` 运行在 PostUpdate（Update 之后），
//!   因此 Update 中修改的节点当帧即可渲染；
//! - 无父节点的 UI 节点会自动挂到隐式的全窗口根节点下；
//! - 0.19 没有 `ViewProjection` 组件，这里用相机基向量做几何投影
//!   （屏幕 x/y 与深度编码方式无关）；
//! - 0.19 移除了 `Windows` 资源：`Window` 本身是组件，主窗口实体带
//!   `PrimaryWindow` 标记，用 `Single<&Window, With<PrimaryWindow>>` 获取；
//! - 给已有实体增删组件用 `Commands::entity(entity)`（0.19 中由
//!   `entity_mut` 改名而来，返回带 `insert`/`remove` 的 `EntityCommands`）。

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::OBJECT_COLOR;
use super::orbit::{PanState, camera_screen_size};
use crate::plugins::core::components::selection::{Selectable, Selected};

/// 选中对象的高亮颜色。
pub const SELECTION_COLOR: Color = Color::srgb(1.0, 0.85, 0.25);

/// 框选矩形标记（UI 节点，绝对定位）。
#[derive(Component)]
pub struct SelectionBox;

/// 框选状态。
#[derive(Resource, Default)]
pub struct SelectionState {
    /// 是否正在框选（左键按住并拖动中）。
    dragging: bool,
    /// 框选起点（逻辑像素）。
    start: Vec2,
}

/// 生成框选矩形。默认隐藏（`Display::None`），框选开始时显示。
pub fn spawn_selection_box(mut commands: Commands) {
    commands.spawn((
        SelectionBox,
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
        BorderColor::all(Color::WHITE),
        ZIndex(10),
    ));
}

/// 框选系统：
/// - 左键按下 → 开始框选，显示矩形；
/// - 按住期间移动鼠标 → 实时同步矩形（框选过程中始终可见）；
/// - 松开 → 隐藏矩形，把中心点投影落入矩形内的对象标记为选中并高亮；
///   未拖动的"单击"产生零面积矩形，自然清空选中。
pub fn selection_box_system(
    mut state: ResMut<SelectionState>,
    mut box_node: Single<&mut Node, With<SelectionBox>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Transform, &Camera, &Projection), With<Camera>>,
    objects: Query<
        (
            Entity,
            &Transform,
            Option<&MeshMaterial3d<StandardMaterial>>,
        ),
        With<Selectable>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    pan: Res<PanState>,
) {
    // 左键按在对象上属于双击聚焦（见 `orbit::pan_camera`），让位给它；
    // 按在地面或天空均启动框选。
    if pan.pressing_on_object() {
        return;
    }

    let window = &*window;
    let cursor = window.cursor_position(); // 逻辑像素

    // 1) 左键按下：开始框选。
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(pos) = cursor {
            state.dragging = true;
            state.start = pos;
            box_node.display = Display::Flex;
        }
    }

    if !state.dragging {
        return;
    }
    let Some(pos) = cursor else {
        return;
    };

    if mouse.pressed(MouseButton::Left) {
        // 2) 拖拽中：实时更新矩形（UI 布局在 PostUpdate 运行，当帧生效）。
        let (min, max) = (state.start.min(pos), state.start.max(pos));
        box_node.left = Val::Px(min.x);
        box_node.top = Val::Px(min.y);
        box_node.width = Val::Px(max.x - min.x);
        box_node.height = Val::Px(max.y - min.y);
        return;
    }

    // 3) 松开：隐藏矩形，计算并应用选中结果。
    state.dragging = false;
    box_node.display = Display::None;

    let (cam_tf, cam, projection) = &*camera;
    let Projection::Perspective(persp) = projection else {
        return; // 三维场景只支持透视相机
    };
    let screen_size = camera_screen_size(cam, window);
    let (min, max) = (state.start.min(pos), state.start.max(pos));

    // 第一阶段（只读）：计算每个对象的目标选中状态。
    struct Change {
        entity: Entity,
        inside: bool,
        material: Option<Handle<StandardMaterial>>,
    }
    let changes: Vec<Change> = objects
        .iter()
        .map(|(entity, tf, mat)| {
            let inside = project_to_screen(
                tf.translation,
                cam_tf,
                persp.fov,
                persp.aspect_ratio,
                screen_size,
            )
            .is_some_and(|p| (min.x..=max.x).contains(&p.x) && (min.y..=max.y).contains(&p.y));
            Change {
                entity,
                inside,
                // 0.19 的 `Handle` 不再是 `Copy`。
                material: mat.map(|m| m.0.clone()),
            }
        })
        .collect();

    // 第二阶段：应用组件变化（Commands 延迟到系统结束后应用）。
    for change in &changes {
        if change.inside {
            commands.entity(change.entity).insert(Selected);
        } else {
            commands.entity(change.entity).remove::<Selected>();
        }
    }

    // 第三阶段：应用高亮颜色（每个立方体有独立材质实例，互不影响）。
    for change in changes {
        if let Some(handle) = change.material {
            // 0.19 的 `Assets::get_mut` 返回 `AssetMut`（带变更通知的 `DerefMut`）。
            if let Some(mut mat) = materials.get_mut(&handle) {
                mat.base_color = if change.inside {
                    SELECTION_COLOR
                } else {
                    OBJECT_COLOR
                };
            }
        }
    }
}

/// 把世界空间点投影到相机屏幕（逻辑像素），点在相机后方时返回 `None`。
fn project_to_screen(
    world: Vec3,
    cam: &Transform,
    fov: f32,          // 垂直视场角（弧度）
    aspect: f32,       // 宽 / 高
    screen_size: Vec2, // 逻辑像素
) -> Option<Vec2> {
    let d = world - cam.translation;
    // 0.19 中 `Transform::forward/right/up` 返回 `Dir3`，转成 `Vec3` 再做点积。
    let z = d.dot(cam.forward().into());
    if z <= 0.0 {
        return None;
    }
    let half_h = z * (fov * 0.5).tan();
    let half_w = half_h * aspect;
    let ndc_x = d.dot(cam.right().into()) / half_w;
    let ndc_y = d.dot(cam.up().into()) / half_h;
    Some(Vec2::new(
        (ndc_x * 0.5 + 0.5) * screen_size.x,
        (1.0 - (ndc_y * 0.5 + 0.5)) * screen_size.y,
    ))
}
