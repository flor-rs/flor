# ViewId 文档

`ViewId` 是 Flor 框架中每个控件的唯一标识符。它是一个轻量级的 Copy 类型，可以自由传递和存储。通过
ViewId，你可以访问控件的状态、布局信息，以及执行各种操作。

---

## 基础概念

每个实现了 `View` trait 的控件都必须持有一个 `ViewId`：

```rust
pub struct MyControl {
    view_id: ViewId,
    // ... 其他字段
}

impl View for MyControl {
    fn view_id(&self) -> ViewId {
        self.view_id
    }
    // ...
}
```

---

## 创建 ViewId

### `ViewId::new()`

创建一个基础的 ViewId：

```rust
let view_id = ViewId::new();
```

### `ViewId::new_with_layout(layout_style)`

创建带有初始布局样式的 ViewId（推荐）：

```rust
use flor::view::state_selector::{LayoutStateSelector, LayoutStateSelectorExt};
use flor::taffy::Size;

let view_id = ViewId::new_with_layout(
LayoutStateSelector::default ()
.normal()
.size(Size::auto ())
);
```

---

## 布局与位置

### `layout()` - 获取布局信息

返回 Taffy 计算出的布局结果，包含尺寸、内容尺寸等信息：

```rust
let layout = self .view_id.layout() ?;
let width = layout.size.width;
let height = layout.size.height;
```

### `abs_location()` - 获取绝对位置

返回控件相对于窗口左上角的绝对位置：

```rust
let (x, y) = self .view_id.abs_location() ?;
```

### `calc_current_style()` - 获取当前 Taffy 样式

返回根据当前状态计算出的 Taffy Style：

```rust
let taffy_style = self .view_id.calc_current_style() ?;
let overflow_x = taffy_style.overflow.x;
```

---

## 控件状态

### `control_state()` - 获取控件状态

返回当前的控件状态（Normal / Hover / Active / Disabled）：

```rust
let state = self .view_id.control_state();
match state {
ControlState::Normal => { /* 正常 */ },
ControlState::Hover => { /* 悬停 */ },
ControlState::Active => { /* 按下 */ },
ControlState::Disabled => { /* 禁用 */ },
}
```

### `control_state_with_pressed(pressed)` - 自定义按下状态

用于控件内部需要独立判断按下状态的场景：

```rust
// 例如：滚动条滑块需要独立的 pressed 状态
let state = self .view_id.control_state_with_pressed( self .is_dragging);
```

### 状态查询方法

| 方法             | 说明         |
|----------------|------------|
| `is_hover()`   | 鼠标是否悬停在控件上 |
| `is_active()`  | 控件是否被按下    |
| `is_focused()` | 控件是否获得焦点   |

---

## 滚动控制

以下方法用于可滚动控件（需要在 VIEW_STORAGE.scroll 中注册）：

### 读取滚动状态

```rust
// 当前滚动位置
let (scroll_x, scroll_y) = self .view_id.scroll_offset().unwrap_or((0.0, 0.0));

// 最大可滚动范围
let (max_x, max_y) = self .view_id.max_scroll_offset().unwrap_or((0.0, 0.0));

// 是否为可滚动视图
let scrollable = self .view_id.is_scroll_view();
```

### 设置滚动位置

```rust
// 绝对滚动
self .view_id.scroll_to(x, y);
self .view_id.scroll_to_x(x);
self .view_id.scroll_to_y(y);

// 相对滚动
self .view_id.scroll_by(delta_x, delta_y);

// 快捷方法
self .view_id.scroll_to_top();
self .view_id.scroll_to_bottom();
```

---

## 重绘与鼠标捕获

### `request_redraw()` - 请求重绘

当控件状态变化后，调用此方法触发重绘：

```rust
self .is_pressed = true;
self .view_id.request_redraw();
```

### `capture_mouse()` / `release_mouse()` - 鼠标捕获

用于拖动操作，确保鼠标移出控件后仍能收到事件：

