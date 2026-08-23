use std::f32::consts::FRAC_PI_2;
use std::ops::Range;

use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseScrollUnit,
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::plugins::core::components::selection::Selectable;

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
///
/// 本模块的其他相机输入：
/// - 左键按住地面（y=0 平面）拖动 → 平移，`pan_camera`；
/// - WASD/方向键移动、Q/E 旋转、光标贴近窗口边缘 → `auto_camera`；
/// - 对象上双击左键 → 相机平滑聚焦到该对象，`focus_camera`。
///
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

    apply_orbit_delta(&mut camera, &settings, delta_pitch, delta_yaw);

    // 物理鼠标滚轮缩放：向上滚近、向下滚远，限制在 2.0 ~ 50.0。
    if matches!(mouse_scroll.unit, MouseScrollUnit::Line) && mouse_scroll.delta.y != 0.0 {
        settings.orbit_distance =
            (settings.orbit_distance * (1.0 + mouse_scroll.delta.y * 0.1)).clamp(2.0, 50.0);
    }

    // 保持相机始终指向环绕目标（平移会移动它）。
    let target = settings.target;
    camera.translation = target - camera.forward() * settings.orbit_distance;
}

/// 左键按下分类（按下瞬间判定）与相机平移状态。
///
/// 按下点命中 `Selectable` 对象 → 归对象所有（双击可聚焦相机）；
/// 否则命中地面范围 → 归平移所有；两者都归位后，框选系统才启动。
#[derive(Resource, Default)]
pub struct PanState {
    /// 左键当前按在地面上。
    pressed_on_ground: bool,
    /// 按下时光标下的地面点（y=0）；拖动期间它始终"粘"在光标下。
    anchor: Option<Vec3>,
    /// 左键当前按在 `Selectable` 对象上。
    pressed_on_object: Option<Entity>,
    /// 上一次左键按下的时间（`Time::elapsed_secs`），用于双击检测。
    last_press_time: Option<f32>,
    /// 上一次左键按下时的光标位置（逻辑像素），用于双击检测。
    last_press_pos: Vec2,
}

impl PanState {
    /// 左键是否正按在地面上（框选系统据此让位）。
    pub(crate) fn pressing_on_ground(&self) -> bool {
        self.pressed_on_ground
    }

    /// 左键是否正按在对象上（框选系统据此让位）。
    pub(crate) fn pressing_on_object(&self) -> bool {
        self.pressed_on_object.is_some()
    }
}

/// 相机聚焦过渡：双击对象后，把环绕目标平滑移动到该对象处。
///
/// 过渡期间用户手动移动相机（平移/键盘/边缘）会立即取消过渡——
/// 通过对比"本系统上一帧写入的 target"与"当前 target"检测。
#[derive(Resource, Default)]
pub struct FocusTransition {
    /// 聚焦目标对象（每帧读取其当前位置）。
    goal_entity: Option<Entity>,
    /// 过渡开始时的 target。
    start: Vec3,
    /// 过渡进度 0..1。
    t: f32,
    /// 本系统上一帧写入的 target（用于检测被手动打断）。
    last_applied: Option<Vec3>,
}

impl FocusTransition {
    /// 开始一次聚焦过渡。
    pub(crate) fn begin(&mut self, goal_entity: Entity, current_target: Vec3) {
        self.goal_entity = Some(goal_entity);
        self.start = current_target;
        self.t = 0.0;
        self.last_applied = None;
    }
}

