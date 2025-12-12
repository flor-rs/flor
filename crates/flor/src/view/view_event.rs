// use crate::view::view_id::ViewId;
// use crate::view::View;
// use taffy::Style;
//
// pub trait ViewEvent: Sized + IntoView<V = Self::DV> + View {
//     /// 装饰后的视图类型
//     type DV: View;
//
//     /// 设置样式
//     fn style(self, style_fn: fn(Style) -> Style) -> Self::DV {
//         let mut view = self.into_view();
//
//         // 应用样式...
//         view
//     }
//
//     /// 点击事件
//     fn on_click(self, f: impl Fn() + Send + Sync + 'static) -> Self::DV {
//         let view = self.into_view();
//         let view_id = view.view_id();
//
//         // 使用 RefCell 获取可变引用
//
//         // if let Some(state) = VIEW_STORAGE.lock().get_state_mut(view_id) {
//         //     state.click_handler = Some(Arc::new(f));
//         // }
//
//         view
//     }
// }
//
// pub trait IntoView: Sized {
//     type V: View + 'static;
//     fn into_view(self) -> Self::V;
// }
//
// impl<VW: View, IV: IntoView<V = VW> + View> ViewEvent for IV {
//     type DV = VW;
// }
//
// impl<VW: View + 'static> IntoView for VW {
//     type V = VW;
//     fn into_view(self) -> Self::V {
//         self
//     }
// }
//
// impl View for Box<dyn View> {
//     fn view_id(&self) -> ViewId {
//         self.as_ref().view_id()
//     }
//
//     // 委托所有 View 方法给内部的 dyn View
//     // fn draw(&mut self, render: &mut Render) -> Result<(), Error> {
//     //     (**self).draw(render)
//     // }
//
//     // 其他方法同理
// }