```rust
fn on_button_down(&mut self, ...) -> Result<(), Error> {
    self.view_id.capture_mouse()?;
    self.is_dragging = true;
    Ok(())
}

fn on_button_up(&mut self, ...) -> Result<(), Error> {
    if self.is_dragging {
        self.view_id.release_mouse()?;
        self.is_dragging = false;
    }
    Ok(())
}
```

---

## 焦点管理

### 初始化焦点

```rust
// 设置焦点作用域（用于 Tab 导航分组）
self .view_id.init_focus_scope(0);

// 设置焦点顺序（Tab 键按此顺序切换）
self .view_id.init_focus_index(1);
```

### 焦点操作

```rust
// 让控件获得焦点
self .view_id.set_focus();

// 查询是否获得焦点
if self .view_id.is_focused() {
// 绘制焦点边框
}
```

---

## Z-Index 层级

控制同级控件的绘制和命中测试顺序：

```rust
// 获取当前 z-index（默认 0）
let z = self .view_id.z_index();

// 设置 z-index（值大的在上层）
self .view_id.set_z_index(100);
```

---

## Transform 变换

为控件设置 2D 变换（旋转、缩放等）：

```rust
use flor_base::types::Transform2D;

// 设置变换：绕中心旋转 20 度
let (cx, cy) = (width / 2.0, height / 2.0);
self .view_id.set_transform(Transform2D::rotate_at_degrees(20.0, cx, cy));

// 获取当前变换
if let Some(transform) = self .view_id.get_transform() {
// ...
}

// 清除变换
self .view_id.clear_transform();
```

### 坐标转换

将窗口坐标转换为控件局部坐标（考虑变换）：

```rust
fn on_click(&mut self, key_state: KeyState, mouse_pos: MousePosition) -> Result<(), Error> {
    // 如果控件有旋转/缩放变换，需要转换坐标
    let local_pos = self.view_id.window_to_local_position(mouse_pos);
    // local_pos 现在是相对于控件左上角的位置
    Ok(())
}
```

---

## 状态更新

### `update_state(state)` - 更新控件状态

用于信号系统触发控件更新：

```rust
// 外部调用
view_id.update_state(Box::new("New Title".to_string()));

// 控件内部处理
fn on_update_state(&mut self, state: Box<dyn Any>) {
    if let Ok(title) = state.downcast::<String>() {
        self.title = *title;
    }
}
```

### `update_class(class_str)` - 更新样式类

动态更新控件的样式类：

```rust
view_id.update_class("text-red-500 font-bold".to_string());
```

---

## Effect 生命周期绑定

### `pending_effect_id(effect_id)` - 绑定 Effect 到控件

当你在控件构造器中创建 effect（如通过 `create_updater_with_id`），这些 effect 需要在控件正式"激活"后才应该执行。
`pending_effect_id` 用于将 effect 注册到控件，当控件的 `on_create` 被调用时，这些 pending effect 会被统一触发。

**使用场景**：

- 控件属性绑定到信号（如 `z_index`, `class`, `transform` 等）
- 需要延迟到控件完全初始化后才执行的副作用

**示例：响应式 Z-Index**

```rust
use flor::signal::effect::updater_effect::create_updater_with_id;

impl<V: View> ZIndexBuilder for V {
    fn z_index(self, z_index: impl Fn() -> i32 + 'static) -> Self {
        let view_id = self.view_id();

        // 创建 updater 并获取 effect_id
        let (effect_id, initial_value) = create_updater_with_id(
            move || z_index(),           // 计算函数（可能读取信号）
            move |v| view_id.set_z_index(v),  // 变化回调
        );

        // 将 effect 注册到控件，等待控件激活时统一触发
        view_id.pending_effect_id(effect_id);

        // 使用初始值
        view_id.set_z_index(initial_value);

        self
    }
}
```

**工作流程**：

```
控件构造 (new)
    ↓
create_updater_with_id 创建 effect，返回 effect_id
    ↓
pending_effect_id(effect_id) 注册到控件
    ↓
控件添加到视图树
    ↓
on_create 被调用 → active_pending_effect_id 触发所有 pending effects
    ↓
effects 开始监听信号变化
```

**为什么需要这个机制？**

1. **延迟执行**：控件构造时可能还没有完全初始化（如还没有 window_id），effect 需要等到控件激活后才能正确执行。