/// 平移：左键按在地面上并拖动鼠标时，相机（环绕目标）随动移动，
/// 按下时的地面点始终粘在光标下方，如同抓住地面拖动，1:1 跟手。
///
/// 按下点分类（按下瞬间判定）：
/// - 命中 `Selectable` 对象 → 归对象所有：不平移、不框选；
///   若构成"双击"（两次按下间隔与距离都很短）则触发相机聚焦（见 `focus_camera`）；
/// - 否则命中地面范围（y=0 顶面矩形，见 `super::GROUND_SIZE`）→ 归平移所有；
/// - 否则（天空区域）→ 归框选所有。
///
/// 按住期间即使光标移出地面范围（射线仍指向 y=0 平面）也继续平移。
///
/// 必须运行在 `orbit_camera`（读 `target` 更新相机位置）与
/// `selection_box_system`（读 `PanState` 决定是否跳过）之前。
pub fn pan_camera(
    mut state: ResMut<PanState>,
    mut settings: ResMut<OrbitCameraSettings>,
    mut focus: ResMut<FocusTransition>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Transform, &Camera, &Projection), With<Camera>>,
    objects: Query<(Entity, &Transform), With<Selectable>>,
    time: Res<Time>,
) {
    let (cam_tf, cam, projection) = *camera;
    let Projection::Perspective(persp) = projection else {
        return; // 三维场景只支持透视相机
    };
    let screen_size = camera_screen_size(cam, &*window);

    // 1) 左键按下：按下点分类（对象 > 地面 > 天空）。
    if mouse.just_pressed(MouseButton::Left) {
        state.pressed_on_ground = false;
        state.anchor = None;
        state.pressed_on_object = None;
        if let Some(cursor) = window.cursor_position() {
            let (origin, dir) =
                cursor_ray(cursor, screen_size, cam_tf, persp.fov, persp.aspect_ratio);
            // 射线命中的最近 Selectable 对象（若有）。
            let mut nearest: Option<Entity> = None;
            let mut nearest_t = f32::INFINITY;
            for (entity, tf) in &objects {
                if let Some(t) = ray_hit_obbox(
                    &origin,
                    &dir,
                    &tf.translation,
                    &tf.rotation,
                    super::OBJECT_HALF_EXTENTS,
                ) {
                    if t < nearest_t {
                        nearest_t = t;
                        nearest = Some(entity);
                    }
                }
            }
            if let Some(entity) = nearest {
                state.pressed_on_object = Some(entity);
                // 构成双击 → 触发相机聚焦到该对象。
                let now = time.elapsed_secs();
                let double = state
                    .last_press_time
                    .is_some_and(|t0| now - t0 < DOUBLE_CLICK_INTERVAL)
                    && (cursor - state.last_press_pos).length() < DOUBLE_CLICK_DIST;
                if double {
                    focus.begin(entity, settings.target);
                }
            } else if let Some(hit) = ray_hit_y0(&origin, &dir) {
                let on_ground = hit.x.abs() <= super::GROUND_SIZE.x * 0.5
                    && hit.z.abs() <= super::GROUND_SIZE.y * 0.5;
                if on_ground {
                    state.pressed_on_ground = true;
                    state.anchor = Some(hit);
                }
            }
            // 记录本次按下，供下一次双击检测。
            state.last_press_time = Some(time.elapsed_secs());
            state.last_press_pos = cursor;
        }
    }

    // 2) 拖动中：精确求解相机应平移的位移，使锚点始终位于光标正下方。
    if state.pressed_on_ground && mouse.pressed(MouseButton::Left) {
        if let (Some(anchor), Some(cursor)) = (state.anchor, window.cursor_position()) {
            let (_, dir) = cursor_ray(cursor, screen_size, cam_tf, persp.fov, persp.aspect_ratio);
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

    // 3) 松开：结束平移/对象按下。
    if !mouse.pressed(MouseButton::Left) {
        state.pressed_on_ground = false;
        state.anchor = None;
        state.pressed_on_object = None;
    }
}

/// 由光标逻辑像素位置构造世界空间射线（起点、方向）。
fn cursor_ray(
    cursor: Vec2,
    screen_size: Vec2,
    cam_tf: &Transform,
    fov: f32,    // 垂直视场角（弧度）
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
        Some(vp) => {
            Vec2::new(vp.physical_size.x as f32, vp.physical_size.y as f32) / window.scale_factor()
        }
        // 0.19 的 `Window::size()` 直接返回逻辑像素。
        None => window.size(),
    }
}

/// 相机自动调整（无需按住鼠标拖动的输入，参照 Kenshi 的相机操作）：
///
/// - **WASD / 方向键**：沿地面移动相机（前进 = 相机朝向投影到地面，
///   左右 = 相机 right 方向）；速度随环绕距离缩放，拉远镜头后
///   同样按键速度覆盖更大地面范围，保持跟手感；
/// - **Q / E**：左转 / 右转相机（与鼠标环绕同向：Q 等价于持续向右拖鼠标）；
/// - **边缘 steering**：光标贴近窗口边缘时，相机向该边方向持续移动
///   （看哪边就往哪边挪），速度随"深入边缘"的距离线性渐增。
///   仅在没有任何鼠标按键按下时启用，避免与环绕/平移/框选手势冲突。
///
/// 必须运行在 `orbit_camera` 之前（两者都写相机 Transform 与 target，链式串行）。
pub fn auto_camera(
    mut settings: ResMut<OrbitCameraSettings>,
    mut camera: Single<&mut Transform, With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // 1) WASD / 方向键：沿地面平移。
    let mut move_dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        move_dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        move_dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        move_dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        move_dir.x += 1.0;
    }
    if move_dir != Vec2::ZERO {
        let move_dir = move_dir.normalize(); // 斜向与轴向速度一致
        // 相机前向/右向投影到地面（y=0）；俯视时前向投影趋近于零，
        // 退化为世界 -Z，避免归一化不稳定。
        let f: Vec3 = camera.forward().into();
        let r: Vec3 = camera.right().into();
        let fwd = Vec3::new(f.x, 0.0, f.z);
        let fwd = if fwd.length_squared() < 1e-8 {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            fwd.normalize()
        };
        let right = Vec3::new(r.x, 0.0, r.z);
        let delta =
            (fwd * move_dir.y + right * move_dir.x) * (KB_PAN_SPEED * settings.orbit_distance) * dt;
        settings.target += delta;
    }

    // 2) Q / E：旋转相机。
    let mut delta_yaw = 0.0;
    if keys.pressed(KeyCode::KeyQ) {
        delta_yaw += KB_YAW_SPEED * dt;
    }
    if keys.pressed(KeyCode::KeyE) {
        delta_yaw -= KB_YAW_SPEED * dt;
    }
    apply_orbit_delta(&mut camera, &settings, 0.0, delta_yaw);

    // 3) 边缘 steering：光标贴近窗口边缘时向该边移动（无任何鼠标按键按下时）。
    let any_mouse = mouse.pressed(MouseButton::Left)
        || mouse.pressed(MouseButton::Right)
        || mouse.pressed(MouseButton::Middle);
    if !any_mouse {
        if let Some(cursor) = window.cursor_position() {
            let size = window.size(); // 逻辑像素
            let mut edge_dir = Vec2::ZERO;
            if cursor.x < EDGE_MARGIN {
                edge_dir.x -= 1.0; // 左边 → 向左
            }
            if size.x - cursor.x < EDGE_MARGIN {
                edge_dir.x += 1.0; // 右边 → 向右
            }
            if cursor.y < EDGE_MARGIN {
                edge_dir.y += 1.0; // 上边 → 向前（深入场景）
            }
            if size.y - cursor.y < EDGE_MARGIN {
                edge_dir.y -= 1.0; // 下边 → 向后
            }
            if edge_dir != Vec2::ZERO {
                let edge_dir = edge_dir.normalize();
                // 深入边缘越深速度越快（线性渐增，贴边最浅处为 0）。
                let depth = EDGE_MARGIN
                    - (cursor
                        .x
                        .min(size.x - cursor.x)
                        .min(cursor.y.min(size.y - cursor.y)))
                    .max(0.0);
                let ramp = (depth / EDGE_MARGIN).clamp(0.0, 1.0);
                let f: Vec3 = camera.forward().into();
                let r: Vec3 = camera.right().into();
                let fwd = Vec3::new(f.x, 0.0, f.z);
                let fwd = if fwd.length_squared() < 1e-8 {
                    Vec3::new(0.0, 0.0, -1.0)
                } else {
                    fwd.normalize()
                };
                let right = Vec3::new(r.x, 0.0, r.z);
                let delta = (fwd * edge_dir.y + right * edge_dir.x)
                    * (KB_PAN_SPEED * settings.orbit_distance)
                    * ramp
                    * dt;
                settings.target += delta;
            }
        }
    }
}

