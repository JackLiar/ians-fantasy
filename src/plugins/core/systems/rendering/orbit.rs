use std::f32::consts::FRAC_PI_2;
use std::ops::Range;

use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// 环绕相机参数。
///
/// 参照 Bevy 0.19 官方示例 `examples/camera/camera_orbit.rs` 的默认值。
#[derive(Resource)]
pub struct OrbitCameraSettings {
    /// 相机到环绕目标的距离。
    ///
    /// 默认 5.83 = (0, 3, 5).length()，与 `setup_scene` 中相机的初始位置一致，
    /// 保证第一帧无跳变。
    pub orbit_distance: f32,
    /// 环绕目标（相机始终注视的中心点）。
    ///
    /// 默认为原点；在地面上左键拖动（平移，见 `pan_camera`）会移动它。
    pub target: Vec3,
    /// 俯仰速度（弧度 / 像素）。
    pub pitch_speed: f32,
    /// 偏航速度（弧度 / 像素），无限制，可 360° 连续环绕。
    pub yaw_speed: f32,
    /// 俯仰限制范围，防止越过天顶/天底翻转。
    pub pitch_range: Range<f32>,
}

impl Default for OrbitCameraSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: Vec3::new(0.0, 3.0, 5.0).length(),
            target: Vec3::ZERO,
            pitch_speed: 0.003,
            yaw_speed: 0.004,
            pitch_range: -pitch_limit..pitch_limit,
        }
    }
}

/// 每帧根据鼠标输入更新相机，使其绕原点（环绕目标）做 360° 环绕。
///
/// 旋转必须"按住 + 移动"：
/// - **Mac 触控板**：双指按住（点击）并移动手指。双指点击是 macOS 的
///   "secondary click"，经 `rightMouseDown:` 上报（buttonNumber=1），
///   因此 winit 映射为 `MouseButton::Right`。按住期间移动量可能以
///   `Pixel` 单位滚轮事件或鼠标位移事件上报，两者都取；
/// - **物理鼠标**：按住中键（滚轮按压）或右键并移动鼠标，由鼠标位移驱动。
///
/// 缩放：**物理鼠标滚轮**（`Line` 单位）→ 缩放距离。
/// 平移：**左键**按住地面（y=0 平面）并拖动，由 `pan_camera` 处理。
/// 触控板普通双指滑动（未点击）不产生任何相机操作。
pub fn orbit_camera(
    mut camera: Single<&mut Transform, With<Camera>>,
    mut settings: ResMut<OrbitCameraSettings>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    // 仅按住右键（触控板双指点击）或中键（物理鼠标按压滚轮）时才旋转。
    let (delta_pitch, delta_yaw) = if mouse_buttons.pressed(MouseButton::Right)
        || mouse_buttons.pressed(MouseButton::Middle)
    {
        // 触控板双指按住并移动时指针不动，移动量以 Pixel 单位滚轮事件上报；
        // 物理鼠标则产生鼠标位移。优先取滚轮增量，取不到再取位移。
        // 二者都已是"自上一帧以来的全量"，不再乘 delta_time。
        let (dx, dy) = if matches!(mouse_scroll.unit, MouseScrollUnit::Pixel)
            && mouse_scroll.delta != Vec2::ZERO
        {
            (mouse_scroll.delta.x, mouse_scroll.delta.y)
        } else {
            let d = mouse_motion.delta;
            (d.x, d.y)
        };
        (dy * settings.pitch_speed, dx * settings.yaw_speed)
    } else {
        (0.0, 0.0)
    };

    if delta_pitch != 0.0 || delta_yaw != 0.0 {
        let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
        let pitch = (pitch + delta_pitch).clamp(settings.pitch_range.start, settings.pitch_range.end);
        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw + delta_yaw, pitch, 0.0);
    }

    // 物理鼠标滚轮缩放：向上滚近、向下滚远，限制在 2.0 ~ 50.0。
    if matches!(mouse_scroll.unit, MouseScrollUnit::Line)
        && mouse_scroll.delta.y != 0.0
    {
        settings.orbit_distance = (settings.orbit_distance * (1.0 + mouse_scroll.delta.y * 0.1))
            .clamp(2.0, 50.0);
    }

    // 保持相机始终指向环绕目标（平移会移动它）。
    let target = settings.target;
    camera.translation = target - camera.forward() * settings.orbit_distance;
}

/// 相机平移（左键拖动）状态。
///
/// `pressed_on_ground` 供框选系统读取以跳过手势：
/// 落在地面上的左键属于平移，不启动框选。
#[derive(Resource, Default)]
pub struct PanState {
    /// 左键当前按在地面上。
    pressed_on_ground: bool,
    /// 按下时光标下的地面点（y=0）；拖动期间它始终"粘"在光标下。
    anchor: Option<Vec3>,
}

