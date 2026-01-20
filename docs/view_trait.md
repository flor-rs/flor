# View Trait 文档

`View` trait 是 flor 框架中所有 UI 控件的基础特征。本文档详细介绍了所有可重写的 `on_` 开头的方法。

> **📌 所有方法都是可选的，按需重写实现。** 框架为大多数方法提供了合理的默认实现，你只需要重写你关心的方法即可。

---

## ⚠️ 重要：坐标系说明

在 flor 框架中，存在两种坐标系：

| 坐标系         | 说明       | 示例               |
|-------------|----------|------------------|
| **窗口坐标系**   | 原点在窗口左上角 | `(0, 0)` = 窗口左上角 |
| **控件局部坐标系** | 原点在控件左上角 | `(0, 0)` = 控件左上角 |

> **🔴 关键提醒**：所有 `on_` 开头的事件回调方法中，`mouse_position` 参数都是 **控件局部坐标系**，不是窗口坐标系！
>
> 这意味着：
> - `mouse_position.x == 0` 表示鼠标在控件的最左边
> - `mouse_position.y == 0` 表示鼠标在控件的最上边
> - 无论控件在窗口中的什么位置，无论控件有无旋转/缩放变换

---

## 生命周期回调

### `on_create`

```rust
fn on_create(&mut self) -> Result<(), Error>
```

**调用时机**：控件被创建并添加到视图树后，首次渲染前调用。

**参数**：无

**返回值**：`Ok(())` 表示创建成功，`Err` 会被记录到日志但不会中断流程。

**何时需要重写**：

- 需要初始化控件内部状态
- 需要加载固定资源（图片、字体等）
- 需要计算初始值

**默认实现**：空实现，无需调用 `super`。

---

## 布局与测量

### `on_measure`

```rust
fn on_measure(
    &mut self,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    style: &Style,
    render: &mut FlorRender,
) -> Result<Size<f32>, Error>
```

**调用时机**：布局计算阶段，当 Taffy 需要知道控件的固有尺寸时调用。

**参数**：

| 参数                 | 类型                     | 说明                               |
|--------------------|------------------------|----------------------------------|
| `known_dimensions` | `Size<Option<f32>>`    | 已知的尺寸约束。如果某个维度已确定（`Some`），可以直接使用 |
| `available_space`  | `Size<AvailableSpace>` | 可用空间（可能是确定值、最小值或无限）              |
| `style`            | `&Style`               | 当前计算的 Taffy 样式                   |
| `render`           | `&mut FlorRender`      | 渲染器引用，可用于测量文本尺寸等                 |

**返回值**：控件希望的尺寸 `Size<f32>`（宽度, 高度）。

**何时需要重写**：

- 文本控件需要根据内容计算尺寸
- 图片控件需要根据图片宽高比计算尺寸
- 任何需要固有尺寸的控件

**默认实现**：返回 `Size::ZERO`。

---

## 帧更新

### `on_frame`

```rust
fn on_frame(&mut self, now: Instant) -> Result<Option<Duration>, Error>
```

**调用时机**：每帧调用（仅当控件可见时）。

**参数**：

| 参数    | 类型        | 说明              |
|-------|-----------|-----------------|
| `now` | `Instant` | 当前时间戳，可用于计算动画进度 |

**返回值**：

- `Some(duration)`：告诉框架在保留模式下，下次调度此方法的**最大等待时间**。例如动画需要每 16ms 更新一帧，则返回
  `Some(Duration::from_millis(16))`。框架保证在此时间内至少调用一次。
- `None`：当前不需要定时回调，框架可以按默认策略调度。

**何时需要重写**：

- 实现动画效果（淡入淡出、位移动画等）
- 需要定时刷新内容（时钟、计时器等）
- 实现过渡效果

**默认实现**：返回 `None`。

**示例**：