/// 应用相机聚焦过渡（必须运行在所有手动相机系统之后）：
/// 把环绕目标沿平滑曲线（smoothstep）移到双击的对象处；
/// 过渡期间 target 若被其他系统移动（用户接管相机），立即取消。
pub fn focus_camera(
    mut focus: ResMut<FocusTransition>,
    mut settings: ResMut<OrbitCameraSettings>,
    objects: Query<&Transform, With<Selectable>>,
    time: Res<Time>,
) {
    let Some(goal_entity) = focus.goal_entity else {
        return;
    };
    // 本系统上一帧写入后 target 被改动 → 用户接管，取消过渡。
    if let Some(last) = focus.last_applied {
        if (settings.target - last).length_squared() > 1e-6 {
            focus.goal_entity = None;
            focus.last_applied = None;
            return;
        }
    }
    let Ok(tf) = objects.get(goal_entity) else {
        focus.goal_entity = None;
        focus.last_applied = None;
        return;
    };
    focus.t += time.delta_secs() / FOCUS_DURATION;
    let t = focus.t.min(1.0);
    let s = t * t * (3.0 - 2.0 * t); // smoothstep
    let new_target = focus.start.lerp(tf.translation, s);
    settings.target = new_target;
    focus.last_applied = Some(new_target);
    if t >= 1.0 {
        focus.goal_entity = None;
        focus.last_applied = None;
    }
}

