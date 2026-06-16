**English** | [中文](README.zh-CN.md)

# Flor

Flor is a modern, easy-to-use, lightweight, high-performance, general-purpose pure Rust native desktop GUI framework.

It features a reactive signal system, widget-based development model, and self-built platform/graphics layers, aiming to
provide a more natural, consistent, and suitable choice for long-term native GUI application development in Rust.

> Note: If you can read Chinese, please refer to the Chinese version (README.zh-CN.md) first. It is the author's
> original draft; other language versions are primarily machine-translated for reference only.

> ⚠️ Stability Notice: Flor is still in its early stages. The current API is only prelimarily stable; before official
> release to crates.io, it may change or be removed at any time without deprecation notices.

## Current Status

Flor is currently in the pre-release stage.

The core framework has entered a convergence phase after design validation, but the official widget library,
documentation site, example projects, and cross-platform support are still being refined. This stage is more suitable
for those willing to observe design evolution, run examples, submit bugs, and participate in early feedback.

If you need fully stable production-grade commitments, we recommend waiting for future official releases.

## Why Flor Exists

Rust's ownership model brings unique challenges to GUI development. Many GUI frameworks encounter these issues in
complex applications:

* Widget trees, state, and lifecycles are entangled with each other;
* Windows and contexts are tightly bound, making multi-window development unnatural;
* Cross-widget access and state updates require significant workaround efforts;
* Simple tools may still pull in heavy dependencies from large window layers, graphics layers, or abstraction crates;
* The boundary between application developers and widget authors is not clearly defined.

Flor was born from the frustration of trying one Rust GUI framework after another without finding a satisfactory
solution.

Initially, I just wanted a Rust GUI framework that felt right to use: no forced context binding, no restrictions
everywhere on multi-window scenarios, no heavy dependencies for simple tools, and no endless wrestling with the borrow
checker over state updates and widget access.

But after rounds of design validation and refactoring, Flor gradually evolved into what it is today:

It doesn't just aim to solve problems in a single scenario, but seeks to find the most perfect answer—for the
author—among small size, high performance, easy-to-use API, multi-window support, widget extensibility, and long-term
maintenance.

In other words, Flor is a "want it all, and more" Rust GUI framework.

This wasn't a deliberately crafted slogan from the start, but a direction that naturally emerged through continuous
trial, error, revision, and refactoring.

## Core Features

* **Pure Rust Native GUI**
  Platform layer, graphics layer, event loop, layout integration, signal system, and view system are all built from
  scratch.

* **Small Size, Few Dependencies**
  No forced dependency on heavy abstraction crates like winit or wgpu, minimizing the dependency tree and binary size.

* **Reactive Signal System**
  Signals can be read and written across threads, supporting O(1) update propagation, reducing manual refresh and
  lifetime gymnastics.

* **No Forced Context Binding**
  Windows, signals, views, and tasks are not tightly bound by a global context, making complex applications easier to
  organize.

* **Flexible Multi-Window**
  Supports creating and controlling windows from any thread and any location, with each window able to independently set
  its refresh mode.

* **Retained Mode + Immediate Mode**
  Flor's retained mode is built on immediate mode execution paths, maintaining near-immediate-mode performance
  characteristics in retained mode, and naturally suitable for high-performance animations.

* **Widget-Based Development Model**
  `View` is the foundation of Flor's widgets. The framework handles lifecycle, hit testing, focus, event dispatch,
  layout, and drawing; widget authors can extend their own widgets on a unified model.

* **Declarative UI DSL and Utility-First Styling**
  Flor supports declarative layout expressions similar to modern frontend, and utility-first styling similar to
  TailwindCSS, making it easy to quickly organize interface structures.

## Quick Start

Flor is still in its early stages and has not been published to crates.io. Please import via Git repository first:

```toml
# cargo.toml
[dependencies]
flor = { git = "https://github.com/flor/flor" }
flor-lys = { git = "https://github.com/flor/flor-lys" }
```

Where:

* `flor` is the core framework, providing infrastructure such as view system, signal system, window management,
  rendering, layout integration, and event dispatch;
