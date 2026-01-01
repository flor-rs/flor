# Flor Layout Class 语法参考

本文档列出 `load_classes` 方法支持的所有布局类名。语法借鉴了 Tailwind CSS，但**并非完全复刻**。请严格按照本文档使用。

---

## 值语法说明

在以下各节中，`*` 代表可接受的值，支持以下格式：

| 格式                     | 示例                       | 说明                      |
|------------------------|--------------------------|-------------------------|
| **数字 (Spacing Scale)** | `4`                      | 转换为 `4 × 4 = 16px`      |
| **像素值**                | `[100px]`                | 直接使用像素值                 |
| **百分比**                | `[50%]`                  | 百分比值                    |
| **rem**                | `[1.5rem]`               | 转换为 `1.5 × 16 = 24px`   |
| **pt**                 | `[12pt]`                 | 转换为 `12 × 1.333 ≈ 16px` |
| **分数**                 | `1/2`, `1/3`             | 转换为百分比 (50%, 33.33%)    |
| **关键字**                | `auto`, `full`, `screen` | 特殊语义值                   |

### 单位转换公式

| 单位    | 转换公式              | 说明                                           |
|-------|-------------------|----------------------------------------------|
| `rem` | `1rem = rem_px`   | 可通过 `set_rem_px()` 配置，默认 16px                |
| `pt`  | `1pt = dpi/72 px` | 基于 StateSelector 的 DPI 设置，默认 96dpi → 1.333px |

> **注意**: 带 `[]` 的值表示任意值语法 (Arbitrary Value)。

### 配置单位转换

```rust
// 设置 rem 基准值 (1rem = 多少 px)
window_id.set_rem_px(16.0);
```

> DPI框架集成自动读取，默认读取窗口，如果启用了“monitor”特性，则使用“monitor”的扩展api读取DPI. (DPI 影响 pt 单位转换)

---

## 1. Display 显示模式

| 类名       | 效果               | Feature Flag   |
|----------|------------------|----------------|
| `flex`   | `Display::Flex`  | `layout-flex`  |
| `block`  | `Display::Block` | `layout-block` |
| `grid`   | `Display::Grid`  | `layout-grid`  |
| `hidden` | `Display::None`  | -              |

---

## 2. Position 定位

| 类名         | 效果                   |
|------------|----------------------|
| `relative` | `Position::Relative` |
| `absolute` | `Position::Absolute` |

---

## 3. Box Sizing 盒模型

| 类名            | 效果                      |
|---------------|-------------------------|
| `box-border`  | `BoxSizing::BorderBox`  |
| `box-content` | `BoxSizing::ContentBox` |

---

## 4. Overflow 溢出

### 4.1 同时设置 X/Y 轴

| 类名                 | 效果               |
|--------------------|------------------|
| `overflow-visible` | X/Y 均为 `Visible` |
| `overflow-hidden`  | X/Y 均为 `Hidden`  |
| `overflow-clip`    | X/Y 均为 `Clip`    |
| `overflow-scroll`  | X/Y 均为 `Scroll`  |

### 4.2 单独设置 X 轴

| 类名                   | 效果            |
|----------------------|---------------|
| `overflow-x-visible` | X 轴 `Visible` |
| `overflow-x-hidden`  | X 轴 `Hidden`  |
| `overflow-x-clip`    | X 轴 `Clip`    |
| `overflow-x-scroll`  | X 轴 `Scroll`  |

### 4.3 单独设置 Y 轴

| 类名                   | 效果            |
|----------------------|---------------|
| `overflow-y-visible` | Y 轴 `Visible` |
| `overflow-y-hidden`  | Y 轴 `Hidden`  |
| `overflow-y-clip`    | Y 轴 `Clip`    |
| `overflow-y-scroll`  | Y 轴 `Scroll`  |

---

## 5. Sizing 尺寸

### 5.1 Width / Height

| 类名模式     | 效果     | 支持的值                                                                   |
|----------|--------|------------------------------------------------------------------------|
| `w-*`    | 设置宽度   | `auto`, `full`, `screen`, `fit`, `min`, `max`, 数字, 分数, `[Npx]`, `[N%]` |
| `h-*`    | 设置高度   | 同上                                                                     |
| `size-*` | 同时设置宽高 | 同上                                                                     |