/// 对相机应用偏航/俯仰增量（俯仰带 `pitch_range` 限位）。
///
/// 鼠标环绕（`orbit_camera`）与键盘旋转（`auto_camera`）共用，
/// 保证两条输入路径的旋转行为完全一致。
fn apply_orbit_delta(
    camera: &mut Transform,
    settings: &OrbitCameraSettings,
    delta_pitch: f32,
    delta_yaw: f32,
) {
    if delta_pitch == 0.0 && delta_yaw == 0.0 {
        return;
    }
    let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let pitch = (pitch + delta_pitch).clamp(settings.pitch_range.start, settings.pitch_range.end);
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw + delta_yaw, pitch, 0.0);
}

/// 射线与 OBB（有向包围盒）求交，返回最近交点距离；无交返回 `None`。
///
/// 盒体由中心、旋转与半尺寸描述（slab 法，在盒体局部坐标系求解）。
fn ray_hit_obbox(origin: &Vec3, dir: &Vec3, center: &Vec3, rot: &Quat, half: Vec3) -> Option<f32> {
    let inv = rot.inverse();
    let local_origin = inv * (origin - center);
    let local_dir = inv * dir;
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    for (o, d, h) in [
        (local_origin.x, local_dir.x, half.x),
        (local_origin.y, local_dir.y, half.y),
        (local_origin.z, local_dir.z, half.z),
    ] {
        // 解 o + t*d = ±h 得两个平面交点参数；
        // d < 0 时两者顺序颠倒，交换保证 t1 ≤ t2（入口/出口）。
        let (mut t1, mut t2) = if d.abs() < 1e-9 {
            if o.abs() > h {
                return None; // 平行且在该轴上已出界
            }
            (f32::NEG_INFINITY, f32::INFINITY)
        } else {
            let inv_d = 1.0 / d;
            ((-h - o) * inv_d, (h - o) * inv_d)
        };
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        tmin = tmin.max(t1);
        tmax = tmax.min(t2);
        if tmin > tmax {
            return None;
        }
    }
    if tmax < 0.0 {
        return None; // 交点在射线后方
    }
    Some(tmin.max(0.0)) // 射线起点在盒内时返回 0
}

