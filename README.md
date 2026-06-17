**English** | [中文](README.zh-CN.md)

# Flor

[Documentation](https://flor-rs.github.io/website/) | [Quick Start](https://flor-rs.github.io/website/guide/startup) | [AI Guide](https://flor-rs.github.io/website/guide/ai) | [Examples](#examples-and-widget-library)

Flor is a modern, easy-to-use, lightweight, high-performance, general-purpose pure Rust native desktop GUI framework.

The documentation is already fairly complete, and the project is in an active evolution and refinement phase. If this
direction resonates with you, please consider starring the project to follow Flor's progress.

It features a reactive signal system, widget-based development model, and self-built platform/graphics layers, aiming to
provide a more natural, consistent, and suitable choice for long-term native GUI application development in Rust.

> Note: If you can read Chinese, please read the Chinese version first. It is the author's original draft; other
> language versions are mainly translated with models and are for reference.

> Stability Notice: Flor is still in its early stage. The current API is only preliminarily stable; before the official
> release to crates.io, it may change or be removed at any time without a deprecation transition.

## Documentation and Entry Points

The detailed documentation is published as a standalone documentation site. This README mainly keeps the framework
positioning, project stage, and quick trial path.

* Documentation site: https://flor-rs.github.io/website/
* Quick overview: https://flor-rs.github.io/website/guide/startup
* AI assistance guide: https://flor-rs.github.io/website/guide/ai
* API documentation: https://flor-rs.github.io/website/api/

If you plan to actually use Flor, reading the documentation first is recommended. Flor is not a "reskinned" Rust GUI
framework. Its window model, signal system, and widget extension approach are fundamentally different from common
frameworks. Many patterns that feel natural in other frameworks may not work in Flor; conversely, some things that are
awkward elsewhere are deliberately designed this way in Flor.

## Core Capabilities

Flor is positioned as a "framework kernel", not a finished GUI product with a complete built-in widget set and default
visual system. It needs to be used together with a widget library, or with widgets implemented by the user. At this
stage, what matters more than screenshots is clarifying the framework-level capability boundaries and the documentation
entry points for those capabilities.

Runnable examples will be added as the example projects are organized; these examples will mainly live in the `flor-lys`
example projects.

| Capability scope                                               |                                                         Current status | Documentation entry                                                                                                                                                                                                                                                                               |
|----------------------------------------------------------------|-----------------------------------------------------------------------:|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Pure Rust native GUI / self-built platform and graphics layers |                                              Implemented and expanding | [Feature overview](https://flor-rs.github.io/website/guide/features/overview), [Window creation and control](https://flor-rs.github.io/website/guide/use/window)                                                                                                                                  |
| Reactive signal system                                         |                                                            Implemented | [Cross-thread reactive signals](https://flor-rs.github.io/website/guide/use/signal)                                                                                                                                                                                                               |
| No forced context binding / cross-thread capability            |                                                            Implemented | [Cross-thread reactive signals](https://flor-rs.github.io/website/guide/use/signal), [Cross-thread window creation](https://flor-rs.github.io/website/guide/features/cross-thread-window-creation)                                                                                                |
| Multi-window creation and control                              |                                                            Implemented | [Window creation and control](https://flor-rs.github.io/website/guide/use/window)                                                                                                                                                                                                                 |
| Widget-based development model / View extension                |                                                            Implemented | [View Trait](https://flor-rs.github.io/website/guide/control/view-trait), [Control state](https://flor-rs.github.io/website/guide/use/control-state), [Developing widgets from scratch](https://flor-rs.github.io/website/guide/control/create-widget)                                            |
| Declarative UI DSL and utility-first styling                   |                                                            Implemented | [Framework DSL](https://flor-rs.github.io/website/guide/use/framework-dsl), [Layout](https://flor-rs.github.io/website/guide/use/builder/layout), [Atomic classes](https://flor-rs.github.io/website/guide/use/builder/class), [Style](https://flor-rs.github.io/website/guide/use/builder/style) |
| Small size, few dependencies, and high-performance direction   | Design goal; quantified data will be added before the official release | For now, see [Cargo.toml](https://github.com/flor-rs/flor/blob/main/Cargo.toml) and the documentation; benchmark / size data will be added later                                                                                                                                                  |
| Widget library                                                 |                                                        Being organized | Flor focuses on the framework kernel and is not expected to provide any built-in widgets; the widget library is maintained separately as [flor-lys](https://github.com/flor-rs/flor-lys), and example projects are being organized                                                                |

## Current Status

Flor is currently in a pre-release / evolution and refinement phase.

The core framework has entered a convergence phase after design validation, and the main architecture and direction are
now clear. Most current work will focus on adding new features and making the necessary evolution around those features
so they fit into the existing window, signal, view, layout, and widget extension models.

In other words, Flor is not a proof-of-concept prototype. The current main track is filling in capabilities and
polishing the ecosystem. Once evolved features become stable, they should mostly settle down instead of being repeatedly
overturned.

Current state:

* Core architecture: converged, and still being validated through real features and the widget library;
* Documentation site: already covers the main design, usage paths, widget extension model, and AI assistance entry
  point, and will keep evolving with the API;
* Official widget library: `flor-lys` is maintained separately as a baseline option for trying Flor directly;
* Example projects: being organized;
* Cross-platform support: Windows first, Linux support is a near-term focus, while macOS and mobile are long-term
  directions.

This stage is more suitable for people who are willing to observe the design evolution, run examples, submit bugs, and
participate in early feedback.

If you need fully stable production-grade commitments, it is better to wait for a future official release.

## Quick Start

Flor is still in its early stage and has not been published to crates.io. Please import it from the Git repositories for
now:

```toml
# cargo.toml
[dependencies]
flor = { git = "https://github.com/flor-rs/flor" }
flor-lys = { git = "https://github.com/flor-rs/flor-lys" }
```

Where:

* `flor` is the core framework, providing infrastructure such as the view system, signal system, window management,
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
            // Widget trees are usually provided by widget libraries, such as flor-lys.
            vec![]
        })?;

    FlorGui.event_loop()?;
    Ok(())
}
```

For more complete usage, please refer to the documentation site, especially the sections on windows, signals, widget
state, and the DSL.

## Examples and Widget Library

Example projects are being organized. At this stage, the suggested path is:

* Read the documentation site's [Quick overview](https://flor-rs.github.io/website/guide/startup) first;
* Then run a minimal window with `flor` + `flor-lys`;
* If you want to write real widgets, read the widget extension sections in the documentation site and refer to the
  implementation of `flor-lys`.

> `flor-lys` still needs some time before its official release. It is currently in a stage where old and new code are
> mixed and capabilities are still being filled in. Releasing it too early would only give users an incomplete and
> hard-to-judge experience, rather than helping them understand or use Flor.

Flor is a **framework**, not a widget library.

It provides the infrastructure required by GUI applications, including the view system, signal system, window
management, rendering, layout integration, and event dispatch. The official widget
library [flor-lys](https://github.com/flor-rs/flor-lys) is maintained as a separate project, giving users a baseline
option for trying Flor directly.

`flor-lys` also has another important role: it continuously serves as a validation ground for the Flor framework itself.
The author will keep participating in `flor-lys` maintenance, using a real widget library to validate Flor's
architecture, API, and extension model, so Flor is not just a framework that works in theory.

The separation between framework and widget library is not only about clearer responsibilities. It is also meant to
encourage individuals, organizations, and companies to create their own brand styles and widget libraries when
maintaining their own software. Flor's `View` trait design makes widget source code relatively portable and adaptable;
in many cases, maintaining a custom widget library does not require understanding the whole framework from scratch.
Referencing, copying, and modifying existing widgets is enough to gradually get started.

This is exactly what the framework's name means:

*Flowers bloom in succession, each with its own brilliance.*

## Suitable Use Cases

Flor is suitable for:

* Native Rust desktop applications;
* Small tools with fast startup and small footprint;
* Multi-window applications;
* High-performance visualization, trading charts, waveforms, monitoring dashboards, and other low-latency rendering
  scenarios;
* GUI projects that require long-term maintenance, multi-person collaboration, and clear state flow;
* Teams that want to maintain custom widget libraries or brand-specific widget systems.

## AI Assistance

Flor's layout DSL is similar to React-style functional UI expressions. This allows large language models, even without
specific training on Flor, to use existing experience with React, JSX, functional components, and declarative UI to help
users quickly generate, modify, and explain Flor interface layouts.

The documentation site provides LLM-oriented documentation entry points for direct use when developing with AI
assistance.

* Chinese: https://flor-rs.github.io/website/zh/guide/ai
* English: https://flor-rs.github.io/website/guide/ai

## Roadmap

Near-term directions:

* Example project organization
* Linux platform layer support
* Debugging support
* Independent debug control window / console
* More development-time auxiliary capabilities

Long-term directions:

* macOS support: requires real devices and long-term maintenance resources, and is not promised for the near term;
* Mobile platform support;
* More rendering backends;
* A more complete cross-platform ecosystem.

About macOS: it is not on the near-term roadmap mainly because the author currently does not have a macOS device for
development and validation. Apple's licensing and ecosystem restrictions also make third-party Hackintosh setups
unsuitable as a support foundation; on the author's current computer, running a macOS virtual machine is slow enough
that even basic validation of the window-layer APIs is difficult to move forward. Before there is a truly maintainable
environment, Flor will not make an easy promise on macOS support.

## Why Flor Exists

Rust's ownership model brings unique challenges to GUI development. Many GUI frameworks can easily run into these issues
in complex applications:

* Widget trees, state, and lifecycles pull against each other;
* Windows and contexts are tightly bound, making multi-window development unnatural;
* Cross-widget access and state updates require many detours;
* Even simple tools may pull in heavy dependencies through large window layers, graphics layers, or abstraction crates;
* The boundary between application developers and widget authors is not clear enough.

Flor was born from the frustration of trying one Rust GUI framework after another and still not finding a satisfactory
answer.

At first, I just wanted a Rust GUI framework that felt good to use: no forced context binding, no restrictions
everywhere in multi-window scenarios, no heavy dependencies for simple tools, and no long wrestling match with the
borrow checker just to update state or access widgets.

But after repeated design validation and refactoring, Flor slowly became what it is today:

It does not only try to solve one narrow scenario. It tries to find the best answer, at least for the author, among
small size, high performance, easy-to-use APIs, multi-window support, widget extensibility, and long-term maintenance.

In other words, Flor is a Rust GUI framework that wants everything, and more.

This was not a slogan deliberately set at the beginning. It is the direction that naturally emerged after continuous
trial, error, rejection, and rebuilding.

## Vision

Flor's vision is to become one of the GUI options that people in the Rust ecosystem naturally think of and are willing
to try.

When developers need to write native desktop applications, I hope Flor can naturally enter the candidate list, just like
Qt for C++ or WPF for C#.

## License

This project is licensed under the Mozilla Public License 2.0. Contributions and distribution must comply with MPL-2.0.
See the LICENSE file in the repository root for details.
