# Flor Macros

Flor 框架的过程宏 crate。

## 提供的宏

### `#[derive(Style)]`

为枚举类型自动生成样式系统代码，包括：

- `XxxStyleKey` - 样式属性键枚举
- `XxxStyleUpdate` - 响应式更新枚举
- `XxxStyleComputed` - 计算后的样式结构体
- `XxxStyleStateSelectorExt` - 链式方法 trait
- `StyleBuilder` 实现

详见 [Style 派生宏文档](../../docs/style_derive_macro.md)