```rust
fn on_frame(&mut self, now: Instant) -> Result<Option<Duration>, Error> {
    // 计算动画进度
    let elapsed = now.duration_since(self.animation_start);
    let progress = (elapsed.as_secs_f32() / self.animation_duration).min(1.0);

    self.current_opacity = self.start_opacity + (self.end_opacity - self.start_opacity) * progress;

    if progress < 1.0 {
        // 动画未完成，请求 16ms 后再次调用（约 60fps）
        Ok(Some(Duration::from_millis(16)))
    } else {
        // 动画完成，不再需要定时回调
        Ok(None)
    }
}
```

---

## 绘制回调

### `on_draw`

```rust
fn on_draw(
    &mut self,
    render: &mut FlorRender,
    abs_location: (f32, f32),
    layout: Layout,
) -> Result<(), Error>
```

**调用时机**：控件需要绘制时。

**参数**：

| 参数             | 类型                | 坐标系       | 说明                      |
|----------------|-------------------|-----------|-------------------------|
| `render`       | `&mut FlorRender` | -         | 渲染器，提供绑定各绑定图形 API 的绑定方法 |
| `abs_location` | `(f32, f32)`      | **窗口坐标系** | 控件在窗口中的绝对位置             |
| `layout`       | `Layout`          | -         | 控件的布局信息（尺寸、滚动条等）        |

> **📝 注意**：`abs_location` 使用**窗口坐标系**，这是因为渲染器的坐标系是窗口坐标系。
> 这与事件回调的 `mouse_position`（局部坐标系）不同。

**何时需要重写**：

- 需要自定义绘制内容的控件
- 几乎所有可见控件都需要重写此方法

**默认实现**：空实现，不绘制任何内容。

### `on_draw_overlay`

```rust
fn on_draw_overlay(
    &mut self,
    render: &mut FlorRender,
    abs_location: (f32, f32),
    layout: Layout,
) -> Result<(), Error>
```

**调用时机**：子控件绘制完成后调用，用于绘制覆盖层。

**参数**：同 `on_draw`。

**何时需要重写**：

- 需要绘制滚动条
- 需要绘制浮动在子控件上方的内容（如遮罩、边框装饰等）

**默认实现**：空实现。

---

## 命中测试

### `on_hit_test`

```rust
fn on_hit_test(&self, mouse_position: MousePosition, key_state: KeyState) -> bool
```

**调用时机**：判断鼠标是否命中控件的内容区域（不包含滚动条等覆盖层）。

**参数**：

| 参数               | 类型              | 坐标系            | 说明                          |
|------------------|-----------------|----------------|-----------------------------|
| `mouse_position` | `MousePosition` | **🔴 控件局部坐标系** | 鼠标位置，`(0, 0)` 为控件左上角        |
| `key_state`      | `KeyState`      | -              | 当前键盘状态（Shift/Ctrl/Alt 是否按下） |

**返回值**：`true` 表示命中，`false` 表示未命中。

**何时需要重写**：

- 非矩形控件（圆形按钮、不规则形状等）
- 需要排除某些透明区域

**默认实现**：✅ 已内置通用实现，检查鼠标是否在 `(0, 0, width, height)` 范围内。**一般无需重写**。

### `on_hit_test_overlay`

```rust
fn on_hit_test_overlay(&self, mouse_position: MousePosition, key_state: KeyState) -> bool
```

**调用时机**：判断鼠标是否命中控件的覆盖层（如滚动条）。**优先级高于 `on_hit_test`**。

**参数**：

| 参数               | 类型              | 坐标系            | 说明                   |
|------------------|-----------------|----------------|----------------------|
| `mouse_position` | `MousePosition` | **🔴 控件局部坐标系** | 鼠标位置，`(0, 0)` 为控件左上角 |
| `key_state`      | `KeyState`      | -              | 当前键盘状态               |

**返回值**：`true` 表示命中覆盖层，将拦截事件不传递给子控件。

**何时需要重写**：

- 自定义滚动条区域
- 添加自定义覆盖交互区域

**默认实现**：✅ 已内置通用实现，检查是否在滚动条区域内。**一般无需重写**。

---

## 鼠标移动事件

> **🔴 重要**：以下所有鼠标事件的 `mouse_position` 参数都是 **控件局部坐标系**。

