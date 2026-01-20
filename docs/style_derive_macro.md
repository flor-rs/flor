# Style 派生宏参考

`#[derive(Style)]` 是 flor 框架中用于生成控件样式系统或其他参数的辅助代码的过程宏。它为枚举类型自动生成完整的样式管理体系，包括类型、trait
扩展、状态管理和 `StyleBuilder` 实现。

---

## 快速开始

```rust
use flor_macros::Style;
use flor::graphics::base::Color;

#[derive(Clone, Debug, Style)]
pub enum LabelStyle {  // 自动为 Label 控件生成 StyleBuilder
    TextColor(Color),
    FontSize(f32),
    FontFamily(String),
}
```

应用 `#[derive(Style)]` 后，宏会自动生成以下内容：

| 生成项                            | 描述                                              |
|--------------------------------|-------------------------------------------------|
| `LabelStyleKey`                | 样式属性键枚举                                         |
| `LabelStyleUpdate`             | 响应式更新枚举 (用于 `view_id.update_state`)             |
| `LabelStyleComputed`           | 计算后的样式结构体 (包含 `Option<T>` 字段)                   |
| `LabelStyleStateSelectorExt`   | `StateSelector` 的链式方法 trait                     |
| `LabelStyleStateSelector`      | `StateSelector<LabelStyleKey, LabelStyle>` 类型别名 |
| `impl LabelStyle::update_view` | 更新视图样式的辅助方法                                     |
| `impl StyleBuilder for Label`  | **自动生成** `.style()` 链式方法 (约定优于配置)               |

---

## 设计理念

Style 宏的设计遵循以下原则:

1. **约定优于配置**: 枚举名以 `Style` 结尾时，自动推导控件名并生成 `StyleBuilder`
2. **声明式**: 只需定义枚举变体，宏自动推导所有辅助代码
3. **类型安全**: 每个样式属性都有明确的类型，编译期检查
4. **状态感知**: 原生支持 Normal/Hover/Focus/Active/Disabled 状态
5. **链式 API**: 生成流畅的 builder 风格接口
6. **响应式**: 与 flor 信号系统无缝集成

---

## StyleBuilder 自动生成

### 约定规则

**默认行为**：如果枚举名以 `Style` 结尾，宏会自动：

- 去掉 `Style` 后缀得到控件名（如 `LabelStyle` → `Label`）
- 为该控件生成 `StyleBuilder<LabelStyleStateSelector>` 实现

```rust
// 枚举名: LabelStyle
// 自动推导控件名: Label
// 自动生成: impl StyleBuilder<LabelStyleStateSelector> for Label
#[derive(Clone, Debug, Style)]
pub enum LabelStyle {
    TextColor(Color),
}
```

### 显式指定控件名

当枚举名不符合 `XxxStyle` 命名规范时，使用 `#[style(control = ControlName)]`：

```rust
#[derive(Clone, Debug, Style)]
#[style(control = MyButton)]  // 显式指定为 MyButton 生成 StyleBuilder
pub enum ButtonAppearance {
    BackgroundColor(Color),
}
```

### 跳过 StyleBuilder 生成

使用 `#[style(builder = false)]` 跳过 StyleBuilder 生成，需手动实现：

```rust
#[derive(Clone, Debug, Style)]
#[style(builder = false)]  // 不生成 StyleBuilder
pub enum CustomStyle {
    Value(f32),
}

// 手动实现 StyleBuilder
impl StyleBuilder<CustomStyleStateSelector> for MyControl {
    fn style(mut self, style_fn: impl Fn(CustomStyleStateSelector) -> CustomStyleStateSelector) -> Self {
        self.style = style_fn(self.style);
        self
    }
}
```

---

## 生成代码详解

假设我们定义如下枚举:

```rust
#[derive(Clone, Debug, Style)]
pub enum ButtonStyle {
    BackgroundColor(Color),
    TextColor(Color),
    BorderRadius(f32),
    Padding(f32, f32),          // 多参数元组变体
    Shadow { blur: f32, offset: f32 },  // 命名字段变体
}
```