* `flor-lys` is the official widget library maintained by the author, providing an out-of-the-box baseline option.

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
            // Widget trees are typically provided by widget libraries, e.g., flor-lys
            vec![]
        })?;

    FlorGui.event_loop()?;
    Ok(())
}
```

> Note: Flor is a **framework**, not a widget library.
>
> It is responsible for providing the infrastructure needed for GUI applications, including view system, signal system,
> window management, rendering, layout integration, and event dispatch capabilities.
> The official widget library [flor-lys](https://github.com/flor/flor-lys) is maintained as a separate project, giving
> users a baseline option to directly try out Flor.
>
> `flor-lys` serves another important purpose: it continuously acts as a validation ground for the Flor framework
> itself. In earlier major versions and the initial phase of the fifth version design, Flor encountered situations
> where "
> the design seemed workable on paper, but couldn't actually work when writing widgets and applications." The author
> will
> continue participating in `flor-lys` maintenance, using a real widget library to validate Flor's architecture, API,
> and
> extension model, ensuring Flor isn't just a framework that works in theory.
>
> However, the author's core focus remains on the Flor framework itself. `flor-lys` will strive to ensure basic
> usability, but if it needs more professional visual presentation, a more complete widget system, and longer-term
> ecosystem expansion, it still requires joint contributions from the community, third-party developers, teams, or
> companies.
>
> The separation of framework and widget library design is not just to make responsibilities clearer, but also to
> encourage individuals, organizations, and companies to create their own brand styles and maintain their own widget
> libraries when maintaining their software.
>
> Flor's `View` trait design makes widget source code highly portable and adaptable. To maintain a widget library with
> your own style, you don't necessarily need to understand the entire framework from scratch; often, you can gradually
> get
> started by referencing, copying, and modifying existing widgets.
>
> This is exactly what the framework's name signifies:
>
> *Flowers bloom in succession, each radiating its own brilliance.*
>
> If you are unfamiliar with this framework and want to try using it, we strongly recommend reading the documentation
> first. The framework has some intentional designs or unique mechanisms. If you don't understand them, you might
> mistake
> them for "bugs."

## Suitable Use Cases

Flor is suitable for:

* Native Rust desktop applications;
* Small, fast-launching utility software;
* Multi-window applications;
* High-performance visualization, trading charts, waveforms, monitoring panels, and other low-latency rendering
  scenarios;
* GUI projects requiring long-term maintenance, multi-person collaboration, and clear state flow;
* Teams wanting to maintain custom widget libraries or brand-specific widget systems.

## Documentation

Detailed documentation is published on a dedicated documentation site. This README only covers the framework overview
and positioning.

https://flor-rs.github.io/website/

> If you plan to actually use Flor, please read the documentation first.
>
> Flor is not a "reskinned" Rust GUI framework. Its window model, signal system, and widget extension approach are
> fundamentally different from common frameworks. Many patterns that are taken for granted in other frameworks may not
> work in Flor; conversely, some things that are awkward in other frameworks are deliberately designed this way in Flor.
>
> Guessing based on experience will likely lead to pitfalls. Spending half an hour browsing the documentation is far
> more cost-effective than spending half a day fixing code later.

## AI Assistance

Flor's layout DSL shares similarities with React-style functional UI expressions.
This enables large language models, even without specific Flor training, to leverage existing React, JSX, functional
component, and declarative UI experience to help users quickly generate, modify, and explain Flor interface layouts.

The documentation site provides LLM-oriented documentation entries for direct use when leveraging AI-assisted
development.

* Chinese: https://flor-rs.github.io/website/zh/ai.html
* English: https://flor-rs.github.io/website/ai.html

## Roadmap

Near-term directions:

* Example project organization
* Linux platform layer support
* Debugging support
* Independent debug control window / console
* More development-time auxiliary capabilities

Long-term directions:

* macOS support;
* Mobile platform support;
* More rendering backends;
* More complete cross-platform ecosystem.

## Vision

Flor's vision is to become one of the GUI options that people in the Rust ecosystem naturally think of and are willing
to try.

When developers need to write native desktop applications, we hope Flor can naturally enter the candidate list, just
like Qt for C++ or WPF for C#.

## License

This project adopts Mozilla Public License 2.0. Contributions and distributions must comply with MPL-2.0. See the
LICENSE file in the repository root directory for details.