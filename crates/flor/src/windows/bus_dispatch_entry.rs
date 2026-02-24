mod button_down_entry;
mod button_up_entry;
mod create_entry;
mod double_click_entry;
mod draw_entry;
mod frame_entry;
mod hit_test_entry;
mod ime_end_entry;
mod ime_input_entry;
mod ime_start_entry;
mod init_focus_manager_entry;
mod key_down_entry;
mod key_up_entry;
mod middle_button_double_click_entry;
mod middle_button_down_entry;
mod middle_button_up_entry;
mod mouse_leave_entry;
mod mouse_move_entry;
mod refresh_layout_entry;
mod right_button_double_click_entry;
mod right_button_down_entry;
mod right_button_up_entry;
mod tooltip_check_entry;
mod wheel_scroll_lines_changed_entry;

use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::view::view_id::ViewId;
use crate::windows::bus::render;
use crate::windows::bus_dispatch_entry::button_down_entry::button_down_entry;
use crate::windows::bus_dispatch_entry::button_up_entry::button_up_entry;
use crate::windows::bus_dispatch_entry::create_entry::create_entry;
use crate::windows::bus_dispatch_entry::double_click_entry::double_click_entry;
use crate::windows::bus_dispatch_entry::draw_entry::draw_entry;
use crate::windows::bus_dispatch_entry::frame_entry::frame_entry;
use crate::windows::bus_dispatch_entry::hit_test_entry::hit_test_entry;
use crate::windows::bus_dispatch_entry::ime_end_entry::ime_end_entry;
use crate::windows::bus_dispatch_entry::ime_input_entry::ime_input_entry;
use crate::windows::bus_dispatch_entry::ime_start_entry::ime_start_entry;
use crate::windows::bus_dispatch_entry::init_focus_manager_entry::init_focus_manager_entry;
use crate::windows::bus_dispatch_entry::key_down_entry::key_down_entry;
use crate::windows::bus_dispatch_entry::key_up_entry::key_up_entry;
use crate::windows::bus_dispatch_entry::middle_button_double_click_entry::middle_button_double_click_entry;
use crate::windows::bus_dispatch_entry::middle_button_down_entry::middle_button_down_entry;
use crate::windows::bus_dispatch_entry::middle_button_up_entry::middle_button_up_entry;
use crate::windows::bus_dispatch_entry::mouse_leave_entry::mouse_leave_entry;
use crate::windows::bus_dispatch_entry::mouse_move_entry::mouse_move_entry;
use crate::windows::bus_dispatch_entry::refresh_layout_entry::refresh_layout_entry;
use crate::windows::bus_dispatch_entry::right_button_double_click_entry::right_button_double_click_entry;
use crate::windows::bus_dispatch_entry::right_button_down_entry::right_button_down_entry;
use crate::windows::bus_dispatch_entry::right_button_up_entry::right_button_up_entry;
use crate::windows::bus_dispatch_entry::tooltip_check_entry::tooltip_check_entry;
use crate::windows::bus_dispatch_entry::wheel_scroll_lines_changed_entry::wheel_scroll_lines_changed_entry;
use crate::windows::entry::WindowEntryVisit;
use flor_base::platform::{InputEvent, KeyCode, KeyState};
use flor_base::platform::{MousePosition, ScrollAxis};
#[cfg(feature = "theme-change")]
use flor_platform_base::ThemeMode;
#[cfg(feature = "drag-drop")]
use flor_platform_base::{DragData, DragFormat, DropEffect};
use graphics::base::RenderContext;
use platform::WindowId;
use std::ops::DerefMut;
use std::sync::atomic::Ordering;
use std::time::Duration;

/// 总线的事件分发入口，给窗口用的
pub trait WindowBusDispatchEntry {
    // 1. 生命周期与核心循环 (Lifecycle & Core Loop)
    fn bus_create_entry(self) -> Result<(), Error>;

    /// 初始化焦点管理器
    fn bus_init_focus_manager_entry(self) -> Result<(), Error>;

    /// 帧逻辑通常在输入处理之后、渲染之前执行
    fn bus_frame_entry(self) -> Result<Option<Duration>, Error>;

    /// 检查 tooltip 悬停计时器，超时则派发 show 事件
    /// 返回值：如果正在等待 tooltip 超时，返回剩余等待时间
    fn bus_tooltip_check_entry(self) -> Option<Duration>;

    // 2. 布局与渲染 (Layout & Rendering)
    fn bus_refresh_layout_entry(self) -> Result<(), Error>;

    fn bus_re_draw_entry(self) -> Result<(), Error>;

    /// 系统主题变更 (深色/浅色)
    /// 参数 theme: 当前最新的主题模式
    #[cfg(feature = "theme-change")]
    fn bus_theme_changed_entry(&mut self, theme: ThemeMode);

    /// 工作区/显示器可用区域变更 (如任务栏移动、分辨率改变)
    /// 无参数：实现者应在收到此消息后，标记布局脏(Dirty)，并在 Layout 阶段主动查询当前显示器信息
    fn bus_work_area_changed_entry(&mut self);