### 1. Key 枚举

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ButtonStyleKey {
    BackgroundColor,
    TextColor,
    BorderRadius,
    Padding,
    Shadow,
}
```

Key 枚举是样式属性的标识符，用于在 `StateSelector` 中索引样式值。

### 2. Update 枚举

```rust
#[derive(Clone, Debug)]
pub enum ButtonStyleUpdate {
    BackgroundColor(ControlState, Color),
    TextColor(ControlState, Color),
    BorderRadius(ControlState, f32),
    Padding(ControlState, f32, f32),
    Shadow { state: ControlState, blur: f32, offset: f32 },
}
```

Update 枚举用于响应式样式更新，通过 `view_id.update_state(Box::new(update))` 发送到视图。

### 3. Computed 结构体

```rust
#[derive(Clone, Debug, Default)]
pub struct ButtonStyleComputed {
    pub background_color: Option<Color>,
    pub text_color: Option<Color>,
    pub border_radius: Option<f32>,
    pub padding: Option<(f32, f32)>,
    pub shadow: Option<(f32, f32)>,
}
```

Computed 结构体存储计算后的最终样式值，所有字段都是 `Option<T>`。

### 4. StateSelectorExt Trait

```rust
pub trait ButtonStyleStateSelectorExt: Sized {
    fn background_color(self, value: Color) -> Self;
    fn text_color(self, value: Color) -> Self;
    fn border_radius(self, value: f32) -> Self;
    fn padding(self, arg0: f32, arg1: f32) -> Self;
    fn shadow(self, blur: f32, offset: f32) -> Self;
    fn compute_style(&self, state: ControlState) -> ButtonStyleComputed;
}
```

该 trait 为 `StateSelector` 添加链式设置方法和样式计算方法。

### 5. 类型别名

```rust
pub type ButtonStyleStateSelector = StateSelector<ButtonStyleKey, ButtonStyle>;
```

方便引用完整泛型类型。

---

## 使用方法

### 定义样式

在控件结构体中存储 `XxxStyleStateSelector`:

```rust
pub struct Label {
    view_id: ViewId,
    title: String,
    style: LabelStyleStateSelector,  // 由宏生成的类型别名
}
```

### 初始化样式

使用 `StateSelector::default()` 创建空样式容器:

```rust
impl Label {
    pub fn new(title: impl StringProp) -> Self {
        Self {
            view_id: ViewId::new(),
            title: title.make(),
            style: StateSelector::default(),
        }
    }
}
```

### 链式设置样式

通过状态切换方法 + 属性设置方法构建样式:

```rust
fn example() {
    // 设置 Normal 状态样式
    let style = StateSelector::default()
        .normal()                           // 切换到 Normal 状态
        .text_color(Color::BLACK)           // 设置文字颜色
        .font_size(16.0)                    // 设置字体大小
        .background(Background::Color(Color::WHITE));

    // 添加 Hover 状态样式
    let style = style
        .hover()                            // 切换到 Hover 状态
        .text_color(Color::BLUE)            // Hover 时文字变蓝
        .background(Background::Color(Color::rgba(0.9, 0.9, 0.9, 1.0)));    
}
```

### 获取计算后样式

在绘制时，根据当前控件状态获取最终样式:

```rust
fn on_draw(&mut self, render: &mut FlorRender, abs_location: (f32, f32), layout: Layout) -> Result<(), Error> {
    let control_state = self.view_id.control_state();

    // 计算样式: Normal + 当前状态覆盖
    let computed = self.style.compute_style(control_state);

    // 使用计算后的样式
    if let Some(color) = computed.text_color {
        // 应用文字颜色
    }
    if let Some(size) = computed.font_size {
        // 应用字体大小
    }
    Ok(())
}
```

### 响应式更新

处理来自信号系统的样式更新:

```rust
fn on_update_state(&mut self, state: Box<dyn Any>) {
    // 先处理其他状态更新...

    // 尝试处理样式更新 (LabelStyleUpdate 由 Style 宏生成)
    if let Ok(update) = state.downcast::<LabelStyleUpdate>() {
        LabelStyle::update_view(&mut self.style, *update);
    }
}
```

### `update_view` 方法详解

`update_view` 是 Style 宏为每个样式枚举自动生成的静态方法，用于将 `XxxStyleUpdate` 更新应用到样式选择器中。

**方法签名**:

```rust
impl LabelStyle {
    pub fn update_view(
        selector: &mut LabelStyleStateSelector,
        update: LabelStyleUpdate
    ) {
        // 根据 update 的变体，更新对应状态的对应属性
    }
}
```

**参数说明**:

| 参数       | 类型                           | 说明              |
|----------|------------------------------|:----------------|
| selector | `&mut XxxStyleStateSelector` | 要更新的样式选择器（可变引用） |
| update   | `XxxStyleUpdate`             | 包含控件状态和新值的更新对象  |

**工作原理**:

1. `XxxStyleUpdate` 枚举变体包含 `ControlState` 和实际值
2. `update_view` 解构更新对象，切换到对应状态，设置对应属性

**生成的代码示例** (以 `LabelStyle::TextColor` 为例):

```rust
impl LabelStyle {
    pub fn update_view(selector: &mut LabelStyleStateSelector, update: LabelStyleUpdate) {
        match update {
            LabelStyleUpdate::TextColor(state, color) => {
                *selector = selector.clone().switch_state(state).text_color(color);
            }
            LabelStyleUpdate::FontSize(state, size) => {
                *selector = selector.clone().switch_state(state).font_size(size);
            }
            // ... 其他变体
        }
    }
}
```

### 完整使用流程

使用 Style 宏开发控件样式的步骤：

**第一步：定义 Style 枚举**

```rust
#[derive(Clone, Debug, Style)]
pub enum LabelStyle {  // 命名为 XxxStyle，自动为 Xxx 生成 StyleBuilder
    TextColor(Color),
    FontSize(f32),
    FontFamily(String),
}
```

**第二步：在控件中声明样式字段**

```rust
pub struct Label {
    view_id: ViewId,
    title: String,
    style: LabelStyleStateSelector,  // 由宏生成的类型别名
}
```

**第三步：实现 `on_update_state` 处理样式更新**

```rust
fn on_update_state(&mut self, state: Box<dyn Any>) {
    // 处理样式更新
    if let Ok(update) = state.downcast::<LabelStyleUpdate>() {
        LabelStyle::update_view(&mut self.style, *update);
    }
}
```

**第四步：使用**

```rust
fn example() {
    // 创建控件并设置样式（StyleBuilder 已自动生成）
    let my_label = label("Hello World")
        .style(|s| s
            .normal()
            .text_color(Color::BLACK)
            .font_size(16.0)
            .hover()
            .text_color(Color::BLUE)
        );
}
```

---

## 状态系统

### 支持的状态

| 状态       | 方法            | 说明      |
|----------|---------------|---------|
| Normal   | `.normal()`   | 默认状态    |
| Hover    | `.hover()`    | 鼠标悬停    |
| Focus    | `.focus()`    | 获得焦点    |
| Active   | `.active()`   | 激活/按下状态 |
| Disabled | `.disabled()` | 禁用状态    |

### 状态继承规则

`compute_style(state)` 方法会:

1. 首先应用 Normal 状态的所有样式 (作为基础)
2. 如果当前状态不是 Normal，则用当前状态的样式覆盖

```rust
fn example() {
    // 示例
    let style = StateSelector::default()
        .normal().text_color(Color::BLACK).font_size(14.0)
        .hover().text_color(Color::BLUE);  // 只覆盖 text_color

    let computed = style.compute_style(ControlState::Hover);
    // computed.text_color = Some(Color::BLUE)   <- Hover 覆盖
    // computed.font_size = Some(14.0)           <- 继承自 Normal
}
```

---

## 高级特性

### #[style(skip_attr)]

完全跳过该变体，不生成任何代码:

```rust
#[derive(Clone, Debug, Style)]
pub enum MyStyle {
    Color(Color),
    #[style(skip_attr)]
    InternalData(SomeType),  // 不会生成 Key、setter 等
}
```

### #[style(skip_linkfn)]

只跳过链式方法生成，保留其他代码:

```rust
#[derive(Clone, Debug, Style)]
pub enum MyStyle {
    Color(Color),
    #[style(skip_linkfn)]
    RawValue(u32),  // 有 Key，但没有 .raw_value() 方法
}
```

这在需要手动控制某些属性设置逻辑时很有用。

---

## 与 StyleBuilder Trait 集成

为了支持更优雅的 API，控件通常实现 `StyleBuilder` trait:

```rust
impl StyleBuilder<LabelStyleStateSelector> for Label {
    fn style(mut self, style: impl Fn(LabelStyleStateSelector) -> LabelStyleStateSelector) -> Self {
        self.style = style(self.style);
        self
    }
}
```

使用示例:

```rust
fn example() {
    let label = label("Hello")
        .style(|s| s
            .normal()
            .text_color(Color::from_hex_str("1E293B").unwrap())
            .font_size(16.0)
            .hover()
            .text_color(Color::from_hex_str("3B82F6").unwrap())
        );
}
```

---

## 与原子类 (Atomic Classes) 集成

控件可通过 `on_update_class` 方法支持类似 Tailwind 的类名语法:

```rust
fn on_update_class(&mut self, control_state: ControlState, class: &str) -> Result<(), Error> {
    let mut selector = self.style.clone();
    selector = match control_state {
        ControlState::Normal => selector.normal(),
        ControlState::Hover => selector.hover(),
        // ...
    };

    // 解析类名并设置样式
    if let Some(rest) = class.strip_prefix("text-") {
        if let Some(color) = parse_color(rest) {
            self.style = selector.text_color(color);
            return Ok(());
        }
    }
    Ok(())
}
```

---

## 变体类型支持

Style 宏支持以下枚举变体形式:

| 变体形式    | 示例                                  | Computed 字段类型              |
|---------|-------------------------------------|----------------------------|
| 单元变体    | `Hidden`                            | (不生成字段)                    |
| 单参数元组变体 | `Color(Color)`                      | `Option<Color>`            |
| 多参数元组变体 | `Padding(f32, f32)`                 | `Option<(f32, f32)>`       |
| 命名字段变体  | `Shadow { blur: f32, offset: f32 }` | `Option<(f32, f32)>` (元组化) |

> **注意**: 命名字段变体在 Computed 结构体中会被转换为元组类型。

---

## 完整示例

```rust
use flor::graphics::base::{Color, FontWeight};
use flor::view::state_selector::StateSelector;
use flor::view::control_state::ControlState;
use flor_macros::Style;

