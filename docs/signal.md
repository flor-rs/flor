# Flor Signal 响应式系统文档

Flor 框架内置了一个轻量级的响应式信号系统。该信号系统受 [Floem](https://github.com/lapce/floem) 启发，参考了其 API
设计，但由我们完全独立实现。与 Floem 不同的是，受到 Flor 的设计理念的影响，**Flor Signal 天生支持跨线程使用**
，信号可以在多线程环境下安全地读写和传递。

---

## 核心概念

### 信号 (Signal)

信号是一个响应式的容器，持有一个值。当信号的值发生变化时，所有依赖于它的副作用 (Effect) 会自动重新执行。

| 类型               | 说明              |
|------------------|-----------------|
| `RwSignal<T>`    | 可读写信号，同时支持读取和写入 |
| `ReadSignal<T>`  | 只读信号，只能读取值      |
| `WriteSignal<T>` | 只写信号，只能写入值      |

### 副作用 (Effect)

副作用是订阅了信号的闭包函数。当依赖的信号变化时，副作用会自动重新执行。

| 函数               | 说明                     |
|------------------|------------------------|
| `create_effect`  | 创建一个基础副作用，可接收上一次执行的返回值 |
| `create_updater` | 创建一个更新器副作用，计算值变化后调用回调  |

---

## 基础用法

### 1. 创建信号

```rust
use flor::signal::rw_signal::create_signal;
use flor::signal::read::Read;
use flor::signal::write::Write;

// 创建一个读写信号
let count = create_signal(0);

// 读取值
let value = count.get(); // 0

// 写入值
count.set(10);

// 更新值（函数式）
count.update( | v| * v += 1);
```

### 2. 分离读写权限

如果需要限制读写权限，可以将 `RwSignal` 拆分为 `ReadSignal` 和 `WriteSignal`：

```rust
let signal = create_signal(0);

// 方式1：拆分
let (read, write) = signal.split();

// 方式2：获取只读/只写视图
let read = signal.as_read();
let write = signal.as_write();

// read 只能读取
let value = read.get();

// write 只能写入
write.set(5);
```

### 3. 创建副作用

当信号变化时自动执行回调：

```rust
use flor::signal::effect::effect::create_effect;

let count = create_signal(0);

// 创建副作用：当 count 变化时自动打印
create_effect( move | prev: Option<() > | {
println ! ("count changed to: {}", count.get());
});

// 修改值，会自动触发副作用
count.set(1); // 打印 "count changed to: 1"
count.set(2); // 打印 "count changed to: 2"
```

### 4. 创建更新器 (Updater)

更新器用于计算派生值，当依赖变化时重新计算并调用回调：

```rust
use flor::signal::effect::updater_effect::create_updater;

let count = create_signal(0);

// 创建更新器
let initial = create_updater(
// 计算函数
move | | format!("Value: {}", count.get()),
// 变化回调
| new_value| println!("Updated: {}", new_value),
);

// initial 是初始计算结果: "Value: 0"
count.set(5); // 触发回调，打印 "Updated: Value: 5"
```

---

## 与 UI 控件集成

信号系统与 Flor UI 框架深度集成，通过 `create_updater` 和 `ViewId::update_state` 实现响应式 UI。

### Label 控件示例

```rust
use flor_lys::label::label;
use flor::signal::rw_signal::create_signal;
use flor::signal::read::Read;
use flor::signal::write::Write;

// 创建一个响应式信号
let title = create_signal("Hello".to_string());

// 方式1：直接传递闭包（推荐）
let my_label = label( move | | title.get());

// 当信号变化时，Label 会自动更新
title.set("World".to_string()); // Label 显示 "World"
```

### 内部实现原理

在 `Label::new` 中：

```rust
pub fn new<P: StringProp>(title: P) -> Self {
    let view_id = ViewId::new_with_layout(...);

    // create_updater 建立响应式绑定
    let title = create_updater(
        move || title.make(),                         // 计算函数（读取信号）
        move |v| view_id.update_state(Box::new(v)),  // 变化回调（更新控件）
    );

    Self { view_id, title, style: ... }
}
```

工作流程：

1. `create_updater` 首次执行计算函数，建立依赖追踪
2. 当 `title.get()` 被调用时，当前 effect 订阅了该信号
3. 当 `title.set(...)` 被调用时，触发所有订阅的 effect
4. Effect 重新计算，调用 `on_change` 回调
5. `ViewId::update_state` 将新值传递给控件的 `on_update_state` 方法
6. 控件更新内部状态并请求重绘

---

## 批量更新 (Batch)

多次信号更新可能导致多次 effect 执行。使用 `batch` 可以将多次更新合并为一次：

```rust
use flor::signal::batch::batch;

let a = create_signal(0);
let b = create_signal(0);

// 不使用 batch：每次 set 都会触发 effect
a.set(1); // 触发
b.set(2); // 触发

// 使用 batch：所有 set 完成后只触发一次
batch(| | {
a.set(10);
b.set(20);
// effect 不会在这里触发
});
// batch 结束，effect 只触发一次
```

**注意事项**：

- `batch` 是线程隔离的，只影响当前线程
- 批处理内的信号更新会被去重，相同信号的 effect 只触发一次

---

## 调试支持

在调试模式下，可以为信号添加标签以便追踪：

```rust
use flor::signal::rw_signal::create_signal_with_label;

let count = create_signal_with_label(0, "user_count");

// 或者
let count = create_signal(0);
count.set_label("user_count");
```

启用 `signal-tracing` feature 可在 release 模式下保留标签。

---

## API 参考

### `Read<T>` trait

```rust
trait Read<T> {
    fn id(&self) -> Id;

    /// 追踪订阅（在 effect 内部自动调用）
    fn track(&self);

    /// 获取值（可能失败）
    fn try_get(&self) -> Option<T> where
        T: Clone + 'static;

    /// 获取值（失败时 panic）
    fn get(&self) -> T where
        T: Clone + 'static;

    /// 销毁信号
    fn destroy(&self);
}
```

### `Write<T>` trait

```rust
trait Write<T> {
    fn id(&self) -> Id;

    /// 设置新值
    fn set(&self, new_value: T) where
        T: 'static;

    /// 更新值（函数式）
    fn update(&self, f: impl FnOnce(&mut T)) where
        T: 'static;

    /// 销毁信号
    fn destroy(&self);
}
```

### `RwSignal<T>`

```rust
impl<T> RwSignal<T> {
    /// 拆分为只读和只写信号
    fn split(self) -> (ReadSignal<T>, WriteSignal<T>);

    /// 获取只读视图
    fn as_read(&self) -> ReadSignal<T>;

    /// 获取只写视图
    fn as_write(&self) -> WriteSignal<T>;

    /// 设置调试标签
    fn set_label(&self, label: &str);
}
```

### 创建函数

```rust
/// 创建读写信号
fn create_signal<T: 'static>(value: T) -> RwSignal<T>;

/// 创建并拆分为读写信号
fn create_rw_signal<T: 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>);

/// 创建带标签的信号
fn create_signal_with_label<T: 'static>(value: T, label: &str) -> RwSignal<T>;

/// 创建副作用
fn create_effect<T>(f: impl Fn(Option<T>) -> T + 'static);

/// 创建更新器
fn create_updater<R>(
    compute: impl Fn() -> R + 'static,
    on_change: impl Fn(R) + 'static,
) -> R;

/// 批量更新
fn batch(f: impl Fn());
```

---

## 运行时机制

### 依赖追踪

1. 全局运行时 `RUNTIME` 存储所有信号值和副作用
2. 线程局部的 `SCOPE` 记录当前正在执行的 effect ID
3. 当 `signal.get()` 被调用时：
    - 调用 `track()` 将当前 effect 订阅到该信号
    - 建立 `Signal ID -> Effect ID` 的映射关系
4. 当 `signal.set()` 被调用时：
    - 遍历所有订阅该信号的 effect
    - 将它们放入更新队列
    - 消息循环执行队列中的 effect

### Effect 执行流程

```
set(value) 
    → RUNTIME.run_signal_effect(signal_id)
    → 添加到 update_queue
    → 消息循环调用 execute_update_queue()
    → run_effects_for_signal(signal_id)
    → effect.run_effect()
```

### 与消息循环集成

Effect 的实际执行发生在 Flor 的消息循环中：

```rust
// 在 lib.rs 的消息循环中
RUNTIME.execute_update_queue();
```

这确保了 UI 更新在正确的时机发生，避免了竞态条件。

---

## 最佳实践

1. **使用闭包传递响应式值**
   ```rust
   // ✅ 推荐
   label(move || title.get())
   
   // ❌ 不推荐（只读取一次，不会响应变化）
   label(title.get())
   ```

2. **拆分读写权限**
   ```rust
   // 只暴露需要的权限
   let (read, write) = create_signal(0).split();
   give_to_child(read);  // 子组件只能读
   store_write(write);   // 父组件保留写权限
   ```

3. **使用 batch 优化多次更新**
   ```rust
   batch(|| {
       state1.set(a);
       state2.set(b);
       state3.set(c);
   });
   ```

4. **避免在 effect 中写入依赖的信号**
   ```rust
   // ❌ 可能导致无限循环
   create_effect(|_| {
       let v = count.get();
       count.set(v + 1);
   });
   ```

5. **及时销毁不需要的信号**
   ```rust
   signal.destroy(); // 释放资源
   ```

---

## 与其他框架对比

| 特性      | Flor Signal | SolidJS  | Leptos |
|---------|-------------|----------|--------|
| 细粒度更新   | ✅           | ✅        | ✅      |
| 自动依赖追踪  | ✅           | ✅        | ✅      |
| Copy 信号 | ✅           | -        | ✅      |
| 批量更新    | ✅           | ✅        | ✅      |
| 线程安全    | ✅           | N/A      | ✅      |
| 调试标签    | ✅           | DevTools | ✅      |

Flor Signal 的设计简洁高效，特别适合原生 GUI 框架的响应式需求。