/// 键盘平移速度系数：平移速度 = 环绕距离 × 该系数（单位/秒）。
const KB_PAN_SPEED: f32 = 1.0;
/// 键盘旋转速度（弧度/秒），与快速鼠标环绕拖动的量级相当。
const KB_YAW_SPEED: f32 = 2.0;
/// 边缘 steering：光标距窗口边缘多近（逻辑像素）时开始生效。
const EDGE_MARGIN: f32 = 10.0;
/// 双击判定：两次按下的最大时间间隔（秒），与操作系统默认双击间隔一致。
const DOUBLE_CLICK_INTERVAL: f32 = 0.5;
/// 双击判定：两次按下位置的最大距离（逻辑像素）。
const DOUBLE_CLICK_DIST: f32 = 8.0;
/// 相机聚焦过渡时长（秒）。
const FOCUS_DURATION: f32 = 0.25;

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_4;

    /// 正上方竖直向下（d.y < 0）：交点应在顶面 y = 1.0。
    #[test]
    fn ray_hits_box_from_directly_above() {
        let origin = Vec3::new(0.0, 3.0, 0.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let t = ray_hit_obbox(
            &origin,
            &dir,
            &Vec3::new(0.0, 0.5, 0.0),
            &Quat::IDENTITY,
            Vec3::new(0.5, 0.5, 0.5),
        )
        .expect("应命中");
        assert!((t - 2.0).abs() < 1e-4, "t = {}", t);
    }

    /// 斜上方射向盒心（d.y < 0 且 d.z < 0）：先穿过前面 z = 0.5。
    /// 这是双击聚焦的典型视角（相机 (0,3,5) 看原点处立方体）。
    #[test]
    fn ray_hits_box_oblique_from_above() {
        let origin = Vec3::new(0.0, 3.0, 5.0);
        let dir = (Vec3::new(0.0, 0.5, 0.0) - origin).normalize();
        let t = ray_hit_obbox(
            &origin,
            &dir,
            &Vec3::new(0.0, 0.5, 0.0),
            &Quat::IDENTITY,
            Vec3::new(0.5, 0.5, 0.5),
        )
        .expect("应命中");
        let p = origin + dir * t;
        assert!((p.z - 0.5).abs() < 1e-3, "交点 z = {}", p.z);
        assert!((p.y - 0.75).abs() < 1e-3, "交点 y = {}", p.y);
    }

    /// 正下方竖直向上（d.y > 0）：交点应在底面 y = 0.0。
    #[test]
    fn ray_hits_box_from_directly_below() {
        let origin = Vec3::new(0.0, -3.0, 0.0);
        let dir = Vec3::new(0.0, 1.0, 0.0);
        let t = ray_hit_obbox(
            &origin,
            &dir,
            &Vec3::new(0.0, 0.5, 0.0),
            &Quat::IDENTITY,
            Vec3::new(0.5, 0.5, 0.5),
        )
        .expect("应命中");
        assert!((t - 3.0).abs() < 1e-4, "t = {}", t);
    }

    /// 擦过盒体的射线不命中。
    #[test]
    fn ray_misses_box() {
        let origin = Vec3::new(5.0, 3.0, 5.0);
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let t = ray_hit_obbox(
            &origin,
            &dir,
            &Vec3::ZERO,
            &Quat::IDENTITY,
            Vec3::new(0.5, 0.5, 0.5),
        );
        assert!(t.is_none());
    }

    /// 旋转 45° 的盒体：沿世界 X 轴的射线命中距离 = 5 - 0.5*sqrt(2)。
    #[test]
    fn ray_hits_rotated_box() {
        let rot = Quat::from_axis_angle(Vec3::Y, FRAC_PI_4);
        let t = ray_hit_obbox(
            &Vec3::new(5.0, 0.0, 0.0),
            &Vec3::new(-1.0, 0.0, 0.0),
            &Vec3::ZERO,
            &rot,
            Vec3::new(0.5, 0.5, 0.5),
        )
        .expect("应命中");
        let expected = 5.0 - 0.5 * FRAC_PI_4.sin() * 2.0; // 0.5*cos45 + 0.5*sin45 = 0.5*sqrt(2)
        assert!((t - expected).abs() < 1e-3, "t = {}，期望 {}", t, expected);
    }

    /// 射线起点在盒体内部时返回 0。
    #[test]
    fn ray_origin_inside_box() {
        let t = ray_hit_obbox(
            &Vec3::new(0.1, 0.5, 0.1),
            &Vec3::new(1.0, 0.0, 0.0),
            &Vec3::new(0.0, 0.5, 0.0),
            &Quat::IDENTITY,
            Vec3::new(0.5, 0.5, 0.5),
        )
        .expect("应命中");
        assert!((t - 0.0).abs() < 1e-6);
    }
}