**示例**: `w-full`, `h-screen`, `w-1/2`, `h-[100px]`, `size-10`

### 5.2 Min/Max Size

| 类名模式      | 效果   |
|-----------|------|
| `min-w-*` | 最小宽度 |
| `min-h-*` | 最小高度 |
| `max-w-*` | 最大宽度 |
| `max-h-*` | 最大高度 |

**示例**: `min-w-0`, `max-h-full`, `min-h-[50px]`

---

## 6. Spacing 间距

### 6.1 Padding 内边距

| 类名模式   | 效果    |
|--------|-------|
| `p-*`  | 四边内边距 |
| `px-*` | 左右内边距 |
| `py-*` | 上下内边距 |
| `pt-*` | 上内边距  |
| `pb-*` | 下内边距  |
| `pl-*` | 左内边距  |
| `pr-*` | 右内边距  |

**支持的值**: 数字, `full`, `[Npx]`, `[N%]`

**示例**: `p-4`, `px-2`, `pt-[10px]`, `pl-1/2`

### 6.2 Margin 外边距

| 类名模式   | 效果    |
|--------|-------|
| `m-*`  | 四边外边距 |
| `mx-*` | 左右外边距 |
| `my-*` | 上下外边距 |
| `mt-*` | 上外边距  |
| `mb-*` | 下外边距  |
| `ml-*` | 左外边距  |
| `mr-*` | 右外边距  |

**支持的值**: `auto`, 数字, `full`, `screen`, 分数, `[Npx]`, `[N%]`

**示例**: `m-4`, `mx-auto`, `mt-2`, `mb-[20px]`

---

## 7. Inset 定位偏移

用于 `absolute` 或 `relative` 定位元素。

| 类名模式        | 效果   |
|-------------|------|
| `inset-*`   | 四边偏移 |
| `inset-x-*` | 左右偏移 |
| `inset-y-*` | 上下偏移 |
| `top-*`     | 上偏移  |
| `bottom-*`  | 下偏移  |
| `left-*`    | 左偏移  |
| `right-*`   | 右偏移  |

**支持的值**: `auto`, 数字, `full`, `screen`, 分数, `[Npx]`, `[N%]`

**示例**: `inset-0`, `top-4`, `left-1/2`, `right-[10px]`

---

## 8. Flexbox

> 需要 Feature Flag: `layout-flex`

### 8.1 Flex Direction

| 类名                 | 效果                             |
|--------------------|--------------------------------|
| `flex-row`         | `FlexDirection::Row`           |
| `flex-row-reverse` | `FlexDirection::RowReverse`    |
| `flex-col`         | `FlexDirection::Column`        |
| `flex-col-reverse` | `FlexDirection::ColumnReverse` |

### 8.2 Flex Wrap

| 类名                  | 效果                      |
|---------------------|-------------------------|
| `flex-wrap`         | `FlexWrap::Wrap`        |
| `flex-wrap-reverse` | `FlexWrap::WrapReverse` |
| `flex-nowrap`       | `FlexWrap::NoWrap`      |

### 8.3 Flex Grow

| 类名模式     | 效果                 |
|----------|--------------------|
| `grow`   | `flex-grow: 1`     |
| `grow-0` | `flex-grow: 0`     |
| `grow-*` | 自定义值, 如 `grow-[2]` |

### 8.4 Flex Shrink

| 类名模式       | 效果                     |
|------------|------------------------|
| `shrink`   | `flex-shrink: 1`       |
| `shrink-0` | `flex-shrink: 0`       |
| `shrink-*` | 自定义值, 如 `shrink-[0.5]` |

### 8.5 Flex Basis

| 类名模式      | 效果            |
|-----------|---------------|
| `basis-*` | 设置 flex-basis |

**支持的值**: `auto`, 数字, 分数, `[Npx]`, `[N%]`

**示例**: `basis-1/4`, `basis-[100px]`, `basis-auto`

---

## 9. Grid

> 需要 Feature Flag: `layout-grid`

### 9.1 Grid Auto Flow