2. **统一触发**：所有 pending effects 在 `on_create` 时统一触发，避免构造过程中的重复执行。

3. **自定义响应式属性**：通过这个机制，你可以让控件的任何属性支持信号绑定。

**自定义响应式属性示例**：

```rust
impl MyControl {
    /// 让 opacity 属性支持信号绑定
    pub fn opacity(self, opacity: impl Fn() -> f32 + 'static) -> Self {
        let view_id = self.view_id();

        let (effect_id, initial) = create_updater_with_id(
            move || opacity(),
            move |v| {
                // 更新控件内部状态
                view_id.update_state(Box::new(OpacityUpdate(v)));
            },
        );

        view_id.pending_effect_id(effect_id);

        // 可以在这里设置初始值
        self
    }
}
```

---

## 资源加载

ViewId 实现了 `LoadRenderResource` trait，可以加载图片资源：

```rust
use flor::render::LoadRenderResource;

// 加载图片
let image_handle = self .view_id.load_image( & image_bytes) ?;

// 加载 SVG（需要 "svg" feature）
let svg_handle = self .view_id.load_svg( & svg_bytes) ?;
```

---

## 访问内部状态

### `with_state()` - 只读访问

```rust
let result = self .view_id.with_state( | state| {
state.layout.size.width
}) ?;
```

### `with_state_mut()` - 可变访问

```rust
self .view_id.with_state_mut( | state| {
state.layout_style = state.layout_style.clone()
.normal()
.size(Size::auto());
}) ?;
```

---

## 常用模式

### 控件初始化

```rust
impl MyControl {
    pub fn new() -> Self {
        let view_id = ViewId::new_with_layout(
            LayoutStateSelector::default()
                .normal()
                .size(Size::auto())
        );

        Self {
            view_id,
            // ...
        }
    }
}
```

### 事件处理中请求重绘

```rust
fn on_mouse_enter(&mut self, ...) -> Result<(), Error> {
    // 状态变化后请求重绘
    self.view_id.request_redraw();
    Ok(())
}
```

### 拖动操作

```rust
fn on_button_down(&mut self, ...) -> Result<(), Error> {
    self.view_id.capture_mouse()?;
    self.drag_start = Some(mouse_position);
    Ok(())
}

fn on_mouse_move(&mut self, ...) -> Result<(), Error> {
    if let Some(start) = self.drag_start {
        // 处理拖动...
        self.view_id.request_redraw();
    }
    Ok(())
}

fn on_button_up(&mut self, ...) -> Result<(), Error> {
    if self.drag_start.is_some() {
        self.view_id.release_mouse()?;
        self.drag_start = None;
    }
    Ok(())
}
```

---

## API 速查表

| 类别            | 方法                                            | 说明                |
|---------------|-----------------------------------------------|-------------------|
| **创建**        | `new()`                                       | 创建基础 ViewId       |
|               | `new_with_layout(style)`                      | 创建带布局样式的 ViewId   |
| **布局**        | `layout()`                                    | 获取布局信息            |
|               | `abs_location()`                              | 获取绝对位置            |
|               | `calc_current_style()`                        | 获取 Taffy 样式       |
| **状态**        | `control_state()`                             | 获取控件状态            |
|               | `is_hover()` / `is_active()` / `is_focused()` | 状态查询              |
| **滚动**        | `scroll_offset()` / `max_scroll_offset()`     | 读取滚动状态            |
|               | `scroll_to()` / `scroll_by()`                 | 设置滚动位置            |
| **重绘**        | `request_redraw()`                            | 请求重绘              |
| **鼠标**        | `capture_mouse()` / `release_mouse()`         | 鼠标捕获              |
| **焦点**        | `set_focus()` / `is_focused()`                | 焦点操作              |
| **层级**        | `z_index()` / `set_z_index()`                 | Z-Index 控制        |
| **变换**        | `set_transform()` / `clear_transform()`       | 2D 变换             |
| **更新**        | `update_state()` / `update_class()`           | 状态/样式更新           |
| **Effect 绑定** | `pending_effect_id(effect_id)`                | 绑定 effect 生命周期到控件 |
