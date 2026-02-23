**English** | [中文](README.zh-CN.md)

# Flor

A modern, ergonomic, lightweight, native GUI framework for Rust.

Flor is a widget-based GUI framework built from scratch — graphics layer, platform layer, layout engine integration, and
signal system — all without heavy dependencies such as winit or wgpu. Designed to be the kind of Rust GUI framework the
author wished existed: **small binary, high performance, ergonomic API, zero design barriers**.

> The name *Flor* is the French spelling of *flower*. It originates from a tool application the author once planned to
> build called *万花姬* (roughly "Myriad Flowers"). As other tools gradually took over parts of its responsibilities,
> what
> remained was the GUI core — which was given new life as this framework, inheriting the *flower* name.

## Motivation

Rust's ownership model creates unique challenges for GUI development. Most existing frameworks suffer from one or more
of:

- **Lifetime-induced design barriers** — Accessing and mutating widgets across the tree is fighting the borrow checker
- **Window context coupling** — Windows bound to global contexts make multi-window workflows painful
- **Heavy dependencies** — Depending on large abstraction layers (e.g. winit, wgpu, etc.) pulls in a massive dependency
  graph for what might be a simple tool

Flor was born out of the frustration of trying framework after framework and finding none satisfactory. Inspired by
miniquad's clean source organization, Floem's signal system, and freexpand_dui's flexible event dispatch — then iterated
through five major design revisions to arrive at a reactive signal architecture that resolves Rust's fundamental
ownership tension for GUI work.

## Design Goals

| Goal                        |                                                                                           |
|-----------------------------|-------------------------------------------------------------------------------------------|
| 🏎️ **Performance**         | Retained-mode by default, instant-mode when needed. Frame-precise wakeup.                 |
| 📦 **Small Binary**         | Self-implemented graphics and platform layers. No dependency on heavy abstraction crates. |
| 🧩 **Ergonomics**           | Signal-driven reactivity eliminates manual update propagation.                            |
| 🔧 **Complete API Surface** | Tray, clipboard, drag-drop, multi-monitor, Hi-DPI, theme detection — built-in.            |

## What Sets Flor Apart

For those evaluating Rust GUI options, here's what Flor brings that you won't easily find elsewhere — features that fill
real gaps in the Rust GUI ecosystem:

- **Self-implemented graphics and platform layers** — No dependency on heavy abstraction crates (e.g. winit, wgpu,
  etc.). Minimal dependencies, small binary size, fast compile.
- **Reactive signals with O(1) updates, thread-safe by design** — Signals can be read and written from any thread. No
  manual UI refresh calls.
- **Per-window retained/immediate mode** — Each window independently chooses its rendering strategy. Mix both in the
  same application.
- **Retained mode built on immediate mode internals** — High-performance animation support is native even in retained
  mode, with frame-precise wakeup scheduling.
- **Create and control windows from anywhere** — No global context, no lifetime gymnastics. Open a window from a button
  click handler, from another thread, from wherever you need.
- **Native handle exposure** — Raw platform window handles are accessible for interop with native APIs and third-party
  libraries.
- **Low floor, high ceiling** — The View trait's consistent API means even a trivial widget gets full framework-managed
  lifecycle, hit testing, focus, tooltips, and event dispatch. Yet the same system scales to complex custom-rendered
  controls.
- **Built for teams and long-lived projects** — Signal-driven data flow is inherently traceable. Add or remove a data
  source by updating signals — no scattered manual refresh logic to hunt down.
- **Framework–widget separation** — The core ships zero widgets. Widget libraries are independent crates. You can even
  write simple games (Snake, Tetris, chess) as `View` implementations.
- **TailwindCSS-like utility classes** — Style widgets with familiar class names like `"flex flex-col gap-2 p-4"`,
  making the transition from web development feel natural.

> Flor aims to become the **default GUI framework** in the Rust ecosystem — the one developers reach for instinctively
> when they need a native desktop application, the way Qt is for C++ or WPF is for C#.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Your Application                     │
├─────────────────────────────────────────────────────────┤
│                 Widget Library (flor-lys)                │  ← Separate crate
├────────────┬──────────┬──────────────┬──────────────────┤
│  View      │  Signal  │  Layout      │  Window          │
│  System    │  System  │  (Taffy)     │  Management      │
├────────────┴──────────┴──────────────┴──────────────────┤
│                    Render Abstraction                    │
├─────────────────────┬───────────────────────────────────┤
│   Direct2D Backend  │         OpenGL Backend            │
├─────────────────────┴───────────────────────────────────┤
│                   Platform Layer                        │
│               (Windows / future: Linux, macOS)          │
└─────────────────────────────────────────────────────────┘
```

### Crate Structure

| Crate                    | Purpose                                                              |
|--------------------------|----------------------------------------------------------------------|
| `flor`                   | Core framework — view system, signals, event loop, window management |
| `flor-base`              | Shared types and graphics abstractions                               |
| `flor-macros`            | Procedural macros (`style!`, `color!`, etc.)                         |
| `flor-graphics-direct2d` | Direct2D rendering backend                                           |
| `flor-graphics-opengl`   | OpenGL rendering backend                                             |
| `flor-platform-windows`  | Windows platform implementation                                      |

## Key Features

### 🔄 Reactive Signal System

O(1) update propagation. Thread-safe. No manual refresh calls.

```rust
use flor::signal::rw_signal::create_signal;
use flor::signal::read::Read;
use flor::signal::write::Write;

let (reader, writer) = create_signal("Hello".to_string()).split();

// Writer can be moved to another thread
writer.set("World".to_string());

// Reader automatically tracks dependencies
let text = reader.get(); // "World"
```

### 🪟 Flexible Multi-Window

Create windows anywhere — no global context required. Each window independently chooses retained or immediate mode.

```rust
use flor::windows::window_options::WindowOption;

