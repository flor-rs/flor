use crate::error::Error;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRendererError, LoadRenderResource};
use crate::signal::Id;
use crate::view::control_state::ControlState;
use crate::view::handler::ViewHandler;
#[cfg(feature = "class")]
use crate::view::resolver::{Class, LayoutResolver};
use crate::view::{ScrollState, View, ViewState, VIEW_STORAGE};
use crate::windows::{render_from_view_id, WindowBusDispatchEntry, WindowEntryVisit};
use flor_base::graphics::RenderContext;
#[cfg(feature = "drag-drop")]
use flor_base::platform::{DragFormat, DropEffect, KeyState};
use flor_base::platform::{MousePosition, WindowOperations};
use flor_base::types::Transform2D;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use platform::WindowId;
use rustc_hash::FxHashMap;
use slotmap::new_key_type;
use std::any::Any;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

new_key_type! {
    pub struct ViewId;
}
impl Display for ViewId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("ViewId {:?}", self))
    }
}

/// view不需要根据窗口进行分类隔离
pub static VIEW_WINDOW: Lazy<FxHashMap<ViewId, WindowId>> = Lazy::new(|| FxHashMap::default());

impl ViewId {
    #[inline]
    pub fn new() -> ViewId {
        VIEW_STORAGE.new_view()
    }

    #[inline]
    pub fn new_with_layout(layout_style_fn: impl FnOnce(ViewId) -> LayoutResolver) -> ViewId {
        let view_id = VIEW_STORAGE.view_ids.lock().insert(());
        let layout_style = layout_style_fn(view_id).normal_layer();
        VIEW_STORAGE.states.write().insert(
            view_id,
            RwLock::new(ViewState {
                layout: Default::default(),
                abs_location: (0.0, 0.0),
                node_id: None,
                layout_style,
                dirty_children: false,
                disable: false,
            }),
        );
        VIEW_STORAGE
            .handlers
            .write()
            .insert(view_id, RwLock::new(ViewHandler::default()));
        view_id
    }

    pub fn parent_view_id(self) -> Option<ViewId> {
        VIEW_STORAGE.parent_view_id.read().get(self).cloned()
    }

    // pub states: RwLock<SecondaryMap<ViewId, RwLock<ViewState>>>,
    pub fn layout(self) -> Result<taffy::Layout, Error> {
        self.with_state(|state| state.layout)
    }

    /// 获取当前状态的 Style（克隆版本）
    pub fn get_current_style(self) -> Result<taffy::Style, Error> {
        self.with_state(|view_state| view_state.layout_style.get_data_clone(self.control_state()))
    }

    /// 借用当前状态的 Style（闭包版本）
    pub fn with_current_style<R>(self, f: impl FnOnce(&taffy::Style) -> R) -> Result<R, Error> {
        let control_state = self.control_state();
        self.with_state(|view_state| {
            let style = view_state.layout_style.get_data_borrow(control_state);
            f(&style)
        })
    }

    pub fn with_state<R>(self, getter: impl FnOnce(&ViewState) -> R) -> Result<R, Error> {
        let state_map = VIEW_STORAGE.states.read();
        let state = state_map
            .get(self)
            .ok_or(Error::ControlUnregistered(self))?
            .read();

        let result = getter(state.deref());

        Ok(result)
    }

    pub fn with_state_mut<R>(self, getter: impl FnOnce(&mut ViewState) -> R) -> Result<R, Error> {
        let state_map = VIEW_STORAGE.states.read();
        let mut state = state_map
            .get(self)
            .ok_or(Error::ControlUnregistered(self))?
            .write();

        let result = getter(state.deref_mut());

        Ok(result)
    }