### `on_mouse_enter`

```rust
fn on_mouse_enter(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
) -> Result<(), Error>
```

**调用时机**：鼠标从其他控件移入本控件时触发一次。

**参数**：

| 参数               | 类型              | 坐标系          | 说明                     |
|------------------|-----------------|--------------|------------------------|
| `key_state`      | `KeyState`      | -            | 当前键盘状态（Shift/Ctrl/Alt） |
| `mouse_position` | `MousePosition` | **🔴 局部坐标系** | 进入点的位置                 |

**何时需要重写**：实现 hover 效果、显示 tooltip 等。

**默认实现**：空实现。

### `on_mouse_move`

```rust
fn on_mouse_move(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
) -> Result<(), Error>
```

**调用时机**：鼠标在控件内移动时持续触发。

**参数**：同 `on_mouse_enter`。

**何时需要重写**：实现拖动操作、实时跟踪鼠标位置等。

**默认实现**：空实现。

### `on_mouse_leave`

```rust
fn on_mouse_leave(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
) -> Result<(), Error>
```

**调用时机**：鼠标移出本控件时触发一次。

**参数**：同 `on_mouse_enter`。

**何时需要重写**：取消 hover 效果、隐藏 tooltip 等。

**默认实现**：空实现。

---

## 鼠标按键事件

> **🔴 重要**：以下所有鼠标事件的 `mouse_position` 参数都是 **控件局部坐标系**。

### 事件总览

| 事件类型 | 左键                | 右键                             | 中键                              |
|------|-------------------|--------------------------------|---------------------------------|
| 按下   | `on_button_down`  | `on_right_button_down`         | `on_middle_button_down`         |
| 释放   | `on_button_up`    | `on_right_button_up`           | `on_middle_button_up`           |
| 点击   | `on_click`        | `on_right_button_click`        | `on_middle_button_click`        |
| 双击   | `on_double_click` | `on_right_button_double_click` | `on_middle_button_double_click` |

### 方法签名

所有按键事件的签名相同：

```rust
fn on_xxx(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
) -> Result<(), Error>
```

**参数**：

| 参数               | 类型              | 坐标系          | 说明                          |
|------------------|-----------------|--------------|-----------------------------|
| `key_state`      | `KeyState`      | -            | 当前键盘状态（Shift/Ctrl/Alt 是否按下） |
| `mouse_position` | `MousePosition` | **🔴 局部坐标系** | 事件发生时的鼠标位置                  |

**注意事项**：

- `on_click` 是合成事件：仅当按下和释放发生在同一控件时触发
- `on_double_click` 由系统判定双击间隔

**何时需要重写**：

- `on_button_down`：开始拖动操作、显示按下效果
- `on_button_up`：结束拖动操作
- `on_click`：响应点击事件（最常用）
- `on_right_button_click`：显示上下文菜单

**默认实现**：空实现。

### 示例

```rust
impl View for MyButton {
    // ... view_id 等必需方法

    fn on_button_down(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        // 记录按下状态，用于显示按下效果
        self.is_pressed = true;
        self.view_id().request_redraw();
        Ok(())
    }

    fn on_button_up(
        &mut self,
        _key_state: KeyState,
        _mouse_position: MousePosition,
    ) -> Result<(), Error> {
        self.is_pressed = false;
        self.view_id().request_redraw();
        Ok(())
    }

    fn on_click(
        &mut self,
        key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        // 执行点击操作
        println!("Button clicked at local position: ({}, {})",
                 mouse_position.x, mouse_position.y);

        // 如果按住 Ctrl 键点击，执行特殊操作
        if key_state.ctrl {
            self.do_ctrl_click_action();
        } else {
            self.do_normal_click_action();
        }
        Ok(())
    }

    fn on_right_button_click(
        &mut self,
        _key_state: KeyState,
        mouse_position: MousePosition,
    ) -> Result<(), Error> {
        // 显示上下文菜单
        self.show_context_menu(mouse_position);
        Ok(())
    }
}
```

---

## 键盘事件

