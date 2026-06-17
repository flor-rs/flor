[English](README.md) | **中文**

# Flor

[Documentation](https://flor-rs.github.io/website/zh/) | [Quick Start](https://flor-rs.github.io/website/zh/guide/startup) | [AI Guide](https://flor-rs.github.io/website/zh/guide/ai) | [Examples](#示例与控件库)

Flor 是一个现代、易用、小体积、高性能、泛用型的纯 Rust 原生桌面 GUI 框架。

文档已经较完整，项目处于持续演进完善期。如果你认同这个方向，欢迎 star 关注 Flor 的演进。

它采用响应式信号系统、控件式开发模型和自研平台/图形层，目标是在 Rust 中提供一个更自然、更一致、更适合长期应用开发的原生 GUI
选择。

> 译注：如果你会中文，请优先阅读本中文版本；这是作者原稿，其他语言版本主要通过模型翻译，仅供参考。

> 稳定性声明：Flor 仍处于早期阶段。当前 API 只是初步稳定；在正式发布到 crates.io 之前，可能随时变更或移除，不会使用弃用标签做过渡。

## 文档与入口

详细文档已独立发布。本 README 主要保留框架定位、项目阶段和快速试用路径。

* 文档站：https://flor-rs.github.io/website/zh/
* 快速了解：https://flor-rs.github.io/website/zh/guide/startup
* AI 辅助指南：https://flor-rs.github.io/website/zh/guide/ai
* API 文档：https://flor-rs.github.io/website/zh/api/

如果你打算真正使用 Flor，建议先看文档。Flor 不是一个“换皮”的 Rust GUI 框架，它的窗口模型、信号系统、控件扩展方式都和常见框架有本质区别。很多在其他框架里理所当然的写法，在
Flor 里可能走不通；反过来，一些在其他框架里很别扭的事情，在 Flor 里是刻意设计成这样的。

## 现在的状态

Flor 目前处于 pre-release / 演进完善期。

核心框架已经进入设计验证后的收敛期，项目架构和主要路线已经摸清。现在更多工作会集中在加新功能，并围绕新功能做必要演进，让它们融入既有的窗口、信号、视图、布局和控件扩展模型。

换句话说，Flor 不是停留在概念验证阶段的原型。当前主线是补齐能力和打磨生态；功能经过演进并稳定后，会尽量定型，而不是频繁推翻。

当前情况：

* 核心架构：已经收敛，继续通过真实功能和控件库验证；
* 文档站：已经覆盖主要设计、使用路径、控件扩展和 AI 辅助入口，会随 API 继续更新；
* 官方控件库：`flor-lys` 独立维护，作为可直接尝试 Flor 的基线选择；
* 示例工程：正在整理中；
* 跨平台支持：Windows 优先，Linux 支持是近期重点，macOS 和移动端属于长期方向。

macOS 暂不进入近期路线，主要原因是作者目前没有可用于开发和验证的 macOS 设备。Apple 的授权和生态限制也使第三方黑苹果方案不适合作为支持基础；而在作者现有电脑上运行
macOS 虚拟机时，性能卡顿到连窗口层 API 的基础验证都难以推进。因此在具备真实可维护环境之前，Flor 不会轻易承诺 macOS 支持。

这个阶段更适合愿意观察设计演进、运行示例、提交 bug、参与早期反馈的人。

如果你需要完全稳定的生产级承诺，建议等待后续正式版本。

## 核心能力

Flor 的定位是“框架内核”，不是“自带完整控件库和默认视觉体系的成品
GUI”。它需要搭配控件库使用，或由用户自行实现控件。因此，现阶段比截图更重要的是说明框架层面的能力边界，以及这些能力在文档中的入口。

示例工程整理完成后，会继续补上可运行示例；这些示例会主要落在 `flor-lys` 的示例工程里。

| 能力范围                     |             当前状态 | 文档入口                                                                                                                                                                                                                                                                                 |
|--------------------------|-----------------:|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 纯 Rust 原生 GUI / 自研平台与图形层 |         已实现并持续扩展 | [Feature 总览](https://flor-rs.github.io/website/zh/guide/features/overview), [窗口创建与控制](https://flor-rs.github.io/website/zh/guide/use/window)                                                                                                                                         |
| 响应式信号系统                  |              已实现 | [跨线程的响应式信号](https://flor-rs.github.io/website/zh/guide/use/signal)                                                                                                                                                                                                                   |
| 不强制绑定上下文 / 跨线程能力         |              已实现 | [跨线程的响应式信号](https://flor-rs.github.io/website/zh/guide/use/signal), [跨线程窗口创建](https://flor-rs.github.io/website/zh/guide/features/cross-thread-window-creation)                                                                                                                      |
| 多窗口创建与控制                 |              已实现 | [窗口创建与控制](https://flor-rs.github.io/website/zh/guide/use/window)                                                                                                                                                                                                                     |
| 控件式开发模型 / View 扩展        |              已实现 | [View Trait](https://flor-rs.github.io/website/zh/guide/control/view-trait), [控件状态](https://flor-rs.github.io/website/zh/guide/use/control-state), [从零开发控件](https://flor-rs.github.io/website/zh/guide/control/create-widget)                                                        |
| 声明式 UI DSL 与工具类样式        |              已实现 | [框架 DSL](https://flor-rs.github.io/website/zh/guide/use/framework-dsl), [布局](https://flor-rs.github.io/website/zh/guide/use/builder/layout), [原子类](https://flor-rs.github.io/website/zh/guide/use/builder/class), [样式](https://flor-rs.github.io/website/zh/guide/use/builder/style) |
| 小体积、少依赖、高性能方向            | 设计目标，待正式发布前补量化数据 | 当前先看 [Cargo.toml](https://github.com/flor-rs/flor/blob/main/Cargo.toml) 和文档；后续会补 benchmark / size 数据                                                                                                                                                                                 |
| 控件库                      |              整理中 | Flor 负责框架内核，预计后续也不会提供任何内置控件；控件库由 [flor-lys](https://github.com/flor-rs/flor-lys) 独立维护，示例工程正在整理                                                                                                                                                                                       |

## 快速开始

目前 Flor 仍处于早期阶段，尚未发布到 crates.io。请先通过 Git 仓库引入：

```toml
# cargo.toml
[dependencies]
flor = { git = "https://github.com/flor-rs/flor" }
flor-lys = { git = "https://github.com/flor-rs/flor-lys" }
```

其中：

* `flor` 是核心框架，提供视图系统、信号系统、窗口管理、渲染、布局集成和事件派发等基础设施；
* `flor-lys` 是作者维护的官方控件库，用来提供一个开箱可用的保底选择。

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
            // 控件树通常由控件库提供，例如 flor-lys
            vec![]
        })?;

    FlorGui.event_loop()?;
    Ok(())
}
```

更完整的使用方式请以文档站为准，尤其是窗口、信号、控件状态和 DSL 相关章节。

## 示例与控件库

示例工程正在整理中。现阶段建议按这个顺序尝试：

* 先看文档站的 [快速了解](https://flor-rs.github.io/website/zh/guide/startup)；
* 再用 `flor` + `flor-lys` 跑最小窗口；
* 如果要写真实控件，阅读文档站的控件扩展章节，并参考 `flor-lys` 的实现。

> `flor-lys` 还需要一些时间才会正式发布。目前它处在新旧代码混合和能力补齐阶段，贸然发布只会给使用者带来不完整、难判断的体验，不会真正帮助大家理解或使用
> Flor。

Flor 是一个**框架**，不是控件库。

它负责提供 GUI
应用所需的基础设施，包括视图系统、信号系统、窗口管理、渲染、布局集成和事件派发等能力。官方控件库 [flor-lys](https://github.com/flor-rs/flor-lys)
会作为独立项目维护，用来给使用者提供一个可以直接尝试 Flor 的保底选择。

`flor-lys` 还有另一个重要作用：它会持续作为 Flor 框架自身的验证场。作者会一直参与 `flor-lys` 的维护，用真实控件库反过来验证
Flor 的架构、API 和扩展模型，确保 Flor 不只是一个理论上成立的框架。

框架与控件库分离并不只是为了让职责更清晰，也是为了鼓励个人、组织和公司在维护自己的软件时，创建属于自己的品牌风格和控件库。Flor
的 `View` trait 设计使控件源码具有较高的可迁移性和可改造性；很多时候，维护自定义控件库并不需要从零理解整个框架，只要能参考、复制、修改已有控件，就可以逐步上手。

这正是这个框架名字的寓意：

*繁花继开，各放异彩。*

## 适合什么

Flor 适合这些方向：

* 原生 Rust 桌面应用；
* 小体积、快启动的工具软件；
* 多窗口应用；
* 高性能可视化、行情、波形、监控面板等低延迟绘制场景；
* 需要长期维护、多人协作、状态流清晰的 GUI 项目；
* 想要维护自定义控件库或品牌控件体系的团队。

## 为什么有 Flor

Rust 的所有权模型为 GUI 开发带来了独特挑战。很多 GUI 框架在复杂应用中容易遇到这些问题：

* 控件树、状态和生命周期互相牵制；
* 窗口和上下文强绑定，多窗口开发体验不自然；
* 跨控件访问和状态更新需要大量绕路；
* 简单工具也可能被大型窗口层、图形层或抽象 crate 拉入庞大依赖；
* 终端应用开发者和控件作者的职责边界不够清晰。

Flor 诞生于作者尝试了一个又一个 Rust GUI 框架后依然找不到满意方案的挫败感。

一开始，我只是想要一个自己用起来顺手的 Rust GUI 框架：不要被上下文强行绑定，不要在多窗口上处处受限，不要为了简单工具拉入过重的依赖，也不要让状态更新和控件访问变成和借用检查器的长期拉扯。

但在一次又一次的设计验证和重构之后，Flor 慢慢变成了现在这个样子：

它不只想解决某个单一场景的问题，而是希望在小体积、高性能、易用 API、多窗口、控件扩展和长期维护之间，找到一个对于作者来说最完美的答案。

换句话说，Flor 是一个“既要，又要，还要”的 Rust GUI 框架。

这不是一开始刻意定下的口号，而是项目在不断试错、推翻和重构之后，自然长出来的方向。

## AI 辅助

Flor 的布局 DSL 与 React 系的函数式 UI 表达有相似之处。
这使得大语言模型即使没有专门训练过 Flor，也可以借助已有的 React、JSX、函数式组件和声明式 UI 经验，帮助用户快速生成、修改和解释
Flor 的界面布局。

文档站有提供面向 LLM 的文档入口，方便在使用 AI 辅助开发时直接提供给模型阅读。

* 中文：https://flor-rs.github.io/website/zh/guide/ai
* English：https://flor-rs.github.io/website/guide/ai

## 路线

近期方向：

* 示例工程整理
* Linux 平台层支持
* 调试支持
* 独立 debug 控制窗口 / 控制台
* 更多开发期辅助能力

长期方向：

* macOS 支持：需要真实设备和长期维护资源，目前暂不承诺近期支持；
* 移动端支持；
* 更多渲染后端；
* 更完整的跨平台生态。

## 愿景

Flor 的愿景是成为 Rust 生态中大家默认会想到、愿意尝试的 GUI 选项之一。

当开发者需要编写原生桌面应用时，希望 Flor 能自然进入候选，就像 C++ 的 Qt、C# 的 WPF 那样。

## License

本项目采用 Mozilla Public License 2.0。贡献和分发需遵循 MPL-2.0。详见仓库根目录的 LICENSE 文件。