    pub fn update_state(self, state: Box<dyn Any>) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().on_update_state(state);
            let _ = self.request_redraw();
        }
    }

    #[cfg(feature = "class")]
    pub fn update_class(self, class_str: String) {
        let mut classes = class_str
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>();

        // 解析并且移除 z-index 相关类
        let mut explicit_z_index = None;
        classes.retain(|class| {
            if let Some(suffix) = class.strip_prefix("z-") {
                if suffix == "auto" {
                    explicit_z_index = Some(0);
                    return false;
                } else if let Ok(z) = suffix.parse::<i32>() {
                    explicit_z_index = Some(z);
                    return false;
                }
            }
            true
        });

        if let Some(z) = explicit_z_index {
            self.set_z_index(z);
        }

        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            let mut view = view.write();

            use crate::log_error::ResultLogExt;
            use crate::view::resolver::parse_state_prefix;
            for class in &classes {
                // 解析状态前缀: "hover:class_name" -> (Hover, "class_name")
                let (control_state, actual_class) = parse_state_prefix(class);
                view.on_update_class(control_state, actual_class)
                    .error_on_err(format!("on_update_class {{ view_id:{} }}", self));
            }
        }

        let mut view_class = VIEW_STORAGE.class.write();
        if !view_class.contains_key(self) {
            view_class.insert(self, Class::new(self));
        }

        let Some(class) = view_class.get_mut(self) else {
            unreachable!()
        };
        class.load_classes(classes);

        let states = VIEW_STORAGE.states.read();
        let Some(view_state) = states.get(self) else {
            unreachable!()
        };
        let mut view_state = view_state.write();

        class.apply_layout(&mut view_state.layout_style);

        if let Some(window_id) = self.window_id() {
            window_id.entry().map(|e| e.mark_layout_dirty());
        }
        self.request_redraw();
    }

    #[inline]
    pub fn push_view(self, view: Box<dyn View + Send + Sync + 'static>) {
        VIEW_STORAGE.add_child(self, view);
        VIEW_STORAGE.reinit_child_view(self);
    }

    pub fn window_id(self) -> Option<WindowId> {
        VIEW_STORAGE.window_ids.read().get(self).copied()
    }

    /// Checks if the view is currently considered visible based on the last frame's calculation.
    ///
    /// used for culling optimizations (e.g., skipping `on_frame`).
    pub fn visual(self) -> bool {
        VIEW_STORAGE
            .visual
            .read()
            .get(self)
            .copied()
            // Fallback to `true` (Fail-Safe):
            // If the state is missing (e.g., first frame), assume visible.
            // This prevents "dead widgets" that never update because they are assumed invisible.
            .is_some()
    }

    pub fn is_scroll_view(self) -> bool {
        VIEW_STORAGE.scroll.read().get(self).copied().is_some()
    }

    pub fn register_scroll(self, scroll_state: ScrollState) {
        VIEW_STORAGE.scroll.write().insert(self, scroll_state);
    }

    /// 获取当前滚动位置 (Current Scroll Offset)
    /// 返回值: (x, y)
    /// 如果控件不可滚动，返回 (0.0, 0.0)
    pub fn scroll_offset(self) -> Option<(f32, f32)> {
        VIEW_STORAGE.scroll.read().get(self).map(|s| s.current)
    }

    /// 获取最大可滚动范围 (Max Scroll Range)
    /// 返回值: (max_x, max_y)
    /// 这对应 Taffy 计算出的 `scroll_width/height`。
    /// 比如内容宽 150，视口宽 100，这里返回 (50.0, 0.0)。
    pub fn max_scroll_offset(self) -> Option<(f32, f32)> {
        VIEW_STORAGE.scroll.read().get(self).map(|s| s.max)
    }

    /// 绝对滚动：滚动到指定位置 (x, y)
    pub fn scroll_to(self, x: f32, y: f32) {
        VIEW_STORAGE.set_scroll_internal(self, Some(x), Some(y), false);
    }

    /// 绝对滚动：仅滚动水平方向
    pub fn scroll_to_x(self, x: f32) {
        VIEW_STORAGE.set_scroll_internal(self, Some(x), None, false);
    }

    /// 绝对滚动：仅滚动垂直方向
    pub fn scroll_to_y(self, y: f32) {
        VIEW_STORAGE.set_scroll_internal(self, None, Some(y), false);
    }

    /// 相对滚动：在当前位置基础上增加 (delta_x, delta_y)
    /// 例如：scroll_by(0.0, 10.0) 向下滚 10px
    pub fn scroll_by(self, delta_x: f32, delta_y: f32) {
        VIEW_STORAGE.set_scroll_internal(self, Some(delta_x), Some(delta_y), true);
    }

    /// 快捷操作：回到顶部
    pub fn scroll_to_top(self) {
        self.scroll_to_y(0.0);
    }

    /// 快捷操作：去到底部
    pub fn scroll_to_bottom(self) {
        // 利用 internal 的钳制逻辑，传一个巨大的值即可自动吸附到底部
        self.scroll_to_y(f32::MAX);
    }

    pub fn is_hover(self) -> bool {
        if let Some(win_id) = self.window_id() {
            if let Some(entry) = win_id.entry() {
                return entry.hover_id == Some(self);
            }
        }
        false
    }

    pub fn is_active(self) -> bool {
        VIEW_STORAGE.pressed.read().get(self).is_some()
    }

    fn is_disabled(self) -> bool {
        let state_map = VIEW_STORAGE.states.read();
        let state = state_map
            .get(self)
            .expect(&format!("view[{self}] not found State"))
            .read()
            .disable;
        state
    }

    /// 有的控件部分地区需要不同的检测，提供一个语法糖方法
    pub fn control_state_with_pressed(self, pressed: bool) -> ControlState {
        if self.is_disabled() {
            return ControlState::Disabled;
        }
        if pressed {
            return ControlState::Active;
        }
        if self.is_hover() {
            return ControlState::Hover;
        }
        ControlState::Normal
    }

    /// 获取控件状态（按优先级：Disabled > Active > Focus > Hover > Normal）
    pub fn control_state(self) -> ControlState {
        if self.is_disabled() {
            return ControlState::Disabled;
        }
        if self.is_active() {
            return ControlState::Active;
        }
        let Some(window_id) = self.window_id() else {
            return ControlState::Normal;
        };
        let Some(entry) = window_id.entry() else {
            return ControlState::Normal;
        };
        if entry.focus_manager.is_focused(self) {
            return ControlState::Focus;
        }
        if entry.hover_id == Some(self) {
            return ControlState::Hover;
        }
        ControlState::Normal
    }

    pub fn update_focus_index(self, focus_index: u32) {
        if let Some(win_id) = self.window_id() {
            if let Some(mut entry) = win_id.entry_mut() {
                entry.focus_manager.update_focused(self, focus_index);
            }
        }
    }

    pub fn init_focus_scope(self, focus_scope: u32) {
        VIEW_STORAGE.focus_scope.write().insert(self, focus_scope);
    }

    pub fn init_focus_index(self, focus_index: u32) {
        VIEW_STORAGE.focus_index.write().insert(self, focus_index);
    }

    pub fn set_focus(self, virtual_index: Option<u16>) {
        if let Some(win_id) = self.window_id() {
            if let Some(mut entry) = win_id.entry_mut() {
                entry
                    .focus_manager
                    .set_focus(self, virtual_index.unwrap_or(1));
            }
        }
    }

    /// 推入焦点作用域
    ///
    /// 调用后，Tab 键只在此控件的子树内循环。
    /// 适用于 Modal Dialog、Popup、侧边栏等需要限制焦点范围的场景。
    ///
    /// # 示例
    /// ```rust
    /// // 打开 Dialog 时
    /// dialog_view_id.push_focus_scope();
    ///
    /// // 关闭时
    /// dialog_view_id.pop_focus_scope();
    /// ```
    pub fn push_focus_scope(self) {
        if let Some(win_id) = self.window_id() {
            if let Some(mut entry) = win_id.entry_mut() {
                entry.focus_manager.push_focus_scope(self);
            }
        }
    }

    /// 弹出焦点作用域
    ///
    /// 恢复到之前的焦点位置。
    pub fn pop_focus_scope(self) {
        if let Some(win_id) = self.window_id() {
            if let Some(mut entry) = win_id.entry_mut() {
                entry.focus_manager.pop_focus_scope();
            }
        }
    }

    pub fn z_index(self) -> i32 {
        VIEW_STORAGE
            .view_z_index
            .read()
            .get(self)
            .copied()
            .unwrap_or(0)
    }

    pub fn set_z_index(self, z_index: i32) {
        let mut x = VIEW_STORAGE.view_z_index.write();
        x.insert(self, z_index);
        if let Some(parent_view_id) = self.parent_view_id() {
            let mut child_ids = VIEW_STORAGE.child_ids.write();
            if let Some(childrens) = child_ids.get_mut(parent_view_id) {
                if childrens.len() > 1 {
                    childrens.sort_by(|x, d| x.z_index().cmp(&d.z_index()));
                }
            }
        }
    }

    pub fn pending_effect_id(self, effect_id: Id) {
        let mut pending_effect_id = VIEW_STORAGE.pending_effect_id.write();
        if let Some(effect_ids) = pending_effect_id.get_mut(self) {
            effect_ids.push(effect_id);
        } else {
            pending_effect_id.insert(self, vec![effect_id]);
        }
    }

    pub fn call_focus(self, virtual_index: u16) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().call_focus(virtual_index);
        }
    }

    pub fn call_blur(self, virtual_index: u16) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().call_blur(virtual_index);
        }
    }

    pub fn is_focused(self) -> bool {
        if let Some(win_id) = self.window_id() {
            if let Some(entry) = win_id.entry() {
                return entry.focus_manager.is_focused(self);
            }
        }
        false
    }

    /// 获取控件的绝对位置（相对于窗口左上角）
    /// 该值在 bus_update_layout 时计算并缓存
    pub fn abs_location(&self) -> Result<(f32, f32), Error> {
        self.with_state(|state| state.abs_location)
    }

    pub fn request_redraw(self) {
        self.window_id()
            .map(|window_id| WindowBusDispatchEntry::request_redraw(&window_id));
    }

    pub fn capture_mouse(self) -> Result<(), Error> {
        if let Some(window_id) = self.window_id() {
            window_id
                .entry_mut()
                .map(|mut entry| entry.capture_view_id = Some(self));
            window_id.capture_mouse()?;
        }
        Ok(())
    }

    pub fn release_mouse(&self) -> Result<(), Error> {
        if let Some(window_id) = self.window_id() {
            window_id
                .entry_mut()
                .map(|mut entry| entry.capture_view_id = None);
            window_id.release_mouse()?;
        }
        Ok(())
    }

    // ========================================================================
    // 声明式 Transform API
    // ========================================================================

    /// 设置控件的声明式变换
    ///
    /// 这个变换会影响控件自身及其所有子控件的绘制和命中测试。
    /// 常用于旋转、缩放等效果。
    ///
    /// # 示例
    /// ```rust
    /// // 绕中心旋转 20 度
    /// view_id.set_transform(Transform2D::rotate_at_degrees(20.0, cx, cy));
    ///
    /// // 缩放 1.5 倍
    /// view_id.set_transform(Transform2D::scale_at(1.5, 1.5, cx, cy));
    /// ```
    pub fn set_transform(self, transform: Transform2D) {
        VIEW_STORAGE.transform.write().insert(self, transform);
        self.request_redraw();
    }

    /// 获取控件的声明式变换
    ///
    /// 如果没有设置变换，返回 None
    pub fn get_transform(self) -> Option<Transform2D> {
        VIEW_STORAGE.transform.read().get(self).copied()
    }

    /// 清除控件的声明式变换
    pub fn clear_transform(self) {
        VIEW_STORAGE.transform.write().remove(self);
        self.request_redraw();
    }

    /// 把窗口坐标转换为控件局部坐标
    ///
    /// 使用 accumulated_transform 的逆变换，将鼠标的窗口坐标转换为
    /// 控件局部坐标（0,0 = 控件左上角）。
    ///
    /// 如果没有累积变换数据，返回原始坐标。
    pub fn window_to_local_position(self, mouse_pos: MousePosition) -> MousePosition {
        let accumulated_transform = VIEW_STORAGE.accumulated_transform.read();
        if let Some(transform) = accumulated_transform.get(self) {
            if let Some((local_x, local_y)) =
                transform.inverse_transform_point(mouse_pos.x as f32, mouse_pos.y as f32)
            {
                return MousePosition {
                    x: local_x as i32,
                    y: local_y as i32,
                };
            }
        }
        // 没有变换或变换不可逆，返回原始坐标
        mouse_pos
    }
}

