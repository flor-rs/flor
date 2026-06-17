use crate::signal::create_updater_with_id;
use crate::view::ViewIdentity;

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
    V: ViewIdentity,
    M: ClassProp, // 约束 M 必须实现了上述 Trait
{
    fn class(self, class_str: M) -> V {
        let view_id = self.identity();

        let layer_id = view_id.new_layout_resolver_layer();

        let (effect_id, _class_str) = create_updater_with_id(
            move || class_str.make(),
            move |class_str| {
                view_id.update_class(layer_id, class_str.clone());
            },
        );
        view_id.pending_effect_id(effect_id);
        self
    }
}