let _window = WindowOption {
title: "My App".to_string(),
width: 1280,
height: 800,
continuous_rendering: false,   // retained mode (default)
tooltip_delay: Duration::from_millis(400),
..WindowOption::default ()
}.open( move | _window_id| {
// Return your widget tree
my_view().into_views()
}) ?;
```

### ⚡ Retained + Immediate Mode

Per-window rendering mode selection. Even in retained mode, the framework supports high-performance animations through
its frame-precise wakeup mechanism (`min_wait_time`).

### 🎨 Widget-Based with Builder Pattern

Every `View` implementation gets a rich builder API for free:

```rust
text_input()
.class("size-md input-filled")
.style( | s| s.placeholder("Enter text...".to_string()).mask('*'))
.on_click( | view_id, key_state, pos| { /* ... */ })
.on_text_change( | text| println!("Changed: {}", text))
.validator( | ch| ch.is_ascii_digit())
.bind_text(text_writer)
```

### 📐 CSS-like Layout

Powered by [Taffy](https://github.com/DioxusLabs/taffy) with Flexbox, Grid, and Block layout support. TailwindCSS-like
utility classes for rapid styling:

```rust
scroll_area(views![
    label("Title").class("text-2xl font-bold"),
    button("Click me").class("btn-outline"),
])
.class("flex flex-col gap-2 p-4")
```

### 🧰 View Trait

The `View` trait is the foundation of every widget. The framework provides ~30 overridable callbacks covering:

- **Lifecycle**: `on_create`, `on_destroy`, `on_child_push`
- **Drawing**: `on_draw`, `on_draw_overlay`, `on_visual_overflow`
- **Mouse**: `on_mouse_enter/leave/move`, `on_click`, `on_button_down/up`
- **Keyboard**: `on_key_down`, `on_key_up`, `on_char_input`
- **Focus**: `on_focus`, `on_blur`
- **Tooltip**: `on_tooltip_show`, `on_tooltip_hide`
- **Layout**: `on_measure`, `on_layout_computed`
- **Frame**: `on_frame` (with precise wakeup scheduling)
- **Drag & Drop**: `on_drag_enter/over/leave`, `on_drop`

Each has both an `on_` method (widget default) and an external handler slot, enabling users to override behavior without
subclassing.

### 🪶 Minimal Dependencies

The framework implements its own:

- **Graphics layer** — Direct2D / OpenGL backends, no dependency on GPU abstraction crates
- **Platform layer** — Direct Win32 API, no dependency on windowing abstraction crates
- **Event loop** — Custom retained/immediate hybrid loop

The result: small binaries and fast compile times.

## Feature Flags

| Feature          | Description                         |
|------------------|-------------------------------------|
| `direct2d`       | Direct2D render backend *(default)* |
| `layout-flex`    | Flexbox layout                      |
| `layout-grid`    | Grid layout                         |
| `layout-block`   | Block layout                        |
| `svg`            | SVG rendering                       |
| `clipboard`      | Clipboard support                   |
| `drag-drop`      | Drag & drop support                 |
| `tray`           | System tray support                 |
| `memory-font`    | In-memory font loading              |
| `monitor`        | Multi-monitor support               |
| `hi-dpi`         | High DPI support                    |
| `theme-change`   | System theme change detection       |
| `signal-tracing` | Signal debug tracing                |

## Quick Start

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
            // Widget tree goes here (provided by a widget library like flor-lys)
            vec![]
        })?;

    FlorGui.event_loop()?;
    Ok(())
}
```

## About Widget Libraries

Flor is a *framework*, not a widget library — it provides the infrastructure (view system, signals, rendering, event
dispatch) but ships no built-in widgets.

To lower the barrier to entry, the author maintains [flor-lys](https://github.com/user/flor-lys) as an official widget
library, giving everyone a solid starting point. But the framework–widget separation is about more than just clean
architecture. It is an invitation:

**Individuals, teams, and companies are encouraged to create and maintain their own widget libraries** with their own
brand identity and visual style. Flor's `View` trait design makes widget source code almost copy-ready — if you can copy
a widget, you can customize it. Forking a single widget to match your brand is as simple as it sounds.

This is precisely what the name of this framework symbolizes:

> *繁花继开，各放异彩。*
>
> — roughly: *May every widget library bloom like a flower — each with its own brilliance.*

## Target Users

- **Tool builders** — Small binary, fast startup, native look
- **Performance-critical applications** — Data dashboards, waveform displays, trading terminals
- **Teams maintaining large Rust projects** — Signal-driven architecture makes data flow traceable and refactor-safe.
  Add or remove a data source? Just update the signals, no manual propagation.
- **Heavy-UI applications** — Custom widgets with full rendering control, far outperforming diff-tree based
  compositional UI frameworks

## Core Modules

| Module    | Description                                                         |
|-----------|---------------------------------------------------------------------|
| `view`    | View trait, ViewId, ViewStorage, ViewBuilder                        |
| `signal`  | Reactive signal system (RwSignal, ReadSignal, WriteSignal, effects) |
| `render`  | Render abstraction layer (FlorRender)                               |
| `windows` | Window creation, event loop, bus dispatch                           |

## Documentation

See the `docs/` directory for detailed guides:

- [`view_trait.md`](docs/view_trait.md) — Complete View trait reference
- [`signal.md`](docs/signal.md) — Signal system guide
- [`layout_syntax.md`](docs/layout_syntax.md) — Layout DSL documentation
- [`style_derive_macro.md`](docs/style_derive_macro.md) — Style macro reference
- [`view_id.md`](docs/view_id.md) — ViewId API reference

## License

TODO