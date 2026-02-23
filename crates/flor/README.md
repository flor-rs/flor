# Flor

Flor GUI 框架的核心 crate，提供视图系统、信号系统、布局引擎集成等核心功能。

## Features

| Feature          | 说明                   |
|------------------|----------------------|
| `direct2d`       | Direct2D 渲染后端 (默认启用) |
| `layout-flex`    | Flexbox 布局支持         |
| `layout-grid`    | Grid 布局支持            |
| `layout-block`   | Block 布局支持           |
| `svg`            | SVG 渲染支持             |
| `clipboard`      | 剪贴板支持                |
| `drag-drop`      | 拖放支持                 |
| `tray`           | 系统托盘支持               |
| `memory-font`    | 内存字体加载               |
| `monitor`        | 多显示器支持               |
| `hi-dpi`         | 高 DPI 支持             |
| `theme-change`   | 系统主题变化监听             |
| `signal-tracing` | 信号调试追踪               |

## 核心模块

- **view** - 视图系统 (View trait, ViewId, ViewStorage)
- **signal** - 响应式信号系统
- **render** - 渲染抽象层
- **windows** - 窗口管理
