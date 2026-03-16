**English** | [Chinese](README.zh-CN.md)

# Flor

A high-performance, signal-driven native GUI framework for Rust. Modern, ergonomic, and widget-based — built with a
small footprint for cross-platform versatility.

> Translator's note: If you can read Chinese, please prefer the Chinese version; it is the author's original manuscript.
> Other language versions are mainly model-translated and are for reference only.

> ⚠️ Stability statement: Flor is still in an early stage. The current API is only preliminarily stable; before the
> first official release on crates.io, it may change or be removed at any time, without a deprecation period.

Flor is a widget-based, reactive GUI framework built from scratch. The graphics layer, platform layer, layout engine
integration, and signal system are all implemented in-house, without depending on heavy libraries such as winit or wgpu.
The goal is to create the Rust GUI framework the author personally wants: **small footprint, high performance, ergonomic
APIs, and zero design barrier**.

Detailed documentation will be released later as a separate website project. This README is only an overview of the
framework and its positioning.

## Positioning and Goals

Flor positions itself as a **general-purpose Rust GUI framework for everyone**: it is not made for a specific product or
vertical domain, and aims to be the first thing people think of when building native Rust desktop apps. The API targets
a low learning curve; the signal system and handle exposure make collaboration and deep extensions easy; and the
in-house graphics/platform layers bring high performance and a small footprint. The official widget library and
documentation are still being polished; in the short term, the focus is on the core framework experience. Later, with a
more complete widget library and docs, the project aims to be impressive out of the box.

Use cases:

- Small utilities and desktop tools: small binaries, fast startup, native look and feel
- High-performance visualization/trading apps: dashboards, waveforms/market data, low-latency drawing
- Multi-maintainer pure-Rust projects: signal-driven dataflow is easy to trace and refactor; less manual invalidation
  and lifetime gymnastics
- Heavy UI with strict performance needs: the widget approach makes custom widgets straightforward, and is often faster
  than diff-tree-based compositional UI

> *Flor* is the French spelling of *flower*. The name comes from a utility app the author once planned called "Wanhua
> Ji". As other planned tools took over parts of its responsibilities, the remaining GUI core gained a new life as this
> framework, inheriting the "Wanhua" name.

---

## Table of Contents