// 定义样式枚举
#[derive(Clone, Debug, Style)]
pub enum CardStyle {
    BackgroundColor(Color),
    BorderColor(Color),
    BorderRadius(f32),
    TextColor(Color),
    FontWeight(FontWeight),
    #[style(skip_linkfn)]
    CustomData(String),  // 不生成链式方法
}

// 在控件中使用
pub struct Card {
    style: CardStyleStateSelector,
}

impl Card {
    pub fn new() -> Self {
        let style = StateSelector::default()
            .normal()
            .background_color(Color::WHITE)
            .border_color(Color::from_hex_str("E5E7EB").unwrap())
            .border_radius(8.0)
            .text_color(Color::from_hex_str("1F2937").unwrap())
            .hover()
            .border_color(Color::from_hex_str("3B82F6").unwrap());

        Self { style }
    }

    pub fn draw(&self, control_state: ControlState) {
        let computed = self.style.compute_style(control_state);

        if let Some(bg) = computed.background_color {
            // 绘制背景
        }
        if let Some(border_color) = computed.border_color {
            // 绘制边框
        }
        // ...
    }
}
```

---

## 相关文档

- [布局语法参考](./layout_syntax.md) - Tailwind 风格布局类名
- [装饰语法参考](./decoration_syntax.md) - 边框、圆角、背景等装饰类名
- [View 特征](../crates/flor/src/view/view.rs) - 控件基础特征