| 类名                    | 效果                          |
|-----------------------|-----------------------------|
| `grid-flow-row`       | `GridAutoFlow::Row`         |
| `grid-flow-col`       | `GridAutoFlow::Column`      |
| `grid-flow-row-dense` | `GridAutoFlow::RowDense`    |
| `grid-flow-col-dense` | `GridAutoFlow::ColumnDense` |

### 9.2 Grid Placement

#### Row 行定位

| 类名模式          | 效果   |
|---------------|------|
| `row-start-*` | 行起始线 |
| `row-end-*`   | 行结束线 |
| `row-span-*`  | 跨行数  |

**支持的值**: 整数, 如 `row-start-1`, `row-span-2`, `row-end-[3]`

#### Column 列定位

| 类名模式          | 效果   |
|---------------|------|
| `col-start-*` | 列起始线 |
| `col-end-*`   | 列结束线 |
| `col-span-*`  | 跨列数  |

**支持的值**: 整数, 如 `col-start-1`, `col-span-3`, `col-end-[4]`

---

## 10. Gap 间隙

> 需要 Feature Flag: `layout-flex` 或 `layout-grid`

| 类名模式      | 效果       |
|-----------|----------|
| `gap-*`   | 行列间隙     |
| `gap-x-*` | 列间隙 (水平) |
| `gap-y-*` | 行间隙 (垂直) |

**支持的值**: 数字, `full`, `[Npx]`, `[N%]`

**示例**: `gap-4`, `gap-x-2`, `gap-y-[10px]`

---

## 11. Alignment 对齐

> 需要 Feature Flag: `layout-flex` 或 `layout-grid`

### 11.1 Align Items (交叉轴子项对齐)

| 类名               | 效果                     |
|------------------|------------------------|
| `items-start`    | `AlignItems::Start`    |
| `items-end`      | `AlignItems::End`      |
| `items-center`   | `AlignItems::Center`   |
| `items-baseline` | `AlignItems::Baseline` |
| `items-stretch`  | `AlignItems::Stretch`  |

### 11.2 Align Self (单个子项交叉轴对齐)

| 类名              | 效果                    |
|-----------------|-----------------------|
| `self-start`    | `AlignSelf::Start`    |
| `self-end`      | `AlignSelf::End`      |
| `self-center`   | `AlignSelf::Center`   |
| `self-baseline` | `AlignSelf::Baseline` |
| `self-stretch`  | `AlignSelf::Stretch`  |

### 11.3 Justify Content (主轴内容对齐)

| 类名                | 效果                             |
|-------------------|--------------------------------|
| `justify-start`   | `JustifyContent::Start`        |
| `justify-end`     | `JustifyContent::End`          |
| `justify-center`  | `JustifyContent::Center`       |
| `justify-between` | `JustifyContent::SpaceBetween` |
| `justify-around`  | `JustifyContent::SpaceAround`  |
| `justify-evenly`  | `JustifyContent::SpaceEvenly`  |
| `justify-stretch` | `JustifyContent::Stretch`      |

### 11.4 Align Content (多行内容交叉轴对齐)

| 类名                | 效果                           |
|-------------------|------------------------------|
| `content-start`   | `AlignContent::Start`        |
| `content-end`     | `AlignContent::End`          |
| `content-center`  | `AlignContent::Center`       |
| `content-between` | `AlignContent::SpaceBetween` |
| `content-around`  | `AlignContent::SpaceAround`  |
| `content-evenly`  | `AlignContent::SpaceEvenly`  |
| `content-stretch` | `AlignContent::Stretch`      |

### 11.5 Justify Items (Grid 内联轴子项对齐)

> 需要 Feature Flag: `layout-grid`

| 类名                      | 效果                      |
|-------------------------|-------------------------|
| `justify-items-start`   | `JustifyItems::Start`   |
| `justify-items-end`     | `JustifyItems::End`     |
| `justify-items-center`  | `JustifyItems::Center`  |
| `justify-items-stretch` | `JustifyItems::Stretch` |

### 11.6 Justify Self (Grid 单个子项内联轴对齐)

> 需要 Feature Flag: `layout-grid`