- [Positioning and Goals](#positioning-and-goals)
- [Design Motivation](#design-motivation)
- [Design Goals](#design-goals)
- [What Makes Flor Unique](#what-makes-flor-unique)
    - [🔄 Reactive Signal System](#-reactive-signal-system)
    - [🪟 Flexible Multi-Window Support](#-flexible-multi-window-support)
    - [⚡ Retained Mode + Immediate Mode](#-retained-mode--immediate-mode)
    - [🎥 Animation = State Functions](#-animation--state-functions)
    - [🎨 Widget Framework + Builder Pattern](#-widget-framework--builder-pattern)
    - [📐 CSS-Style Layout](#-css-style-layout)
    - [🧰 The View Trait](#-the-view-trait)
    - [🪶 Minimal Dependencies](#-minimal-dependencies)
- [Quick Start](#quick-start)
- [Crate Structure](#crate-structure)
- [Rendering Backends](#rendering-backends)
- [Feature Flags](#feature-flags)
- [About the Widget Library](#about-the-widget-library)
- [License](#license)

---

## Design Motivation

Rust's ownership model brings unique challenges to GUI development. Most existing frameworks suffer from some of the
following issues:

- **Lifetime-driven design barriers**: accessing and mutating state across a widget tree often turns into a fight with
  the borrow checker
- **Window-context coupling**: binding windows to a global context makes multi-window workflows extremely painful
- **Heavy dependencies**: relying on large abstraction layers (such as winit, wgpu, etc.) pulls a huge dependency tree
  for a simple tool

Flor was born out of the frustration of trying framework after framework and still not finding a satisfying solution.
Inspired by miniquad's simple source organization, Floem's signal system, and FreeXpand_DUI's flexible event dispatch
ideas, Flor went through five major design iterations and eventually formed a reactive-signal-based architecture that
fundamentally resolves Rust's ownership tensions in GUI development.

> **What is a "design barrier"?**
>
> 1. Rust lifetimes make it hard to freely access widgets and handle interactions like in object-oriented languages
> 2. Many frameworks bind context to a window, or constrain windows inside a global context, making window operations
     miserable
> 3. Without flexible multi-window support, tool-style apps become hard to accept

## Design Goals

| Goal                   | Notes                                                                                  |
|------------------------|----------------------------------------------------------------------------------------|
| 🏎️ **Performance**    | Retained mode by default, immediate mode on demand. Frame-accurate wake-up scheduling. |
| 📦 **Small footprint** | In-house graphics and platform layers, no heavy abstraction crates.                    |
| 🧩 **Ergonomics**      | Signal-driven reactivity that eliminates manual UI invalidation propagation.           |
| 🔧 **Complete APIs**   | Tray, clipboard, drag-drop, multi-monitor, Hi-DPI, theme detection: out of the box.    |

## What Makes Flor Unique

If you are evaluating Rust GUI options, the following are features Flor brings that are hard to find elsewhere. They
fill real gaps in the Rust GUI ecosystem:

- **In-house graphics and platform layers**: no heavy abstraction crates (winit, wgpu, etc.). Few dependencies, small
  binaries, fast compiles.
- **Reactive signals, O(1) update propagation, thread-safe**: signals can be read and written from any thread, without
  manual UI refresh.
- **Hybrid rendering modes (retained built on immediate)**: each window can independently choose retained or immediate
  mode. Retained mode runs on the immediate-mode execution path, with performance effectively on par with immediate
  mode (the gap is negligible), and frame-accurate wake-ups enable high-performance animations.
- **Create and control windows anywhere**: no global context, no lifetime gymnastics. Open windows from a button
  callback, from another thread, or anywhere you need.
- **Native handle exposure**: access native platform window handles for easy interop with native APIs and third-party
  libraries.
- **Low floor, high ceiling**: the consistent `View` trait API means even the simplest widget gets full lifecycle
  management, hit testing, focus, tooltips, and event dispatch. The same system also supports complex custom-rendered
  widgets.
- **Designed for teams and long-lived projects**: signal-driven dataflow is naturally traceable. Adding/removing data
  sources is just updating signals, instead of cleaning up scattered manual refresh logic.
- **Framework and widgets decoupled**: the core framework contains no built-in widgets; widget libraries are maintained
  independently. Even a simple game can be implemented as a `View`.
- **TailwindCSS-like utility classes**: define layout and styles with familiar class names like
  `"flex flex-col gap-2 p-4"`, which feels natural for developers coming from the Web.

> Vision: make Flor one of the GUI options people naturally think of and are willing to try in the Rust ecosystem. When
> developers need a native desktop app, it should be a default candidate, like Qt in C++ or WPF in C#.

### 🔄 Reactive Signal System

O(1) update propagation. Thread-safe. No manual refresh calls.

```rust
use flor::signal::rw_signal::create_signal;
use flor::signal::read::Read;
use flor::signal::write::Write;

fn signal() {
    let (reader, writer) = create_signal("Hello".to_string()).split();

    // Writer can be moved to another thread
    writer.set("World".to_string());

    // Reader automatically tracks dependencies
    let text = reader.get(); // "World"
}
```

### 🪟 Flexible Multi-Window Support

Create windows anywhere, no global context. Each window can independently choose retained mode or immediate mode.

```rust
use flor::windows::window_options::WindowOption;

fn main() -> Result<(), Box<dyn Error>> {
    let _window = WindowOption {
        title: "My App".to_string(),
        width: 1280,
        height: 800,
        continuous_rendering: false,   // Retained mode (default)
        tooltip_delay: Duration::from_millis(400),
        ..WindowOption::default()
    }.open(move |_window_id| {
        // Return the widget tree
        my_view().into_views()
    })?;
}
```

### ⚡ Retained Mode + Immediate Mode

Each window can independently choose its rendering mode. Even in retained mode, frame-accurate wake-up scheduling (
`min_wait_time`) makes high-performance animations a natural fit.

### 🎥 Animation = State Functions

`on_frame` can be implemented as a state function: compute state directly from absolute time, instead of accumulating
delta time. This makes animation code naturally idempotent, drift-free, and predictable. Animation is not a special
mechanism; it is just `f(now)`: the state at the current moment.

### 🎨 Widget Framework + Builder Pattern

Every `View` implementation automatically gets a rich builder-style API:

```rust
fn demo() {
    text_input()
        .class("size-md input-filled")
        .style(|s| s.placeholder("Enter text...".to_string()).mask('*'))
        .on_click(|view_id, key_state, pos| { /* ... */ })
        .on_text_change(|text| println!("Changed: {}", text))
        .validator(|ch| ch.is_ascii_digit())
        .bind_text(text_writer)
}
```

### 📐 CSS-Style Layout

Based on [Taffy](https://github.com/DioxusLabs/taffy), with Flexbox, Grid, and Block layouts. With TailwindCSS-like
utility classes, you can define styles quickly:

```rust
fn demo() {
    scroll_area(views![
    label("Title").class("text-2xl font-bold"),
    button("Click").class("btn-outline"),
])
        .class("flex flex-col gap-2 p-4")
}
```

### 🧰 The View Trait

The `View` trait is the foundation of all widgets. The framework provides about 30 overridable callbacks, covering:

- **Lifecycle**: `on_create`, `on_destroy`, `on_child_push`
- **Rendering**: `on_draw`, `on_draw_overlay`, `on_visual_overflow`
- **Mouse**: `on_mouse_enter/leave/move`, `on_click`, `on_button_down/up`
- **Keyboard**: `on_key_down`, `on_key_up`, `on_char_input`
- **Focus**: `on_focus`, `on_blur`
- **Tooltips**: `on_tooltip_show`, `on_tooltip_hide`
- **Layout**: `on_measure`, `on_layout_computed`
- **Frame**: `on_frame` (supports precise wake-up scheduling)
- **Drag-and-drop**: `on_drag_enter/over/leave`, `on_drop`

Each event has both an `on_` method (the widget's default implementation) and external handler slots, allowing users to
override behavior without inheritance.

### 🪶 Minimal Dependencies

The framework implements in-house:

- **Graphics layer**: Direct2D / OpenGL backends, no GPU abstraction crate
- **Platform layer**: calls Win32 APIs directly, no window abstraction crate
- **Event loop**: custom retained/immediate hybrid loop

Result: small binaries and fast compile times.

## Quick Start

```rust
use flor::{FlorGui, views};
use flor::windows::window_options::WindowOption;

fn main() -> Result<(), Box<dyn Error>> {
    FlorGui.init()?;

    WindowOption {
        title: "Hello Flor".to_string(),
        width: 800,
        height: 600,
        ..WindowOption::default()
    }
        .open(move |_window| {
            // Widget tree (provided by a widget library such as flor-lys)
            vec![]
        })?;

    FlorGui.event_loop()?;
    Ok(())
}
```

> **Note**: Flor is a *framework*, not a widget library. It provides the infrastructure: view system, signals,
> rendering, and event dispatch, but contains no built-in widgets.

## Crate Structure

The graphics backends and the platform layer sit at the same dependency level and do not depend on each other; the core
framework selects them as needed.

| Crate                    | Purpose                                                             | Layer             |
|--------------------------|---------------------------------------------------------------------|-------------------|
| `flor`                   | Core framework: view system, signals, event loop, window management | L2                |
| `flor-base`              | Shared types and graphics abstractions                              | L0 base           |
| `flor-macros`            | Proc macros (`style!`, `color!`, etc.)                              | L0 base           |
| `flor-graphics-direct2d` | Direct2D rendering backend                                          | L1 render backend |
| `flor-graphics-opengl`   | OpenGL rendering backend                                            | L1 render backend |
| `flor-platform-windows`  | Windows platform implementation                                     | L1 platform       |

Smaller layer numbers mean lower-level; crates within the same layer do not depend on each other.

## Rendering Backends

- `direct2d`: Windows-native Direct2D GPU backend
- `opengl`: compatibility GPU backend
- `tiny-skia`: pure CPU rendering backend

> Flor supports both GPU and CPU rendering: enable at least one backend. At most, you can enable 1 GPU backend + 1 CPU
> backend at the same time; if GPU initialization fails, it falls back to CPU.

## Feature Flags

| Feature                        | Description                                                                   |
|--------------------------------|-------------------------------------------------------------------------------|
| `direct2d`                     | Direct2D GPU rendering backend (enables the GPU pipeline)                     |
| `opengl`                       | OpenGL GPU rendering backend                                                  |
| `tiny-skia`                    | Pure CPU rendering backend (enables the CPU pipeline)                         |
| `cpu-render-backend`           | Explicitly select the CPU pipeline (`tiny-skia` enables it implicitly)        |
| `gpu-render-backend`           | Explicitly select the GPU pipeline (`direct2d`/`opengl` enable it implicitly) |
| `layout-flex`                  | Flexbox layout (taffy/flexbox)                                                |
| `layout-grid`                  | Grid layout (taffy/grid)                                                      |
| `layout-block`                 | Block layout (taffy/block)                                                    |
| `class`                        | Class-name support (Tailwind-style)                                           |
| `svg`                          | Top-level SVG support, passed through to the render backend                   |
| `clipboard`                    | Clipboard                                                                     |
| `drag-drop`                    | Drag-and-drop                                                                 |
| `tray`                         | System tray                                                                   |
| `memory-font`                  | Load fonts from memory                                                        |
| `monitor`                      | Multi-monitor                                                                 |
| `hi-dpi`                       | High DPI                                                                      |
| `theme-change`                 | Detect system theme changes                                                   |
| `signal-tracing`               | Signal debug tracing (todo)                                                   |
| `cross-thread-window-creation` | Allow creating windows across threads                                         |

## About the Widget Library

To encourage people to try the framework, the author maintains [flor-lys](https://github.com/user/flor-lys) as an
official widget library, as a safe default choice.

But separating the framework from widget libraries is not just about flexibility and separation of concerns. It also
encourages **individuals, organizations, and companies** to build their own branded style and maintain their own widget
libraries. Flor's `View` trait design makes widget source code almost copy-and-paste reusable: if you can copy, you can
get started quickly.

That is the meaning behind this framework's name:

> *繁花继开，各放异彩。*  
> *Flowers bloom in endless succession, each shining with its own radiance.*

## Roadmap

- Widget library (in progress)
- Linux platform layer
- API polishing and documentation site before the first release

## License

This project is licensed under the Mozilla Public License 2.0 (MPL-2.0). Contributions and redistribution must comply
with MPL-2.0. See the `LICENSE` file at the repository root.
