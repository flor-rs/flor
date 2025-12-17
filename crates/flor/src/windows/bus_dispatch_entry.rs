use crate::error::Error;
use crate::log_error::ResultLogExt;
use crate::view::style::layout::CalcTaffyStyle;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use crate::view::{collect_layout_children, View};
use crate::windows::bus::{render, render_from_view_id};
use crate::windows::entry::WindowEntryVisit;
use flor_graphics_base::RenderContext;
use flor_platform_base::MousePosition;
#[cfg(feature = "theme-change")]
use flor_platform_base::ThemeMode;
use flor_platform_base::{InputEvent, KeyCode, KeyState, WindowApi};
use log::{trace, warn};
use platform::WindowId;
use std::ops::DerefMut;
use std::time::{Duration, Instant};
use taffy::{AvailableSpace, Size, Style};

/// 总线的事件分发入口，给窗口用的
pub trait WindowBusDispatchEntry {
    // 1. 生命周期与核心循环 (Lifecycle & Core Loop)
    fn bus_create_entry(&self) -> Result<(), Error>;

    /// 初始化焦点管理器
    fn bus_init_focus_manager_entry(&mut self) -> Result<(), Error>;

    /// 帧逻辑通常在输入处理之后、渲染之前执行
    fn bus_frame_entry(&self) -> Result<Option<Duration>, Error>;

    // 2. 布局与渲染 (Layout & Rendering)
    fn bus_refresh_layout_entry(&self) -> Result<(), Error>;

    fn bus_re_draw_entry(&self) -> Result<(), Error>;

    /// 系统主题变更 (深色/浅色)
    /// 参数 theme: 当前最新的主题模式
    #[cfg(feature = "theme-change")]
    fn bus_theme_changed_entry(&mut self, theme: platform::base::ThemeMode);

    /// 工作区/显示器可用区域变更 (如任务栏移动、分辨率改变)
    /// 无参数：实现者应在收到此消息后，标记布局脏(Dirty)，并在 Layout 阶段主动查询当前显示器信息
    fn bus_work_area_changed_entry(&mut self);

    /// 鼠标滚轮设置变更
    /// 参数 lines: 系统设置的一次滚动行数 (Windows 默认为 3)
    fn bus_wheel_scroll_lines_changed_entry(&mut self, lines: u32);

    // 3. 命中测试 (Hit Testing)
    /// 交互事件的前置条件，确定事件归属
    fn bus_hit_test_entry(&self, mouse_pos: MousePosition, key_state: KeyState) -> ViewId;

    // 4. 鼠标事件 (Mouse Events)
    fn bus_mouse_move_entry(&self, key_state: KeyState, mouse_position: MousePosition);

    fn bus_mouse_leave_entry(&self);

    // ========================================================================
    // [新增] 鼠标按键事件 (Mouse Button Events)
    // 对应 LButton, RButton, MButton 的 Down, Up, DoubleClick
    // 统一使用 &mut self，因为点击通常伴随状态变更(Focus等)
    // ========================================================================

    // ---- 左键 (Left Button) ----
    fn bus_l_button_down_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_l_button_up_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_l_button_dbl_click_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);

    // ---- 右键 (Right Button) ----
    fn bus_r_button_down_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_r_button_up_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_r_button_dbl_click_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);

    // ---- 中键 (Middle Button) ----
    fn bus_m_button_down_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_m_button_up_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);
    fn bus_m_button_dbl_click_entry(&mut self, key_state: KeyState, mouse_position: MousePosition);

    // 5. 键盘事件 (Keyboard Events)
    fn bus_key_down_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool);

    fn bus_key_up_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool);

    // 6. 输入法事件 (IME Events) [New]
    fn bus_ime_start_entry(&self);

    fn bus_ime_input_entry(&self, input_event: InputEvent);

    /// 对应 WM_IME_ENDCOMPOSITION
    fn bus_ime_end_entry(&self);

    /// 封装的WindowOperations
    fn request_redraw(&self);
}

