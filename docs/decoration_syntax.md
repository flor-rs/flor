# Flor Decoration 语法参考

本文档列出 `Decoration` 支持的所有装饰类名。语法借鉴了 Tailwind CSS。

---

## 快速开始

```rust
use flor::view::state_selector::{Decoration, DecorationState};

// 解析类名
let deco = Decoration::parse(& [
"bg-white",
"border",
"border-gray-300",
"rounded-lg",
"hover:bg-gray-50",
]);

// 直接访问各状态样式 (所有状态都继承 normal)
let border_width = deco.normal.border_width;
let hover_bg = deco.hover.background_color;

// 或通过 get() 方法
let style = deco.get(DecorationState::Active);
```

---

## 设计特点

- **所有状态都有值**：`normal`, `hover`, `focus`, `active`, `disabled` 都是 `DecorationStyle`，不是 `Option`
- **自动继承**：非 normal 状态自动继承 normal，然后用自己的值覆盖
- **无需判空**：直接访问任何状态的样式

```rust
// 所有状态都可以直接访问，无需 unwrap
let hover_bg = deco.hover.background_color;  // Option<Color>，但字段本身存在
let focus_radius = deco.focus.border_radius; // 继承自 normal
```

---

## 1. Border 边框

| 类名模式                     | 效果      |
|--------------------------|---------|
| `border`                 | 四边 1px  |
| `border-0/2/4/8`         | 四边 N px |
| `border-[Npx]`           | 任意值     |
| `border-t/b/l/r-*`       | 单边      |
| `border-x/y-*`           | 左右 / 上下 |
| `border-{color}-{shade}` | 边框颜色    |

---

## 2. Rounded 圆角

| 类名                            | 效果 (px)        |
|-------------------------------|----------------|
| `rounded-none`                | 0              |
| `rounded-sm/md/lg/xl/2xl/3xl` | 2/6/8/12/16/24 |
| `rounded`                     | 4              |
| `rounded-full`                | 9999           |
| `rounded-[Npx]`               | 任意值            |
| `rounded-t/b/l/r-*`           | 双角             |
| `rounded-tl/tr/bl/br-*`       | 单角             |

---

## 3. Background 背景色

| 类名模式                         | 示例                        |
|------------------------------|---------------------------|
| `bg-{color}-{shade}`         | `bg-white`, `bg-blue-500` |
| `bg-transparent/black/white` | 特殊颜色                      |
| `bg-[#hex]`                  | `bg-[#f5f5f5]`            |

---

## 4. Opacity 透明度

| 类名模式               | 效果            |
|--------------------|---------------|
| `opacity-0/50/100` | 百分比           |
| `opacity-[N]`      | 任意值 (0.0-1.0) |

---

## 5. 状态前缀

| 前缀          | 状态         |
|-------------|------------|
| (无)         | `normal`   |
| `hover:`    | `hover`    |
| `focus:`    | `focus`    |
| `active:`   | `active`   |
| `disabled:` | `disabled` |

---

## 6. API 参考

### Decoration

```rust
pub struct Decoration {
    pub normal: DecorationStyle,
    pub hover: DecorationStyle,    // 继承 normal 后覆盖
    pub focus: DecorationStyle,
    pub active: DecorationStyle,
    pub disabled: DecorationStyle,
}

impl Decoration {
    fn new() -> Self;
    fn parse(classes: &[&str]) -> Self;
    fn parse_with_rem(classes: &[&str], rem_px: f32) -> Self;
    fn get(&self, state: DecorationState) -> &DecorationStyle;
    fn parse_single(&mut self, class: &str);
}
```

### DecorationStyle

```rust
pub struct DecorationStyle {
    pub border_width: BorderRect,
    pub border_color: Option<Color>,
    pub border_radius: CornerRadius,
    pub background_color: Option<Color>,
    pub opacity: Option<f32>,
}
```

### BorderRect / CornerRadius

```rust
pub struct BorderRect {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}
```
