# Resolver 派生宏参考

`#[derive(Resolver)]` 是 flor 框架中用于生成控件样式系统或其他参数解析系统（如布局）的辅助代码的过程宏。它为枚举类型自动生成完整的样式/配置管理体系，包括类型别名、链式方法扩展、状态管理结构体和
`StyleBuilder` 实现。

---

## 快速开始

```rust
use flor_macros::Resolver;
use flor_base::types::Color;

#[derive(Clone, Debug, Resolver)]
pub enum LabelStyle {  // 会自动为 Label 控件生成 StyleBuilder
    TextColor(Color),
    FontSize(f32),
    FontFamily(String),
}
```

应用 `#[derive(Resolver)]` 后，宏体会自动生成以下内容：

| 生成项                            | 描述                                                              |
|--------------------------------|-----------------------------------------------------------------|
| `LabelStyleKey`                | 样式属性键枚举，用于标识某种属性                                                |
| `LabelStyleUpdate`             | 响应式更新枚举，配合 `updated_view` 使用                                    |
| `LabelStyleComputed`           | 计算后的样式结构体 (所有字段自动转为 `Option<T>` ，供最终绘制使用)                       |
| `LabelStyleResolverExt`        | Resolver 链式方法特征 (如 `.text_color()`)                             |
| `LabelStyleResolver`           | `Resolver<LabelStyleKey, LabelStyle, LabelStyleComputed>` 的类型别名 |
| `impl LabelStyle::update_view` | 更新视图样式的辅助静态方法                                                   |
| `computed_label_style` 独立函数    | 计算最终 `Computed` 结果的计算函数                                         |
| `impl StyleBuilder for Label`  | **自动生成** `.style(...)` 链式方法 (通过默认规则推断绑定对哪个控件生成)                 |

---

## 控制与生成规则

Resolver 宏非常灵活，通过 `#[resolver(...)]` 可以控制产出的代码内容。

### 1. StyleBuilder 自动生成规则 (约定优于配置)

**默认行为**：
如果不携带任何参数，如果被修饰的枚举名以 `Style` 结尾，宏将自动推导：

- 将该 `EnumName` 移除 `Style` 后缀得到**控件名**（例如：`LabelStyle` ➡推测修饰的控件为 `Label` ）。如果是没有后缀，则使用原名推导。
- 接收样式的字段名为 `style`。
- 然后对它自动实现 `StyleBuilder` 特征。

### 2. 手动指定 StyleBuilder 规则

如果你遇到枚举名与想要绑定的控件不对应，或者控件内的接收字段不叫 `style`，可手动指定：

```rust
#[derive(Clone, Debug, Resolver)]
#[resolver(control = MyButton, style_field = specific_style)]
pub enum ButtonAppearance {
    BackgroundColor(Color),
}
```

这就会为 `MyButton` 结构体实现 `StyleBuilder`，赋值字段指向 `specific_style`。

### 3. 跳过 StyleBuilder 生成

```rust
#[derive(Clone, Debug, Resolver)]
#[resolver(builder = false)]
pub enum CustomStyle {
    Value(f32),
}
```

此时不会为控件生成对应的 `StyleBuilder` 代码。

### 4. 其它开关与参数配置

控制生成不同部件：

| 属性参数                  | 作用                                                                                |
|-----------------------|-----------------------------------------------------------------------------------|
| `update_view = false` | 不生成 `XxxUpdate` 枚举和 `update_view` 关联辅助方法                                          |
| `computed = false`    | 不生成 `XxxComputed` 结构体                                                             |
| `computed_fn = false` | 不生成 `computed_xxx_xxx` 计算辅助函数                                                     |
| `default = false`     | 不生成 Resolver 的 Default 实现                                                         |
| `data = Type`         | 指定 Resolver 计算完成后的 Data (即 D) 类型。未指定时，如果有生成 Computed 则指向 Computed 类型，否则不生成 Alias。 |

