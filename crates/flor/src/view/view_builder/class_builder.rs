use crate::signal::effect::updater_effect::create_updater_with_id;
use crate::view::View;

// 使用 define_prop! 宏生成 ClassProp trait
crate::define_prop!(clone ClassProp, String, extra: &'static str => |s: &str| s.to_string());

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
        let (effect_id, class_str) = create_updater_with_id(
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
        view_id.pending_effect_id(effect_id);
        view_id.update_class(class_str);
        self
    }
}
