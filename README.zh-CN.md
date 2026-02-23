[English](README.md) | **中文**

# Flor

一个现代、易用、轻量、原生的 Rust GUI 框架。

Flor 是一个从零构建的控件式 GUI 框架 —— 图形层、平台层、布局引擎集成、信号系统 —— 全部自研实现，不依赖 winit、wgpu
等重型库。设计初衷是创造一个作者自己理想中的 Rust GUI 框架：**小体积、高性能、易用的 API、零设计屏障**。

> *Flor* 是 *flower* 的法语拼写。这个名字源于作者曾经计划开发的一个工具应用"万花姬"。随着其他计划开发的工具承担了它的部分职责，剩下的
> GUI 核心部分获得了新生，成为了这个框架，继承了"万花"之名。

---

## 目录

- [设计动机](#设计动机)
- [设计目标](#设计目标)
- [Flor 的独特之处](#flor-的独特之处)
    - [🔄 响应式信号系统](#-响应式信号系统)
    - [🪟 灵活的多窗口](#-灵活的多窗口)
    - [⚡ 保留模式 + 即时模式](#-保留模式--即时模式)
    - [� 动画 = 状态函数](#-动画--状态函数)
    - [�🎨 控件式框架 + Builder 模式](#-控件式框架--builder-模式)
    - [📐 CSS 风格布局](#-css-风格布局)
    - [🧰 View 特征](#-view-特征)
    - [🪶 极简依赖](#-极简依赖)
- [快速开始](#快速开始)
- [架构](#架构)
- [Feature Flags](#feature-flags)
- [关于控件库](#关于控件库)
- [目标用户](#目标用户)
- [核心模块](#核心模块)
- [文档](#文档)
- [License](#license)

---

## 设计动机

Rust 的所有权模型为 GUI 开发带来了独特的挑战。现有的大多数框架或多或少存在以下问题：

- **生命周期导致的设计屏障** — 跨控件树访问和修改状态，就是在和借用检查器作斗争
- **窗口与上下文耦合** — 把窗口绑定到全局上下文，导致多窗口工作流的使用体验极其糟糕
- **重型依赖** — 依赖大型抽象层（如 winit、wgpu 等）为一个简单的工具拉入了庞大的依赖树

Flor 诞生于尝试了一个又一个框架后依然找不到满意方案的挫败感。受到 miniquad 简洁的源码组织方式的鼓舞、Floem 信号系统的启发、以及
FreeXpand_DUI 灵活事件派发思想的影响 —— 历经五个大版本的设计迭代，最终形成了基于响应式信号的架构，从根本上解决了 Rust 在
GUI 领域的所有权矛盾。

> **设计屏障是什么？**
>
> 1. Rust 的生命周期特性导致无法像面向对象语言那样自由地访问控件、处理交互
> 2. 很多框架把上下文与窗口绑定，或把窗口约束到全局上下文内，窗口操作体验极差
> 3. 没有灵活的多窗口支持，对工具型应用来说难以接受

## 设计目标

| 目标             |                                     |
|----------------|-------------------------------------|
| 🏎️ **性能**     | 默认保留模式，按需即时模式。帧精度唤醒机制。              |
| 📦 **小体积**     | 自研图形层和平台层，不依赖重型抽象 crate。            |
| 🧩 **易用性**     | 信号驱动的响应式机制，消除手动刷新传播。                |
| 🔧 **完善的 API** | 托盘、剪贴板、拖放、多显示器、Hi-DPI、主题检测 —— 开箱即用。 |

## Flor 的独特之处

如果你正在评估 Rust GUI 技术方案，以下是 Flor 带来的、在其他框架中很难找到的特性 —— 它们填补了 Rust GUI 生态的真实缺口：

- **自研图形层和平台层** —— 不依赖重型抽象 crate（如 winit、wgpu 等）。依赖极少，二进制体积小，编译快。
- **响应式信号系统，O(1) 更新，线程安全** —— 信号可从任意线程读写，无需手动刷新 UI。
- **每个窗口独立的保留/即时模式** —— 每个窗口独立选择渲染策略，同一应用中可以混合使用。
- **基于即时模式的保留模式** —— 即使在保留模式下也天然支持高性能动画，通过帧精度唤醒调度实现。
- **随处创建和控制窗口** —— 无全局上下文，无生命周期体操。在按钮点击回调中、在另一个线程中、在任何你需要的地方开窗口。
- **原生句柄暴露** —— 可获取平台窗口原生句柄，方便与 native API 和第三方库互操作。
- **低下限、高上限** —— View trait 的一致性 API 意味着即使是最简单的控件也能获得框架完整的生命周期管理、命中测试、焦点、工具提示和事件派发。而同样的系统也能支撑复杂的自定义渲染控件。
- **为团队和长期项目而设计** —— 信号驱动的数据流天然可追踪。增删数据源只需更新信号，不需要到处清理分散的手动刷新逻辑。
- **框架与控件分离** —— 核心框架不包含任何控件。控件库是独立的 crate。你甚至可以把简单的小游戏（贪吃蛇、俄罗斯方块、棋类）当作
  `View` 实现来写。
- **类似 TailwindCSS 的工具类** —— 支持用 `"flex flex-col gap-2 p-4"` 等熟悉的类名来定义布局与样式，让从 Web
  开发转来的开发者倍感亲切。

> Flor 的目标是成为 Rust 生态中的**默认 GUI 框架** —— 当开发者需要一个原生桌面应用时，本能地伸手去拿的那个框架，就像 C++ 的
> Qt、C# 的 WPF 那样。

### 🔄 响应式信号系统

O(1) 更新传播。线程安全。无需手动刷新调用。

```rust
use flor::signal::rw_signal::create_signal;
use flor::signal::read::Read;
use flor::signal::write::Write;

let (reader, writer) = create_signal("Hello".to_string()).split();

// Writer 可以 move 到其他线程
writer.set("World".to_string());

// Reader 自动追踪依赖
let text = reader.get(); // "World"
```

### 🪟 灵活的多窗口

随处创建窗口 — 无需全局上下文。每个窗口独立选择保留模式或即时模式。

```rust
use flor::windows::window_options::WindowOption;

let _window = WindowOption {
    title: "My App".to_string(),
    width: 1280,
    height: 800,
    continuous_rendering: false,   // 保留模式（默认）
    tooltip_delay: Duration::from_millis(400),
    ..WindowOption::default()
}.open(move |_window_id| {
    // 返回控件树
    my_view().into_views()
})?;
```

### ⚡ 保留模式 + 即时模式

每个窗口可独立选择渲染模式。即使在保留模式下，框架也通过帧精度唤醒机制（`min_wait_time`）天然支持高性能动画。

### 🎥 动画 = 状态函数

`on_frame` 的实现可以写成状态函数 —— 直接根据当前绝对时间计算状态，而不是累加 delta
时间。这意味着动画代码天然是幂等的、无漂移的、可预测的。动画不是一个特殊机制，它只是 `f(now)` —— 当前时刻的状态。

### 🎨 控件式框架 + Builder 模式

每个 `View` 实现自动获得丰富的 Builder API：

```rust
text_input()
    .class("size-md input-filled")
    .style(|s| s.placeholder("请输入文本...".to_string()).mask('*'))
    .on_click(|view_id, key_state, pos| { /* ... */ })
    .on_text_change(|text| println!("变化: {}", text))
    .validator(|ch| ch.is_ascii_digit())
    .bind_text(text_writer)
```

### 📐 CSS 风格布局

基于 [Taffy](https://github.com/DioxusLabs/taffy)，支持 Flexbox、Grid 和 Block 布局。类似 TailwindCSS 的工具类，快速完成样式定义：

```rust
scroll_area(views![
    label("标题").class("text-2xl font-bold"),
    button("点击").class("btn-outline"),
])
.class("flex flex-col gap-2 p-4")
```

### 🧰 View 特征

`View` trait 是所有控件的基础。框架提供约 30 个可重写的回调方法，覆盖：

- **生命周期**: `on_create`、`on_destroy`、`on_child_push`
- **绘制**: `on_draw`、`on_draw_overlay`、`on_visual_overflow`
- **鼠标**: `on_mouse_enter/leave/move`、`on_click`、`on_button_down/up`
- **键盘**: `on_key_down`、`on_key_up`、`on_char_input`
- **焦点**: `on_focus`、`on_blur`
- **工具提示**: `on_tooltip_show`、`on_tooltip_hide`
- **布局**: `on_measure`、`on_layout_computed`
- **帧**: `on_frame`（支持精确唤醒调度）
- **拖放**: `on_drag_enter/over/leave`、`on_drop`

每个事件都有 `on_` 方法（控件默认实现）和外置 handler 槽位，允许用户在不继承的情况下覆盖行为。

### 🪶 极简依赖

框架自研实现了：

- **图形层** — Direct2D / OpenGL 后端，不依赖 GPU 抽象 crate
- **平台层** — 直接调用 Win32 API，不依赖窗口抽象 crate
- **事件循环** — 自定义保留/即时混合循环

结果：小巧的二进制体积和快速的编译时间。

## 快速开始

```rust
use flor::{FlorGui, views};
use flor::windows::window_options::WindowOption;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    FlorGui.init()?;

    WindowOption {
        title: "Hello Flor".to_string(),
        width: 800,
        height: 600,
        ..WindowOption::default()
    }
        .open(move |_window| {
            // 控件树（由控件库如 flor-lys 提供）
            vec![]
        })?;

    FlorGui.event_loop()?;
    Ok(())
}
```

> **注意**：Flor 是一个*框架*，不是控件库。它提供基础设施 — 视图系统、信号、渲染、事件派发 — 但不包含任何内置控件。

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                       你的应用                           │
├─────────────────────────────────────────────────────────┤
│                  控件库 (flor-lys)                       │  ← 独立 crate
├────────────┬──────────┬──────────────┬──────────────────┤
│  View      │  信号     │  布局        │  窗口            │
│  视图系统   │  系统     │  (Taffy)     │  管理            │
├────────────┴──────────┴──────────────┴──────────────────┤
│                      渲染抽象层                          │
├─────────────────────┬───────────────────────────────────┤
│   Direct2D 后端      │         OpenGL 后端                │
├─────────────────────┴───────────────────────────────────┤
│                       平台层                             │
│               (Windows / 未来: Linux, macOS)             │
└─────────────────────────────────────────────────────────┘
```

### Crate 结构

| Crate                    | 用途                       |
|--------------------------|--------------------------|
| `flor`                   | 核心框架 — 视图系统、信号、事件循环、窗口管理 |
| `flor-base`              | 共享类型和图形抽象                |
| `flor-macros`            | 过程宏（`style!`、`color!` 等） |
| `flor-graphics-direct2d` | Direct2D 渲染后端            |
| `flor-graphics-opengl`   | OpenGL 渲染后端              |
| `flor-platform-windows`  | Windows 平台实现             |

## Feature Flags

| Feature          | 说明                   |
|------------------|----------------------|
| `direct2d`       | Direct2D 渲染后端 *（默认）* |
| `layout-flex`    | Flexbox 布局           |
| `layout-grid`    | Grid 布局              |
| `layout-block`   | Block 布局             |
| `svg`            | SVG 渲染               |
| `clipboard`      | 剪贴板支持                |
| `drag-drop`      | 拖放支持                 |
| `tray`           | 系统托盘支持               |
| `memory-font`    | 内存字体加载               |
| `monitor`        | 多显示器支持               |
| `hi-dpi`         | 高 DPI 支持             |
| `theme-change`   | 系统主题变化检测             |
| `signal-tracing` | 信号调试追踪               |

## 关于控件库

为了鼓励大家尝试这个框架，作者维护了 [flor-lys](https://github.com/user/flor-lys) 作为官方控件库，给大家一个保底选择。

但框架与控件库分离的设计，不仅仅是为了让框架更加灵活、职责分离。更是鼓励**个人、组织、公司**
在维护自己的软件时，创建自己的品牌风格，维护自己风格的控件库。Flor 的 `View` trait 设计使得控件源码几乎是 copy 可用的 ——
想要维护自己风格的控件，只要会 copy 也能简单地上手。

这正是这个框架名字的寓意：

> *繁花继开，各放异彩。*

## 目标用户

- **小工具开发者** — 小体积、快启动、原生外观
- **高性能需求的业务** — 数据大屏、波形图应用、交易证券终端
- **需要多人维护的纯 Rust 项目** — 信号驱动架构使数据流可追踪、可重构。增删数据源只需处理信号，无需到处写更新逻辑
- **重型 UI 且重性能的项目** — 控件式框架，可自定义控件获得完全的渲染控制，性能远超基于 diff tree 的组合式 UI 框架

## 核心模块

| 模块        | 说明                                              |
|-----------|-------------------------------------------------|
| `view`    | View trait、ViewId、ViewStorage、ViewBuilder       |
| `signal`  | 响应式信号系统（RwSignal、ReadSignal、WriteSignal、Effect） |
| `render`  | 渲染抽象层（FlorRender）                               |
| `windows` | 窗口创建、事件循环、消息派发                                  |

## 文档

详细文档请查看 `docs/` 目录：

- [`view_trait.md`](docs/view_trait.md) — View Trait 完整参考
- [`signal.md`](docs/signal.md) — 信号系统指南
- [`layout_syntax.md`](docs/layout_syntax.md) — 布局 DSL 文档
- [`style_derive_macro.md`](docs/style_derive_macro.md) — Style 宏参考
- [`view_id.md`](docs/view_id.md) — ViewId API 参考

## License

TODO

## TODO

例子，截图，性能，不同平台体积表，不同平台支持状态表，与其他框架的比较

开箱即用 (Out-of-the-box) —— 纯 Rust 实现，零 C/C++ 系统级依赖。无需配置 CMake、无需安装额外的 UI 库开发包。git clone 后直接
cargo run 即可运行。

国际化（i18n），因为信号机制的原因，应该由控件库甚至是用户层处理。

路线图 (Roadmap)

[x] Windows 原生平台层与 Direct2D 后端

[x] 响应式信号系统与 Taffy 布局集成

[ ] OpenGL 后端完善

[ ] macOS 与 Linux 平台层支持