### `on_key_down`

```rust
fn on_key_down(
    &mut self,
    code: KeyCode,
    is_alt: bool,
    is_ctrl: bool,
    is_shift: bool,
) -> Result<(), Error>
```

**调用时机**：键盘按键按下（仅当控件获得焦点时）。

**参数**：

| 参数         | 类型        | 说明           |
|------------|-----------|--------------|
| `code`     | `KeyCode` | 按下的按键代码      |
| `is_alt`   | `bool`    | Alt 键是否被按下   |
| `is_ctrl`  | `bool`    | Ctrl 键是否被按下  |
| `is_shift` | `bool`    | Shift 键是否被按下 |

**何时需要重写**：实现快捷键、文本输入、游戏控制等。

**默认实现**：空实现。

### `on_key_up`

```rust
fn on_key_up(
    &mut self,
    code: KeyCode,
    is_alt: bool,
    is_ctrl: bool,
    is_shift: bool,
) -> Result<(), Error>
```

**调用时机**：键盘按键释放（仅当控件获得焦点时）。

**参数**：同 `on_key_down`。

**何时需要重写**：需要检测按键释放的场景（游戏、长按功能等）。

**默认实现**：空实现。

---

## 焦点事件

### `on_focus_gained`

```rust
fn on_focus_gained(&mut self) -> Result<(), Error>
```

**调用时机**：控件获得焦点时。

**参数**：无

**何时需要重写**：显示焦点边框、启动光标闪烁等。

**默认实现**：空实现。

### `on_focus_lost`

```rust
fn on_focus_lost(&mut self) -> Result<(), Error>
```

**调用时机**：控件失去焦点时。

**参数**：无

**何时需要重写**：隐藏焦点边框、停止光标闪烁、提交输入内容等。

**默认实现**：空实现。

---

## 输入法事件 (IME)

### `on_ime_start`

```rust
fn on_ime_start(&mut self) -> Result<(), Error>
```

**调用时机**：输入法开始组合输入。

**参数**：无

**何时需要重写**：显示输入法组合窗口、准备接收输入。

**默认实现**：空实现。

### `on_ime_input`

```rust
fn on_ime_input(&mut self, input_event: &InputEvent) -> Result<(), Error>
```

**调用时机**：输入法输入内容时。

**参数**：

| 参数            | 类型            | 说明             |
|---------------|---------------|----------------|
| `input_event` | `&InputEvent` | 输入事件，包含输入的文本内容 |

**何时需要重写**：文本输入控件需要处理输入内容。

**默认实现**：空实现。

### `on_ime_end`

```rust
fn on_ime_end(&mut self) -> Result<(), Error>
```

**调用时机**：输入法结束组合输入。

**参数**：无

**何时需要重写**：关闭输入法组合窗口。

**默认实现**：空实现。

---

## 拖放事件

> **📝 注意**：需启用 `drag-drop` feature。

### `on_drag_enter`

```rust
fn on_drag_enter(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
    format: &[DragFormat],
) -> Result<DropEffect, Error>
```

**调用时机**：拖拽操作进入控件区域。

**参数**：

| 参数               | 类型              | 坐标系          | 说明          |
|------------------|-----------------|--------------|-------------|
| `key_state`      | `KeyState`      | -            | 当前键盘状态      |
| `mouse_position` | `MousePosition` | **🔴 局部坐标系** | 鼠标位置        |
| `format`         | `&[DragFormat]` | -            | 拖拽数据支持的格式列表 |

**返回值**：允许的放置效果（`Copy`/`Move`/`Link`/`None`）。

**默认实现**：返回 `DropEffect::None`。

### `on_drag_over`

```rust
fn on_drag_over(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
    format: &[DragFormat],
) -> Result<DropEffect, Error>
```

**调用时机**：拖拽操作在控件内移动。

**参数**：同 `on_drag_enter`。

**返回值**：允许的放置效果。

**默认实现**：返回 `DropEffect::None`。

### `on_drag_leave`

```rust
fn on_drag_leave(&mut self) -> Result<(), Error>
```