| 类名                     | 效果                     |
|------------------------|------------------------|
| `justify-self-start`   | `JustifySelf::Start`   |
| `justify-self-end`     | `JustifySelf::End`     |
| `justify-self-center`  | `JustifySelf::Center`  |
| `justify-self-stretch` | `JustifySelf::Stretch` |

---

## 12. Text Align 文本对齐

> 需要 Feature Flag: `layout-block`

| 类名            | 效果                        |
|---------------|---------------------------|
| `text-left`   | `TextAlign::LegacyLeft`   |
| `text-center` | `TextAlign::LegacyCenter` |
| `text-right`  | `TextAlign::LegacyRight`  |

---

## 13. Aspect Ratio 宽高比

| 类名              | 效果                     |
|-----------------|------------------------|
| `aspect-square` | 1:1 (正方形)              |
| `aspect-video`  | 16:9 (视频比例)            |
| `aspect-[N]`    | 自定义比例, 如 `aspect-[2]`  |
| `aspect-[N/M]`  | 分数比例, 如 `aspect-[4/3]` |

---

## 14. Scrollbar Width 滚动条宽度

| 类名模式          | 效果        |
|---------------|-----------|
| `scrollbar-*` | 设置滚动条预留宽度 |

**支持的值**: 数字 (直接使用), `[Npx]`

**示例**: `scrollbar-0`, `scrollbar-4`, `scrollbar-[10px]`

---

## 15. 状态前缀 (State Prefixes)

布局类支持状态前缀，用于在不同控件状态下应用不同的布局样式。

### 支持的状态前缀

| 前缀          | 对应状态      | 说明      |
|-------------|-----------|---------|
| (无前缀)       | `Normal`  | 默认状态    |
| `hover:`    | `Hover`   | 鼠标悬停状态  |
| `focus:`    | `Focus`   | 获取焦点状态  |
| `active:`   | `Active`  | 激活/按下状态 |
| `disabled:` | `Disable` | 禁用状态    |

### 使用示例

```
class="p-4 hover:p-6 focus:p-8"
```

- 正常状态: padding = 16px
- 悬停状态: padding = 24px
- 焦点状态: padding = 32px

```
class="w-full hover:w-1/2 items-center hover:items-start"
```

- 正常状态: 宽度 100%, 居中对齐
- 悬停状态: 宽度 50%, 顶部对齐

### 状态继承

非 Normal 状态会**继承** Normal 状态的样式，并在其基础上覆盖。例如：

```
class="p-4 pl-2 hover:pt-8"
→ Normal: { left: 8px, right: 16px, top: 16px, bottom: 16px }
→ Hover:  { left: 8px, right: 16px, top: 32px, bottom: 16px }
```

---

## 不支持的语法

以下是 Tailwind CSS 中存在但**本实现不支持**的语法：

- `fixed`, `sticky` (Position)
- `z-*` (Z-Index)
- `border-*` (Border Width，仅厚度，颜色等属于样式而非布局)
- `grid-cols-*`, `grid-rows-*` (Grid Template)
- `auto-cols-*`, `auto-rows-*` (Grid Auto Tracks)
- `order-*` (Flex/Grid Order)
- `place-content-*`, `place-items-*`, `place-self-*`
- 负值 (如 `-m-4`, `-top-2`)
- `space-x-*`, `space-y-*` (Space Between)
- 响应式前缀 (如 `md:`, `lg:`, `sm:`)

---

## 合并行为说明

当同一属性通过多个类名设置时，遵循以下规则：

1. **后声明覆盖先声明**: `p-4 p-8` → 只有 `p-8` 生效，最终 padding = 32px (8×4)
2. **具体覆盖通用的对应部分**: `p-4 pl-2` → left=8px (被 `pl-2` 覆盖), 其余=16px (保留 `p-4`)
3. **独立维度合并**: `w-4 h-8` → width=16px, height=32px (组合为一个 Size)
4. **状态独立累积**: 每个状态 (Normal/Hover/Focus 等) 的类名独立处理，不会相互覆盖

**示例**:

```
class="p-4 pl-2 pt-1"
→ Padding { left: 8px, right: 16px, top: 4px, bottom: 16px }

class="p-4 p-8"
→ Padding { left: 32px, right: 32px, top: 32px, bottom: 32px }  // p-8 完全覆盖 p-4
```