impl WindowBusDispatchEntry for WindowId {
    fn bus_create_entry(&self) -> Result<(), Error> {
        let view_id = self.view_id();
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write().bus_create()?;
        }
        Ok(())
    }

    fn bus_init_focus_manager_entry(&mut self) -> Result<(), Error> {
        let view_id = self.view_id();
        // 帮我写一下实现。
        // 遍历所有的child_ids 然后根据view_id
        let mut all_child_view_ids = Vec::new();
        let mut queue = vec![view_id];

        // 使用代码块限制 read 锁的生命周期
        {
            let child_ids_guard = VIEW_STORAGE.child_ids.read();

            while let Some(current_id) = queue.pop() {
                // 将当前节点加入列表
                all_child_view_ids.push(current_id);

                // 如果有子节点，将子节点加入队列待处理
                if let Some(children) = child_ids_guard.get(current_id) {
                    queue.extend_from_slice(children);
                }
            }
        }

        let mut focus_index = VIEW_STORAGE.focus_index.write();

        let mut focus_list = vec![];

        for child_view_id in all_child_view_ids {
            if let Some(focus_index) = focus_index.remove(child_view_id) {
                focus_list.push((focus_index, child_view_id));
            }
        }

        self.entry_mut()
            .map(|mut v| v.focus_manager.set_focus_list(focus_list));

        Ok(())
    }

    fn bus_frame_entry(&self) -> Result<Option<Duration>, Error> {
        if let Some(view) = VIEW_STORAGE.views.read().get(self.view_id()) {
            return view.write().bus_frame(Instant::now());
        }
        Ok(None)
    }

    fn bus_refresh_layout_entry(&self) -> Result<(), Error> {
        trace!("enter relayout");

        let Some(window_entry) = self.entry() else {
            warn!("window not found entry in re_layout_entry function.");
            return Ok(());
        };

        let layout_tree = &mut window_entry.taffy_tree.write();
        let view_id = window_entry.view_id;

        let states = VIEW_STORAGE.states.read();
        let Some(view_state_cell) = states.get(view_id) else {
            warn!("View storage's states not found view_id:{view_id:?}");
            return Ok(());
        };

        let view_state = view_state_cell.read();
        let old_node_id = view_state.node_id;

        let mut style_update = view_state
            .layout_style
            .calc_taffy_style(view_id.control_state());

        // 这里的特殊逻辑：如果 style 有更新，必须强制加上 100% 的尺寸限制
        if let Some(s) = &mut style_update {
            s.size = Size::from_percent(1.0, 1.0);
        }

        drop(view_state);

        let children = collect_layout_children(view_id, layout_tree)?;

        let root_node_id = match (old_node_id, style_update) {
            (Some(node_id), None) => {
                if !children.is_empty() {
                    layout_tree.set_children(node_id, &children)?;
                }
                node_id
            }
            (Some(node_id), Some(new_style)) => {
                layout_tree.set_style(node_id, new_style)?;
                if !children.is_empty() {
                    layout_tree.set_children(node_id, &children)?;
                }
                node_id
            }
            (None, style_opt) => {
                let style = style_opt.unwrap_or_else(|| Style {
                    size: Size::from_percent(1.0, 1.0),
                    ..Default::default()
                });
                if children.is_empty() {
                    layout_tree.new_leaf_with_context(style, view_id)?
                } else {
                    layout_tree.new_with_children(style, &children)?
                }
            }
        };

        if old_node_id != Some(root_node_id) {
            let mut view_state = view_state_cell.write();
            view_state.node_id = Some(root_node_id);
        }

        let client_size = self.get_client_size()?;
        layout_tree.compute_layout_with_measure(
            root_node_id,
            Size {
                height: AvailableSpace::Definite(client_size.1 as f32),
                width: AvailableSpace::Definite(client_size.0 as f32),
            },
            |known_dimensions, available_space, _node_id, node_context_view_id, style| {
                if let Some(view_id) = node_context_view_id {
                    if let Some(dyn_view) = VIEW_STORAGE.views.read().get(*view_id) {
                        if let Some(render) = render_from_view_id(*view_id).as_deref() {
                            let mut render = render.write();
                            let render = render.deref_mut();
                            let mut view = dyn_view.write();
                            return view
                                .measure(known_dimensions, available_space, style, render)
                                .unwrap_or(Size::ZERO);
                        }
                    }
                }
                Size::ZERO
            },
        )?;

        {
            trace!("bus_update_layout begin");
            if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
                view.write().bus_update_layout(layout_tree)?;
            }
            trace!("bus_update_layout end");
        }

        Ok(())
    }

    fn bus_re_draw_entry(&self) -> Result<(), Error> {
        let Some(render) = render(*self) else {
            return Ok(());
        };
        let mut render = render.write();
        render.begin().error_on_err("fail begin render");
        let view_id = self.view_id();
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .bus_draw(&mut render, (0f32, 0f32))
                .error_on_err(format!("draw({:?}) error", view_id));
        }
        render.end().error_on_err("fail end render");
        Ok(())
    }

    #[cfg(feature = "theme-change")]
    #[allow(unused_variables)]
    fn bus_theme_changed_entry(&mut self, theme: ThemeMode) {}

    fn bus_work_area_changed_entry(&mut self) {}

    fn bus_wheel_scroll_lines_changed_entry(&mut self, lines: u32) {
        let view_id = self.view_id();
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .bus_wheel_scroll_lines_changed_entry(lines)
                .error_on_err(format!("draw({:?}) error", view_id));
        }
    }

    fn bus_hit_test_entry(&self, mouse_pos: MousePosition, key_state: KeyState) -> ViewId {
        let cache_guard = VIEW_STORAGE.z_index_sort.read();

        if let Some(render_list) = cache_guard.get(self) {
            for &view_id in render_list.iter().rev() {
                if let Some(view_lock) = VIEW_STORAGE.views.read().get(view_id) {
                    if view_lock.read().on_hit_test(mouse_pos, key_state) {
                        return view_id;
                    }
                }
            }
        }
        self.view_id()
    }

    fn bus_mouse_move_entry(&self, key_state: KeyState, mouse_position: MousePosition) {
        if let Some(view_id) = self.entry().map(|v| v.capture_view_id).flatten() {
            if let Some(view) = VIEW_STORAGE.views.write().get(view_id) {
                view.write()
                    .on_mouse_leave(key_state, mouse_position)
                    .error_on_err(format!(
                        "on_mouse_leave {{ key_state: {:?}, mouse_position: {:?} }}",
                        key_state, mouse_position
                    ));
                return;
            }
        }

        // 1. 【获取新 ID】: 必定是一个有效的 ViewId (最差也是窗口自己)
        let new_hovered_id = self.bus_hit_test_entry(mouse_position, key_state);

        // 2. 【获取旧 ID】: 可能是 None (如果刚从窗口外移入)
        let old_hovered_id = self.entry().and_then(|v| v.hover_id);

        let views = VIEW_STORAGE.views.read();

        // =========================================================
        // 逻辑 A: 处理【离开】(MouseLeave)
        // 条件：之前有东西，且那个东西不是现在这个
        // =========================================================
        if let Some(old_id) = old_hovered_id {
            if old_id != new_hovered_id {
                if let Some(view_lock) = views.get(old_id) {
                    // 旧的离开
                    view_lock
                        .write()
                        .on_mouse_leave(key_state, mouse_position)
                        .error_on_err(format!(
                            "on_mouse_leave {{ key_state: {:?}, mouse_position: {:?} }}",
                            key_state, mouse_position
                        ));
                }
            }
        }

        // =========================================================
        // 逻辑 B: 处理【进入】(MouseEnter)
        // 条件：旧的是 None，或者 旧的 != 新的
        // =========================================================
        // 注意：这里不需要unwrap new_hovered_id，因为它就是 ViewId
        if old_hovered_id != Some(new_hovered_id) {
            if let Some(view_lock) = views.get(new_hovered_id) {
                // 新的进入
                view_lock
                    .write()
                    .on_mouse_enter(key_state, mouse_position)
                    .error_on_err(format!(
                        "on_mouse_enter {{ key_state: {:?}, mouse_position: {:?} }}",
                        key_state, mouse_position
                    ));
            }
        }

        // =========================================================
        // 逻辑 C: 处理【移动】(MouseMove)
        // 条件：只要在窗口内，当前命中的这个 View 就要持续收到 Move
        // =========================================================
        if let Some(view_lock) = views.get(new_hovered_id) {
            view_lock
                .write()
                .on_mouse_move(key_state, mouse_position)
                .error_on_err(format!(
                    "on_mouse_move {{ key_state: {:?}, mouse_position: {:?} }}",
                    key_state, mouse_position
                ));
        }

        // =========================================================
        // 3. 更新状态
        // =========================================================
        if let Some(mut entry) = self.entry_mut() {
            trace!("update hovered id {:?}", new_hovered_id);
            // 更新为 Some(ViewId)
            entry.hover_id = Some(new_hovered_id);
        }
    }

    fn bus_mouse_leave_entry(&self) {
        self.entry_mut().map(|mut v| {
            if v.hover_id != None {
                dbg!("call");
                v.hover_id = None;
                self.request_redraw();
            }
        });
    }

    // ==================== 左键 (Left Button) ====================

    fn bus_l_button_down_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            self.entry_mut()
                .map(|mut v| v.l_down_view_id = Some(view_id));
            view.write()
                .on_l_button_down(key_state, mouse_position)
                .error_on_err(format!("on_l_button_down {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_l_button_up_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            if let Some(spawn_click) = self.entry().map(|v| v.l_down_view_id == Some(view_id)) {
                if spawn_click {
                    view.write()
                        .on_l_button_click(key_state, mouse_position)
                        .error_on_err(format!("on_l_button_click {{ view_id:{} }}]", view_id));
                    view_id.set_focus();
                }
            }

            view.write()
                .on_l_button_up(key_state, mouse_position)
                .error_on_err(format!("on_l_button_up {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_l_button_dbl_click_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .on_l_button_dbl_click(key_state, mouse_position)
                .error_on_err(format!("on_l_button_dbl_click {{ view_id:{} }}]", view_id));
        }
    }

    // ==================== 右键 (Right Button) ====================

    fn bus_r_button_down_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            self.entry_mut()
                .map(|mut v| v.r_down_view_id = Some(view_id));
            view.write()
                .on_r_button_down(key_state, mouse_position)
                .error_on_err(format!("on_r_button_down {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_r_button_up_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            if let Some(spawn_click) = self.entry().map(|v| v.r_down_view_id == Some(view_id)) {
                if spawn_click {
                    view.write()
                        .on_r_button_click(key_state, mouse_position)
                        .error_on_err(format!("on_r_button_click {{ view_id:{} }}]", view_id));
                }
            }
            view.write()
                .on_r_button_up(key_state, mouse_position)
                .error_on_err(format!("on_r_button_up {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_r_button_dbl_click_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .on_r_button_dbl_click(key_state, mouse_position)
                .error_on_err(format!("on_r_button_dbl_click {{ view_id:{} }}]", view_id));
        }
    }

    // ==================== 中键 (Middle Button) ====================

    fn bus_m_button_down_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            self.entry_mut()
                .map(|mut v| v.m_down_view_id = Some(view_id));
            view.write()
                .on_m_button_down(key_state, mouse_position)
                .error_on_err(format!("on_m_button_down {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_m_button_up_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            if let Some(spawn_click) = self.entry().map(|v| v.m_down_view_id == Some(view_id)) {
                if spawn_click {
                    view.write()
                        .on_m_button_click(key_state, mouse_position)
                        .error_on_err(format!("on_m_button_click {{ view_id:{} }}]", view_id));
                }
            }
            view.write()
                .on_m_button_up(key_state, mouse_position)
                .error_on_err(format!("on_m_button_up {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_m_button_dbl_click_entry(&mut self, key_state: KeyState, mouse_position: MousePosition) {
        let view_id = self
            .entry()
            .map(|v| v.capture_view_id)
            .flatten()
            .unwrap_or(self.bus_hit_test_entry(mouse_position, key_state));
        if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
            view.write()
                .on_m_button_dbl_click(key_state, mouse_position)
                .error_on_err(format!("on_m_button_dbl_click {{ view_id:{} }}]", view_id));
        }
    }

    fn bus_key_down_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        let views = VIEW_STORAGE.views.read();

        if let Some(view_id) = self
            .entry()
            .and_then(|entry| entry.focus_manager.current_view_id())
        {
            if let Some(view) = views.get(view_id) {
                view.write()
                    .on_key_down(code, is_alt, is_ctrl, is_shift)
                    .error_on_err(format!(
                        "on_key_down {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
                        code, is_alt, is_ctrl, is_shift
                    ));
                return;
            }
        }

        self.on_key_down(code, is_alt, is_ctrl, is_shift)
            .error_on_err(format!(
                "on_key_down {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
                code, is_alt, is_ctrl, is_shift
            ));
    }

    fn bus_key_up_entry(&mut self, code: KeyCode, is_alt: bool, is_ctrl: bool, is_shift: bool) {
        let views = VIEW_STORAGE.views.read();

        if let Some(view_id) = self
            .entry()
            .and_then(|entry| entry.focus_manager.current_view_id())
        {
            if let Some(view) = views.get(view_id) {
                view.write()
                    .on_key_up(code, is_alt, is_ctrl, is_shift)
                    .error_on_err(format!(
                        "on_key_up {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
                        code, is_alt, is_ctrl, is_shift
                    ));
                return;
            }
        }

        self.on_key_up(code, is_alt, is_ctrl, is_shift)
            .error_on_err(format!(
                "on_key_up {{ code: {:?}, is_alt: {:?}, is_ctrl: {:?}, is_shift: {:?} }}",
                code, is_alt, is_ctrl, is_shift
            ));
    }

    fn bus_ime_start_entry(&self) {
        if let Some(view_id) = self
            .entry()
            .map(|v| v.focus_manager.current_view_id())
            .flatten()
        {
            if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
                view.write()
                    .on_ime_start()
                    .error_on_err(format!("on_ime_start {{ view_id:{} }}]", view_id));
            }
        }
    }

    fn bus_ime_input_entry(&self, input_event: InputEvent) {
        if let Some(view_id) = self
            .entry()
            .map(|v| v.focus_manager.current_view_id())
            .flatten()
        {
            if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
                view.write()
                    .on_ime_input(&input_event)
                    .error_on_err(format!("on_ime_input {{ {:?} }}]", input_event));
            }
        }
    }

    fn bus_ime_end_entry(&self) {
        if let Some(view_id) = self
            .entry()
            .map(|v| v.focus_manager.current_view_id())
            .flatten()
        {
            if let Some(view) = VIEW_STORAGE.views.read().get(view_id) {
                view.write()
                    .on_ime_end()
                    .error_on_err(format!("on_ime_end {{ view_id:{} }}]", view_id));
            }
        }
    }

    fn request_redraw(&self) {
        flor_platform_base::WindowOperations::request_redraw(self)
            .warn_on_err("request_redraw fail");
    }
}