**调用时机**：拖拽操作离开控件区域。

**参数**：无

**默认实现**：空实现。

### `on_drop`

```rust
fn on_drop(
    &mut self,
    key_state: KeyState,
    mouse_position: MousePosition,
    data: &DragData,
) -> Result<DropEffect, Error>
```

**调用时机**：在控件上释放拖拽内容。

**参数**：

| 参数               | 类型              | 坐标系          | 说明       |
|------------------|-----------------|--------------|----------|
| `key_state`      | `KeyState`      | -            | 当前键盘状态   |
| `mouse_position` | `MousePosition` | **🔴 局部坐标系** | 释放时的鼠标位置 |
| `data`           | `&DragData`     | -            | 拖拽的实际数据  |

**返回值**：实际执行的放置效果。

**默认实现**：返回 `DropEffect::None`。

---

## 滚动事件

### `on_wheel_scroll_lines_changed`

```rust
fn on_wheel_scroll_lines_changed(
    &mut self,
    axis: ScrollAxis,
    delta: f32,
    key_state: KeyState,
    mouse_position: MousePosition,
) -> Result<(), Error>
```

**调用时机**：鼠标滚轮滚动。

**参数**：

| 参数               | 类型              | 坐标系          | 说明                                     |
|------------------|-----------------|--------------|----------------------------------------|
| `axis`           | `ScrollAxis`    | -            | 滚动轴方向（水平 `Horizontal` / 垂直 `Vertical`） |
| `delta`          | `f32`           | -            | 滚动量（行数），正值向下/向右，负值向上/向左                |
| `key_state`      | `KeyState`      | -            | 当前键盘状态（可用于判断是否按住 Shift 切换方向）           |
| `mouse_position` | `MousePosition` | **🔴 局部坐标系** | 滚动时鼠标在控件内的位置                           |

**何时需要重写**：需要处理滚动内容时。

**默认实现**：空实现。

---

## 子控件事件

### `on_child_push`

```rust
fn on_child_push(&mut self) -> Result<(), Error>
```

**调用时机**：有新的子控件被添加时。

**参数**：无

**何时需要重写**：需要响应子控件变化的容器控件。

**默认实现**：空实现。

### `on_child_dispose`

```rust
fn on_child_dispose(&mut self) -> Result<(), Error>
```

**调用时机**：有子控件被移除时。

**参数**：无

**何时需要重写**：需要响应子控件变化的容器控件。

**默认实现**：空实现。

---

## 状态更新

### `on_update_state`

```rust
fn on_update_state(&mut self, state: Box<dyn Any>)
```

**调用时机**：通过 `ViewId::update_state()` 外部更新控件状态时。一般配合信号机制使用。

**参数**：

| 参数      | 类型             | 说明                |
|---------|----------------|-------------------|
| `state` | `Box<dyn Any>` | 新状态数据，需要向下转型为具体类型 |

**何时需要重写**：需要支持外部状态更新的控件。

**默认实现**：空实现。

### `on_update_class`

```rust
fn on_update_class(
    &mut self,
    control_state: ControlState,
    class_name: &str,
) -> Result<(), Error>
```

**调用时机**：通过 `ViewId::update_class()` 更新控件样式类时。

**参数**：

| 参数              | 类型             | 说明                                 |
|-----------------|----------------|------------------------------------|
| `control_state` | `ControlState` | 状态前缀（Normal/Hover/Active/Disabled） |
| `class_name`    | `&str`         | 样式类名                               |

**何时需要重写**：需要响应样式类变化的控件。

**默认实现**：空实现。

---

## 最佳实践

1. **坐标系意识**：始终记住鼠标事件使用**局部坐标系**，绘制使用**窗口坐标系**
2. **按需实现**：只重写你需要的方法，利用好默认实现
3. **避免阻塞**：事件回调应快速返回，耗时操作应异步处理
4. **错误处理**：返回 `Err` 不会中断程序，但会被记录到日志
5. **请求重绘**：状态变化后调用 `self.view_id().request_redraw()` 触发重绘