    /// 鼠标滚轮设置变更
    fn bus_wheel_scroll_lines_changed_entry(
        self,
        axis: ScrollAxis,
        delta: f32,
        key_state: KeyState,
        mouse_position: MousePosition,
    );

    // 3. 命中测试 (Hit Testing)
    /// 交互事件的前置条件，确定事件归属
    fn bus_hit_test_entry(self, mouse_pos: MousePosition, key_state: KeyState) -> ViewId;

    // 4. 鼠标事件 (Mouse Events)
    fn bus_mouse_move_entry(self, key_state: KeyState, mouse_position: MousePosition);

    fn bus_mouse_leave_entry(self);

    // ========================================================================
    // [新增] 鼠标按键事件 (Mouse Button Events)
    // 对应 LButton, RButton, MButton 的 Down, Up, DoubleClick
    // 统一使用 &mut self，因为点击通常伴随状态变更(Focus等)
    // ========================================================================

    // ---- 左键 (Left Button) ----
    fn bus_button_down_entry(self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_button_up_entry(self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_double_click_entry(self, key_state: KeyState, mouse_position: MousePosition);

    // ---- 右键 (Right Button) ----
    fn bus_right_button_down_entry(self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_right_button_up_entry(self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_right_button_double_click_entry(
        self,
        key_state: KeyState,
        mouse_position: MousePosition,
    );

    // ---- 中键 (Middle Button) ----
    fn bus_middle_button_down_entry(self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_middle_button_up_entry(self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_middle_button_double_click_entry(
        self,
        key_state: KeyState,
        mouse_position: MousePosition,
    );

    // 5. 键盘事件 (Keyboard Events)
    fn bus_key_down_entry(self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool);

    fn bus_key_up_entry(self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool);

    // 6. 输入法事件 (IME Events) [New]
    fn bus_ime_start_entry(self);

    fn bus_ime_input_entry(self, input_event: InputEvent);

    /// 对应 WM_IME_ENDCOMPOSITION
    fn bus_ime_end_entry(self);

    // 7. 拖放事件 (Drag & Drop Events)
    // 对应系统层的 DragEnter, DragOver, DragLeave, Drop
    // 注意：Enter 和 Over 通常需要返回 DragOperation (None/Copy/Move/Link) 以更新系统光标样式

    /// 拖拽进入窗口区域
    #[cfg(feature = "drag-drop")]
    fn bus_drag_enter_entry(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect;

    /// 拖拽在窗口内移动 (高频触发)
    #[cfg(feature = "drag-drop")]
    fn bus_drag_over_entry(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect;

    /// 拖拽离开窗口区域或被取消
    #[cfg(feature = "drag-drop")]
    fn bus_drag_leave_entry(&mut self);

    /// 并在有效区域释放鼠标
    #[cfg(feature = "drag-drop")]
    fn bus_drop_entry(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        data: &DragData,
    ) -> DropEffect;

    /// 封装的WindowOperations
    fn request_redraw(&self);

    fn update_child_layout_dpi(&self, dpi_x: f32, dpi_y: f32);
}

impl WindowBusDispatchEntry for WindowId {
    fn bus_create_entry(self) -> Result<(), Error> {
        create_entry(self)
    }

    fn bus_init_focus_manager_entry(self) -> Result<(), Error> {
        init_focus_manager_entry(self)
    }

    fn bus_frame_entry(self) -> Result<Option<Duration>, Error> {
        frame_entry(self)
    }

    fn bus_tooltip_check_entry(self) -> Option<Duration> {
        tooltip_check_entry(self)
    }

    fn bus_refresh_layout_entry(self) -> Result<(), Error> {
        refresh_layout_entry(self)
    }

    fn bus_re_draw_entry(self) -> Result<(), Error> {
        let Some(render) = render(self) else {
            return Ok(());
        };
        let mut render = render.write();
        render.begin().error_on_err("fail begin render");
        draw_entry(self, render.deref_mut())?;
        render.end().error_on_err("fail end render");
        Ok(())
    }

    #[cfg(feature = "theme-change")]
    #[allow(unused_variables)]
    fn bus_theme_changed_entry(&mut self, theme: ThemeMode) {}

    fn bus_work_area_changed_entry(&mut self) {}

    fn bus_wheel_scroll_lines_changed_entry(
        self,
        axis: ScrollAxis,
        delta: f32,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) {
        wheel_scroll_lines_changed_entry(self, axis, delta, key_state, mouse_position);
    }

    fn bus_hit_test_entry(self, mouse_pos: MousePosition, key_state: KeyState) -> ViewId {
        hit_test_entry(self, mouse_pos, key_state)
    }

    fn bus_mouse_move_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        mouse_move_entry(self, key_state, mouse_position)
    }

    fn bus_mouse_leave_entry(self) {
        mouse_leave_entry(self)
    }

    // ==================== 左键 (Left Button) ====================

    fn bus_button_down_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        button_down_entry(self, key_state, mouse_position)
    }

    fn bus_button_up_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        button_up_entry(self, key_state, mouse_position)
    }

    fn bus_double_click_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        double_click_entry(self, key_state, mouse_position)
    }

    // ==================== 右键 (Right Button) ====================

    fn bus_right_button_down_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        right_button_down_entry(self, key_state, mouse_position)
    }

    fn bus_right_button_up_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        right_button_up_entry(self, key_state, mouse_position)
    }

    fn bus_right_button_double_click_entry(
        self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) {
        right_button_double_click_entry(self, key_state, mouse_position)
    }

    // ==================== 中键 (Middle Button) ====================

    fn bus_middle_button_down_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        middle_button_down_entry(self, key_state, mouse_position)
    }

    fn bus_middle_button_up_entry(self, key_state: KeyState, mouse_position: MousePosition) {
        middle_button_up_entry(self, key_state, mouse_position)
    }

    fn bus_middle_button_double_click_entry(
        self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) {
        middle_button_double_click_entry(self, key_state, mouse_position)
    }

    fn bus_key_down_entry(self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        key_down_entry(self, code, is_alt, is_ctrl, is_shift)
    }

    fn bus_key_up_entry(self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        key_up_entry(self, code, is_alt, is_ctrl, is_shift)
    }

    fn bus_ime_start_entry(self) {
        ime_start_entry(self)
    }

    fn bus_ime_input_entry(self, input_event: InputEvent) {
        ime_input_entry(self, input_event)
    }

    fn bus_ime_end_entry(self) {
        ime_end_entry(self)
    }

    #[cfg(feature = "drag-drop")]
    // 1. [DragEnter] 鼠标首次进入窗口
    fn bus_drag_enter_entry(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect {
        // 1. 命中测试
        let target_id = self.bus_hit_test_entry(mouse_position, key_state);

        // 2. 更新状态：记录当前 ViewId
        if let Some(mut entry) = self.entry_mut() {
            entry.current_drag_target = Some(target_id);
        }

        // 3. 投递给目标控件
        target_id.call_drag_enter(key_state, mouse_position, format)
    }

    // 2. [DragOver] 核心状态机：负责分发 Enter/Leave/Over
    #[cfg(feature = "drag-drop")]
    fn bus_drag_over_entry(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        format: &[DragFormat],
    ) -> DropEffect {
        // 1. 计算新目标
        let new_target_id = self.bus_hit_test_entry(mouse_position, key_state);

        // 2. 获取旧目标
        let old_target_id = self.entry().and_then(|v| v.current_drag_target);

        // 3. 状态判断
        if Some(new_target_id) != old_target_id {
            // Case A: 目标切换 (A -> B)

            // A-1: 通知旧控件 Leave
            if let Some(old_id) = old_target_id {
                if let Some(view) = VIEW_STORAGE.views.read().get(old_id) {
                    view.write().call_drag_leave();
                }
            }

            // A-2: 更新状态
            if let Some(mut entry) = self.entry_mut() {
                entry.current_drag_target = Some(new_target_id);
            }

            // A-3: 通知新控件 Enter (合成事件)
            new_target_id.call_drag_enter(key_state, mouse_position, format)
        } else {
            // Case B: 目标未变 (在 A 内部移动)

            // 直接通知 Over
            if let Some(view) = VIEW_STORAGE.views.read().get(new_target_id) {
                return view
                    .write()
                    .call_drag_over(key_state, mouse_position, format);
            }
            DropEffect::None
        }
    }

    // 3. [DragLeave] 鼠标离开窗口
    #[cfg(feature = "drag-drop")]
    fn bus_drag_leave_entry(&mut self) {
        // 获取旧目标
        let old_target_id = self.entry().and_then(|v| v.current_drag_target);

        // 1. 通知 Leave
        if let Some(old_id) = old_target_id {
            old_id.call_drag_leave();
        }

        // 2. 清理状态
        if let Some(mut entry) = self.entry_mut() {
            entry.current_drag_target = None;
        }
    }

    // 4. [Drop] 鼠标松开
    #[cfg(feature = "drag-drop")]
    fn bus_drop_entry(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
        data: &DragData,
    ) -> DropEffect {
        // 1. 再次命中测试
        let target_id = self.bus_hit_test_entry(mouse_position, key_state);

        // 2. 清理状态
        if let Some(mut entry) = self.entry_mut() {
            entry.current_drag_target = None;
        }

        // 3. 投递 Drop
        if let Some(view) = VIEW_STORAGE.views.read().get(target_id) {
            return view.write().call_drop(key_state, mouse_position, data);
        }

        DropEffect::None
    }

    fn request_redraw(&self) {
        flor_base::platform::WindowOperations::request_redraw(self)
            .warn_on_err("request_redraw fail");
    }

    fn update_child_layout_dpi(&self, dpi_x: f32, dpi_y: f32) {
        if let Some(entry) = self.entry() {
            entry.unit.load().dpi_x.store(dpi_x, Ordering::Relaxed);
            entry.unit.load().dpi_y.store(dpi_y, Ordering::Relaxed);
        }
    }
}