示例：只生成键枚举和方法扩展，不生成 Computed，配合内部 Taffy 样式系统使用：

```rust
#[derive(Clone, Debug, Resolver)]
#[resolver(update_view = false, computed = false, computed_fn = false, data = taffy::Style)]
pub enum Layout {
    Display(Display),
    Size(Size<Dimension>),
}
```

---

## 变体级别的跳过设置

### `#[resolver(skip_attr)]`

完全跳过该变体，既不生成 Key 枚举分支、也不生成 Computed 字段、计算逻辑或扩展方法。

```rust
#[derive(Clone, Debug, Resolver)]
pub enum MyStyle {
    Color(Color),
    #[resolver(skip_attr)]
    InternalData(SomeType),
}
```

### `#[resolver(skip_linkfn)]`

仅跳过生成**链式设置方法** (`ResolverExt` 里面的特征方法)。

```rust
#[derive(Clone, Debug, Resolver)]
pub enum MyStyle {
    Color(Color),
    #[resolver(skip_linkfn)]
    RawValue(u32),  // 有 Key 和 Computed等，但没有 .raw_value() 方法
}
```

---

## 运行机制与核心类型详解

假设定义如下枚举：

```rust
#[derive(Clone, Debug, Resolver)]
pub enum BoxStyle {
    BackgroundColor(Color),
    BorderRadius(f32),
}
```

### 1. 自动推导出的 Resolver 别名与链式操作

我们在控件内定义属性即可：

```rust
pub struct BoxControl {
    style: BoxStyleResolver, // <- 宏生成的类型别名
}
```

因为生成了 `BoxStyleResolverExt`，我们可以做链式设置：

```rust
let r: BoxStyleResolver = Resolver::default ()
.background_color(Color::RED)
.border_radius(4.0);
```

### 2. Update 枚举及 `update_view` 特征

生成了 `BoxStyleUpdate`，你可以通过消息通知到控件（比如从信号发出）。

```rust
#[derive(Clone, Debug)]
pub enum BoxStyleUpdate {
    BackgroundColor(ControlState, Color),
    BorderRadius(ControlState, f32),
}
```

控件内部处理更新时调用生成的静态方法应用更新：

```rust
BoxStyle::update_view( & mut self .style, update);
```

它内置会解析对应变体，更新底层的状态值映射表。

### 3. Computed 结构体计算

生成了 `BoxStyleComputed` 和带有性能克隆优化的 `computed_box_style` 函数：

```rust
let computed: BoxStyleComputed = computed_box_style( & unit_resolver, state, & self .style.variants);
```

**状态应用优先级规则**:

1. 首先提取出 `ControlState::Normal` 状态下的全套属性配置。
2. 提取并应用当前所处的特定的 `ControlState`（如 `Hover` 状态的配置）。若目标属性在新状态下被设置，会覆盖 Normal 的配置。
3. 对每个具体属性生成出最终计算好的 `Option<Value>` 到 Computed 中。

---

## 完整控件开发流结合的示例

```rust
use flor_macros::Resolver;
use flor::graphics::base::Color;
use flor::view::view_builder::style_builder::StyleBuilder;
use flor::view::resolver::Resolver as CoreResolver;

#[derive(Clone, Debug, Resolver)]
pub enum CardStyle {
    BackgroundColor(Color),
    BorderRadius(f32),
    TextColor(Color),
}

pub struct Card {
    style: CardStyleResolver, // 宏自动生成的类型别名
}

impl Card {
    pub fn new() -> Self {
        Self { style: CoreResolver::default() }
    }
}

// 控件会自动拥有 impl StyleBuilder<CardStyleResolver> for Card 了，可以直接使用。

fn usage() {
    let mut card = Card::new().style(|s| {
        s.background_color(Color::WHITE)
            .border_radius(8.0)
            .text_color(Color::BLACK)
    });
}
```
