use std::f32::consts::FRAC_PI_2;
use std::ops::Range;

use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit,
};
use bevy::prelude::*;

/// 环绕相机参数。
///
/// 参照 Bevy 0.19 官方示例 `examples/camera/camera_orbit.rs` 的默认值。
#[derive(Resource)]
pub struct OrbitCameraSettings {
    /// 相机到环绕目标（原点）的距离。
    ///
    /// 默认 5.83 = (0, 3, 5).length()，与 `setup_scene` 中相机的初始位置一致，
    /// 保证第一帧无跳变。
    pub orbit_distance: f32,
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

    // 保持相机始终指向环绕目标（原点）。
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * settings.orbit_distance;
}
