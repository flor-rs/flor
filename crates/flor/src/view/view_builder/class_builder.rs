use crate::signal::effect::updater_effect::create_updater;
use crate::view::View;

/// 定义统一的属性行为接口
pub trait ClassProp: 'static {
    fn make(&self) -> String;
}

// 1. 实现文本类型：String
// 在响应式系统中，它充当一个返回固定值的“常量闭包”
impl ClassProp for String {
    fn make(&self) -> String {
        self.clone()
    }
}

// 2. 实现静态字符串切片：&'static str
impl ClassProp for &'static str {
    fn make(&self) -> String {
        self.to_string()
    }
}

// 3. 实现闭包/函数：Fn() -> String
// 这是真正的动态计算，会触发响应式依赖收集
impl<F> ClassProp for F
where
    F: Fn() -> String + 'static,
{
    fn make(&self) -> String {
        (self)()
    }
}

// 定义 Builder Trait
// T 根据你的上下文可能是 PhantomData 或其他标识，此处保留泛型位
pub trait ClassBuilder<M> {
    fn class(self, class_str: M) -> Self;
}

// 实现 Builder
impl<V, M> ClassBuilder<M> for V
where
    V: View,
    M: ClassProp, // 约束 M 必须实现了上述 Trait
{
    fn class(self, class_str: M) -> V {
        let view_id = self.view_id();

        // create_updater 的第一个参数通常是一个 Fn，会多次执行以重新计算
        // class_str 被 move 进这个闭包，成为闭包环境的一部分
        let class_str = create_updater(
            move || {
                // 无论是 String 还是 Fn，统一调用 make()
                // 如果是 String，这里就相当于取环境里的变量
                // 如果是 Fn，这里就是执行计算
                class_str.make()
            },
            move |class_str| {
                view_id.update_class(class_str);
            },
        );
        view_id.update_class(class_str);
        self
    }
}
