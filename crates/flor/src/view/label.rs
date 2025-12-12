// use crate::error::Error;
// use crate::signal::effect::updater_effect::create_updater;
// use crate::view::view_id::ViewId;
// use crate::view::View;
// use flor_graphics_base::color::Color;
// use flor_graphics_base::render::RenderBase;
// use graphics::Render;
// use log::{debug, trace};
// use slotmap::Key;
// use std::any::Any;
// use taffy::{AlignSelf, AvailableSpace, Layout, Size, Style};
//
// #[derive(Debug)]
// pub struct Label {
//     view_id: ViewId,
//     title: String,
// }
//
// impl View for Label {
//     fn view_id(&self) -> ViewId {
//         self.view_id
//     }
//
//     fn on_draw(&mut self, render: &mut Render, layout: Layout) -> Result<(), Error> {
//         let x = layout.location.x;
//         let y = layout.location.y;
//         let w = layout.size.width;
//         let h = layout.size.height;
//
//         debug!("label layout : {:?}", layout);
//
//         // 绘制按钮
//         let mut backed_brush =
//             render.create_solid_color_brush(Color::from_hex_str("0CF")?, None)?;
//         if self.view_id.is_activate() {
//             backed_brush = render.create_solid_color_brush(Color::from_hex_str("5EC")?, None)?;
//         }
//         render.fill_quad(x, y, w, h, &backed_brush)?;
//         let text_format = render.create_text_format("宋体")?;
//         let text_brush = render.create_solid_color_brush(Color::from_hex_str("FFF")?, None)?;
//         render.draw_text(&self.title, &text_format, x, y, w, h, &text_brush)?;
//         Ok(())
//     }
//
//     fn update_state(&mut self, state: Box<dyn Any>) {
//         if let Ok(title) = state.downcast::<String>() {
//             self.title = *title;
//         }
//     }
//
//     fn measure(
//         &mut self,
//         _known_dimensions: Size<Option<f32>>,
//         available_space: Size<AvailableSpace>,
//         _style: &Style,
//         render: &Render,
//     ) -> Result<Size<f32>, Error> {
//         fn available_to_f32(space: AvailableSpace) -> f32 {
//             match space {
//                 AvailableSpace::Definite(v) => v,
//                 AvailableSpace::MinContent => 0.0, // 强制折行 → 最小宽度
//                 AvailableSpace::MaxContent => f32::MAX / 2.0, // 不限制 → 全部放一行
//             }
//         }
//         let width = available_to_f32(available_space.width);
//         let height = available_to_f32(available_space.height);
//
//         let text_format = render.create_text_format("宋体")?;
//         let (width, height) =
//             render.measure_text(self.title.as_str(), &text_format, width, height)?;
//         trace!("label({:?}) measure : w:{width},h:{height}", self.view_id());
//         Ok(Size { width, height })
//     }
// }
//
// impl Label {
//     pub fn new<F, R>(title: F) -> Self
//     where
//         F: 'static,
//         F: Fn() -> R,
//         R: AsRef<str>,
//     {
//         let style = Style {
//             size: Size::auto(),
//             align_self: Some(AlignSelf::Start),
//             ..Default::default()
//         };
//
//         let view_id = ViewId::new_with_style(style);
//         trace!("label new view_id: {:?}", view_id.data());
//         let title = create_updater(
//             move || title().as_ref().to_string(),
//             move |v| view_id.update_state(Box::new(v)),
//         );
//
//         Self { view_id, title }
//     }
//
//     pub fn text_color(self) {}
// }
//
// #[inline]
// pub fn label<F, R>(title: F) -> Label
// where
//     F: 'static,
//     F: Fn() -> R,
//     R: AsRef<str>,
// {
//     Label::new(title)
// }