impl PanState {
    /// 左键是否正按在地面上（框选系统据此让位）。
    pub(crate) fn pressing_on_ground(&self) -> bool {
        self.pressed_on_ground
    }
}

/// 平移：左键按在地面上并拖动鼠标时，相机（环绕目标）随动移动，
/// 按下时的地面点始终粘在光标下方，如同抓住地面拖动，1:1 跟手。
///
/// - 按下点必须在地面范围内（y=0 顶面矩形，见 `super::GROUND_SIZE`），
///   否则不平移（左键落在天空区域时仍走框选）；
/// - 按住期间即使光标移出地面范围（射线仍指向 y=0 平面）也继续平移；
/// - 必须运行在 `orbit_camera`（读 `target` 更新相机位置）与
///   `selection_box_system`（读 `PanState` 决定是否跳过）之前。
pub fn pan_camera(
    mut state: ResMut<PanState>,
    mut settings: ResMut<OrbitCameraSettings>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Transform, &Camera, &Projection), With<Camera>>,
) {
    let (cam_tf, cam, projection) = *camera;
    let Projection::Perspective(persp) = projection else {
        return; // 三维场景只支持透视相机
    };
    let screen_size = camera_screen_size(cam, &*window);

    // 1) 左键按下：按下点在地面范围内则开始平移。
    if mouse.just_pressed(MouseButton::Left) {
        state.pressed_on_ground = false;
        state.anchor = None;
        if let Some(cursor) = window.cursor_position() {
            let (origin, dir) =
                cursor_ray(cursor, screen_size, cam_tf, persp.fov, persp.aspect_ratio);
            if let Some(hit) = ray_hit_y0(&origin, &dir) {
                let on_ground = hit.x.abs() <= super::GROUND_SIZE.x * 0.5
                    && hit.z.abs() <= super::GROUND_SIZE.y * 0.5;
                if on_ground {
                    state.pressed_on_ground = true;
                    state.anchor = Some(hit);
                }
            }
        }
    }

    // 2) 拖动中：精确求解相机应平移的位移，使锚点始终位于光标正下方。
    if state.pressed_on_ground && mouse.pressed(MouseButton::Left) {
        if let (Some(anchor), Some(cursor)) = (state.anchor, window.cursor_position()) {
            let (_, dir) =
                cursor_ray(cursor, screen_size, cam_tf, persp.fov, persp.aspect_ratio);
            if dir.y < -1e-6 {
                // 过光标的射线方向只取决于相机朝向；锚点在 y=0 上，
                // 解出射线经过锚点的距离 t，相机（即环绕目标）应平移 delta。
                let t = -cam_tf.translation.y / dir.y;
                let delta = anchor - cam_tf.translation - t * dir;
                if delta.length_squared() > 1e-10 {
                    settings.target += delta;
                }
            }
        }
    }

    // 3) 松开：结束平移。
    if !mouse.pressed(MouseButton::Left) {
        state.pressed_on_ground = false;
        state.anchor = None;
    }
}

/// 由光标逻辑像素位置构造世界空间射线（起点、方向）。
fn cursor_ray(
    cursor: Vec2,
    screen_size: Vec2,
    cam_tf: &Transform,
    fov: f32, // 垂直视场角（弧度）
    aspect: f32, // 宽 / 高
) -> (Vec3, Vec3) {
    let ndc_x = cursor.x / screen_size.x * 2.0 - 1.0;
    let ndc_y = 1.0 - cursor.y / screen_size.y * 2.0;
    let half_tan = (fov * 0.5).tan();
    let dir_local = Vec3::new(ndc_x * half_tan * aspect, ndc_y * half_tan, -1.0).normalize();
    (cam_tf.translation, cam_tf.rotation * dir_local)
}

/// 射线与 y=0 平面求交；射线与平面平行或指向上方时返回 `None`。
fn ray_hit_y0(origin: &Vec3, dir: &Vec3) -> Option<Vec3> {
    if dir.y >= -1e-6 {
        return None;
    }
    let t = -origin.y / dir.y;
    (t > 0.0).then(|| origin + dir * t)
}

/// 相机视口的逻辑像素尺寸（主相机即整个窗口）。
pub(crate) fn camera_screen_size(camera: &Camera, window: &Window) -> Vec2 {
    match &camera.viewport {
        Some(vp) => Vec2::new(vp.physical_size.x as f32, vp.physical_size.y as f32)
            / window.scale_factor(),
        // 0.19 的 `Window::size()` 直接返回逻辑像素。
        None => window.size(),
    }
}
