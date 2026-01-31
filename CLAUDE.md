# Flor GUI Framework

## 项目概述

Flor 是一个 Rust GUI 框架，采用传统 Rust 生命周期管理，结合响应式设计思想。

## Workspace 结构

| Crate                  | 路径                       | 说明            |
|------------------------|--------------------------|---------------|
| flor                   | crates/flor              | 核心框架          |
| flor-base              | crates/base              | 基础类型和工具       |
| flor-macros            | crates/macros            | 过程宏           |
| flor-graphics-opengl   | crates/graphics/opengl   | OpenGL 渲染后端   |
| flor-graphics-direct2d | crates/graphics/direct2d | Direct2D 渲染后端 |
| flor-platform-windows  | crates/platform/windows  | Windows 平台支持  |

## 设计思想

- **生命周期一致性**: UI 组件与实例生命周期一致，遵循传统 Rust 所有权模型
- **响应式设计**: 结合现代响应式 UI 模式
- **统一事件模型**: Window 也是 View，所有 UI 组件共享统一的事件系统
- **view_event**: 处理所有 View 的公共事件，支持链式调用 (`into_view`)
- **view_storage**: 存储通用数据和公用事件模型
- **view trait**: 负责消息的传递与处理

## 常用命令

```bash
# 构建
cargo build

# 检查
cargo check

# 测试
cargo test

# 运行示例 (如果有)
cargo run --example <name>
```

## 主要依赖

- **taffy**: 布局引擎 (Flexbox/Grid)
- **windows**: Windows API 绑定
- **Direct2D / OpenGL**: 图形渲染
- **parking_lot**: 同步原语
- **slotmap**: 高效存储

## 代码规范

- 避免使用 `unwrap()` (workspace 已配置 clippy 警告)
- 使用 `Result` 进行错误处理
- 文档注释使用中文或英文均可

## View Trait 事件方法

### 鼠标事件

| 方法名                     | 说明   |
|-------------------------|------|
| `on_mouse_enter`        | 鼠标进入 |
| `on_mouse_move`         | 鼠标移动 |
| `on_mouse_leave`        | 鼠标离开 |
| `on_button_down`        | 左键按下 |
| `on_button_up`          | 左键抬起 |
| `on_click`              | 左键点击 |
| `on_double_click`       | 左键双击 |
| `on_right_button_down`  | 右键按下 |
| `on_right_button_up`    | 右键抬起 |
| `on_right_button_click` | 右键点击 |

### Resolver 工具方法

`flor::view::resolver` 模块提供的通用解析方法:

| 方法                         | 说明                        |
|----------------------------|---------------------------|
| `parse_color(value)`       | 解析颜色 (hex, tailwind, 关键字) |
| `parse_rounded(class)`     | 解析 rounded-* 类名           |
| `parse_font_weight(name)`  | 解析 font-* 类名中的字重          |
| `extract_bracket_value(s)` | 提取 `[value]` 中的值          |

控件的 `on_update_class` 实现应优先使用这些方法，而非重复编写解析逻辑

## 控件库

控件库在

H:\code\rustrover\flor-lys