impl LoadRenderResource for ViewId {
    fn load_image(&self, image: &[u8]) -> Result<FlorImageHandle, FlorRendererError> {
        if let Some(x) = render_from_view_id(*self) {
            let mut render = x.write();
            render.create_image_from_bytes(&image)
        } else {
            Err(FlorRendererError::RenderNotFound)
        }
    }
    fn load_raw_image(
        &self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<FlorImageHandle, FlorRendererError> {
        if let Some(x) = render_from_view_id(*self) {
            let mut render = x.write();
            render.create_image_from_raw_bytes(raw_bytes, width, height, delays)
        } else {
            Err(FlorRendererError::RenderNotFound)
        }
    }
    #[cfg(feature = "svg")]
    fn load_svg(&self, svg: &[u8]) -> Result<FlorSvgHandle, FlorRendererError> {
        if let Some(x) = render_from_view_id(*self) {
            let mut render = x.write();
            render.create_svg(svg)
        } else {
            Err(FlorRendererError::RenderNotFound)
        }
    }
}

#[cfg(feature = "drag-drop")]
impl ViewId {
    pub(crate) fn call_drag_enter(
        self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            return view
                .write()
                .call_drag_enter(key_state, mouse_position, format);
        }
        DropEffect::None
    }
    pub(crate) fn call_drag_leave(self) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            // 无返回值，保持原有风格
            view.write().call_drag_leave();
        }
    }
}
