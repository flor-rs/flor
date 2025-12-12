use crate::error::Error;
#[cfg(feature = "svg")]
use crate::render::FlorSvgHandle;
use crate::render::{FlorImageHandle, FlorRenderError, LoadRenderResource};
use crate::view::control_state::ControlState;
use crate::view::draw_state::DrawState;
use crate::view::style::layout::{Layout, LayoutKey, LayoutStateSelector};
use crate::view::view_state::ViewState;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::View;
use crate::windows::bus::render_from_view_id;
use crate::windows::entry::WindowEntryVisit;
use flor_graphics_base::{RenderContext, SurfaceId};
use once_cell::sync::Lazy;
use platform::WindowId;
use rustc_hash::FxHashMap;
use slotmap::{new_key_type, Key};
use std::any::Any;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use taffy::{LengthPercentage, Rect};

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
    pub fn layout(self) -> taffy::Layout {
        VIEW_STORAGE
            .states
            .read()
            .get(self)
            .expect(&format!("view[{self}] not found WindowId"))
            .read()
            .layout
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
        }
    }

    //     pub child_ids: RwLock<SecondaryMap<ViewId, Vec<ViewId>>>,
    pub fn push_child_ids(self, child_ids: impl IntoIterator<Item = ViewId>) {
        let mut child_ids_read = VIEW_STORAGE.child_ids.write();
        if let Some(view_ids) = child_ids_read.get_mut(self) {
            view_ids.extend(child_ids);
        } else {
            child_ids_read.insert(self, child_ids.into_iter().collect());
        }
    }

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

    pub fn is_hover(self) -> bool {
        if let Some(win_id) = self.window_id() {
            if let Some(entry) = win_id.entry() {
                return entry.active_id == Some(self);
            }
        }
        false
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

    pub fn draw_state(self) -> DrawState {
        if self.is_disabled() {
            return DrawState::Disabled;
        }
        if self.is_hover() {
            return DrawState::Hover;
        }
        DrawState::Normal
    }

    pub fn draw_state_and_pressed(self, pressed: bool) -> DrawState {
        if self.is_disabled() {
            return DrawState::Disabled;
        }
        if pressed {
            return DrawState::Pressed;
        }
        if self.is_hover() {
            return DrawState::Hover;
        }
        DrawState::Normal
    }

    pub fn control_state(self) -> ControlState {
        if self.is_disabled() {
            return ControlState::Disable;
        }
        if self.is_hover() {
            return ControlState::Hover;
        }
        // todo
        // ControlState::Active
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

    pub fn on_focus_gained(self) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().on_focus_gained();
        }
    }
    pub fn on_focus_lost(self) {
        if let Some(view) = VIEW_STORAGE.views.read().get(self) {
            view.write().on_focus_gained();
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
        raw_bytes: Vec<Vec<u8>>,
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
