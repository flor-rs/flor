use crate::signal::create_updater_with_id;
use crate::view::ViewIdentity;
use flor_base::types::Transform2D;

crate::define_prop!(clone TransformProp, Transform2D);

/// Transform Builder trait
///
/// 提供声明式的变换设置，支持固定值或响应式信号驱动。
pub trait TransformBuilder {
    /// 设置控件的变换
    ///
    /// # 参数
    /// - `transform`: 返回 Transform2D 的闭包，支持信号驱动
    ///
    /// # 示例
    /// ```rust
    /// // 固定旋转
    /// div().transform(Transform2D::rotate_at_degrees(20.0, 50.0, 50.0))
    ///
    /// // 信号驱动动画
    /// let rotation = RwSignal::new(0.0f32);
    /// div().transform(Transform2D::rotate_at_degrees(rotation.get(), 50.0, 50.0))
    /// ```
    fn transform(self, transform: impl TransformProp) -> Self;
}

impl<V: ViewIdentity> TransformBuilder for V {
    fn transform(self, transform: impl TransformProp) -> Self {
        let view_id = self.identity();
        let (effect_id, init_transform) =
            create_updater_with_id(move || transform.make(), move |v| view_id.set_transform(v));
        view_id.pending_effect_id(effect_id);
        view_id.set_transform(init_transform);
        self
    }
}
