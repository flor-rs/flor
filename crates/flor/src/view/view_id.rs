use crate::error::Error;
use crate::log_error::ResultLogExt;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRenderError, LoadRenderResource};
use crate::view::class::ClassLoader;
use crate::view::control_state::ControlState;
use crate::view::state_selector::{parse_state_prefix, CalcTaffyStyle, LayoutStateSelector};
use crate::view::view_state::ViewState;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use crate::windows::bus::render_from_view_id;
use crate::windows::bus_dispatch_entry::WindowBusDispatchEntry;
use crate::windows::entry::WindowEntryVisit;
use flor_graphics_base::RenderContext;
use flor_platform_base::WindowOperations;
#[cfg(feature = "drag-drop")]
use flor_platform_base::{DragFormat, DropEffect, KeyState, MousePosition};
use once_cell::sync::Lazy;
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
    pub fn new_with_layout(layout_style: LayoutStateSelector) -> ViewId {
        VIEW_STORAGE.new_view_with_state(ViewState {
            layout_style,
            ..ViewState::new()
        })
    }

    pub fn parent_view_id(self) -> Option<ViewId> {
        VIEW_STORAGE.parent_view_id.read().get(self).cloned()
    }

    //     pub states: RwLock<SecondaryMap<ViewId, RwLock<ViewState>>>,
    pub fn layout(self) -> Result<taffy::Layout, Error> {
        self.with_state(|state| state.layout)
    }

    pub fn calc_current_style(self) -> Result<taffy::Style, Error> {
        self.with_state(|view_state| {
            view_state
                .layout_style
                .calc_taffy_style(self.control_state())
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

    pub fn update_class(self, class_str: String) {
        let classes = class_str.split_whitespace().collect::<Vec<_>>();

        // 更新布局样式
        let _ = self.with_state_mut(|state| {
            state.layout_style.load_classes(classes.as_slice());
        });

        // 通知 View 更新各个 class
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            for class in classes {
                // 解析状态前缀: "hover:class_name" -> (Hover, "class_name")
                let (control_state, actual_class) = parse_state_prefix(class);
                view.write()
                    .on_update_class(control_state, actual_class)
                    .error_on_err(format!("on_update_class {{ view_id:{} }}", self));
            }
        }
    }

    //     pub child_ids: RwLock<SecondaryMap<ViewId, Vec<ViewId>>>,
    // pub fn push_child_ids(self, child_ids: impl IntoIterator<Item = ViewId>) {
    //     let mut child_ids_read = VIEW_STORAGE.child_ids.write();
    //     if let Some(view_ids) = child_ids_read.get_mut(self) {
    //         view_ids.extend(child_ids);
    //     } else {
    //         child_ids_read.insert(self, child_ids.into_iter().collect());
    //     }
    // }

    #[inline]
    pub fn push_view(self, view: Box<dyn View + Send + Sync + 'static>) {
        VIEW_STORAGE.add_child(self, view);
    }

    // pub fn set_view<State>(&self, fn_view: FnView<State>, state: Arc<State>) {
    //     let root_dyn_view = create_updater(
    //         {
    //             let state = state.clone(); // 闭包捕获 Arc
    //             move || {
    //                 // 临时借用 &T
    //                 let r = state.as_ref();
    //                 fn_view(window_id, AppRef { arc: r }) // 仅闭包内部使用
    //             }
    //         },
    //         move |view| {
    //             fn_view.update_state(Box::new(view));
    //         },
    //     );
    // }
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
            .unwrap_or(true)
    }

    pub fn is_scroll_view(self) -> bool {
        VIEW_STORAGE.scroll.read().get(self).copied().is_some()
    }

    /// 获取当前滚动位置 (Current Scroll Offset)
    /// 返回值: (x, y)
    /// 如果控件不可滚动，返回 (0.0, 0.0)
    pub fn scroll_offset(self) -> Option<(f32, f32)> {
        VIEW_STORAGE
            .scroll
            .read()
            .get(self)
            .map(|s| s.current)
    }

    /// 获取最大可滚动范围 (Max Scroll Range)
    /// 返回值: (max_x, max_y)
    /// 这对应 Taffy 计算出的 `scroll_width/height`。
    /// 比如内容宽 150，视口宽 100，这里返回 (50.0, 0.0)。
    pub fn max_scroll_offset(self) -> Option<(f32, f32)> {
        VIEW_STORAGE
            .scroll
            .read()
            .get(self)
            .map(|s| s.max)
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
            return ControlState::Disable;
        }
        if pressed {
            return ControlState::Active;
        }
        if self.is_hover() {
            return ControlState::Hover;
        }
        ControlState::Normal
    }

    pub fn control_state(self) -> ControlState {
        if self.is_disabled() {
            return ControlState::Disable;
        }
        if self.is_hover() {
            return ControlState::Hover;
        }
        if self.is_active() {
            return ControlState::Active;
        }
        if self.is_hover() {
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

    pub fn set_focus(self) {
        if let Some(win_id) = self.window_id() {
            if let Some(mut entry) = win_id.entry_mut() {
                entry.focus_manager.set_focus(self);
            }
        }
    }

    pub fn update_z_index(self, z_index: i32) {
        let _ = self.with_state_mut(|state| {
            state.z_index = z_index;
        });
        if let Some(window_id) = self.window_id() {
            VIEW_STORAGE.rebuild_render_cache(window_id)
        }
    }

    pub fn call_focus_gained(self) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().call_focus_gained();
        }
    }

    pub fn call_focus_lost(self) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().call_focus_lost();
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
}

impl LoadRenderResource for ViewId {
    fn load_image(&self, image: &[u8]) -> Result<FlorImageHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(*self) {
            let mut render = x.write();
            render.create_image_from_bytes(&image)
        } else {
            Err(FlorRenderError::RenderNotFound)
        }
    }
    fn load_raw_image(
        &self,
        raw_bytes: &Vec<Vec<u8>>,
        width: u32,
        height: u32,
        delays: Vec<u16>,
    ) -> Result<FlorImageHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(*self) {
            let mut render = x.write();
            render.create_image_from_raw_bytes(raw_bytes, width, height, delays)
        } else {
            Err(FlorRenderError::RenderNotFound)
        }
    }
    #[cfg(feature = "svg")]
    fn load_svg(&self, svg: &[u8]) -> Result<FlorSvgHandle, FlorRenderError> {
        if let Some(x) = render_from_view_id(*self) {
            let mut render = x.write();
            render.create_svg(svg)
        } else {
            Err(FlorRenderError::RenderNotFound)
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
