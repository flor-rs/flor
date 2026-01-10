use crate::error::Error;
use crate::render::FlorRender;
use crate::view::view_id::ViewId;
use crate::view::view_storage::VIEW_STORAGE;
use flor_graphics_base::{RenderContext, Transform2D};
use log::trace;
use taffy::{Display, Layout};

enum DrawStage {
    Enter,
    Exit {
        transform_depth: Option<u32>,
        clip_depth: u32,
        layout: Layout,
    },
}

struct DrawFrame {
    view_id: ViewId,
    abs_location: (f32, f32),
    stage: DrawStage,
}

pub fn draw_entry(root_id: ViewId, render: &mut FlorRender) -> Result<(), Error> {
    let views = VIEW_STORAGE.views.read();
    let child_map = VIEW_STORAGE.child_ids.read();

    let mut stack = Vec::with_capacity(64);

    stack.push(DrawFrame {
        view_id: root_id,
        abs_location: (0.0, 0.0),
        stage: DrawStage::Enter,
    });

    while let Some(frame) = stack.pop() {
        match frame.stage {
            DrawStage::Enter => {
                let view_id = frame.view_id;

                let Some(view) = views.get(view_id) else {
                    continue;
                };

                if !view_id.visual() {
                    continue;
                }

                let layout = view_id.layout()?;
                if view_id.calc_current_style()?.display == Display::None {
                    continue;
                }

                let abs_location = (
                    frame.abs_location.0 + layout.location.x,
                    frame.abs_location.1 + layout.location.y,
                );

                trace!("view({}).draw_entry", view_id);

                let mut transform_depth = None;
                let clip_depth = render.get_clip_depth()?;

                view.write().on_draw(render, abs_location, layout)?;

                let content_rect = (
                    abs_location.0,
                    abs_location.1,
                    layout.size.width,
                    layout.size.height,
                );
                render.push_clip(content_rect)?;

                if let Some((scroll_x, scroll_y)) = view_id.scroll_offset() {
                    if scroll_x != 0.0 || scroll_y != 0.0 {
                        transform_depth = Some(render.get_transform_depth()?);
                        render.push_transform(&Transform2D::translation(-scroll_x, -scroll_y))?;
                    }
                }

                stack.push(DrawFrame {
                    view_id,
                    abs_location,
                    stage: DrawStage::Exit {
                        transform_depth,
                        clip_depth,
                        layout,
                    },
                });

                if let Some(children) = child_map.get(view_id) {
                    for &child_id in children.iter().rev() {
                        stack.push(DrawFrame {
                            view_id: child_id,
                            abs_location,
                            stage: DrawStage::Enter,
                        });
                    }
                }
            }

            DrawStage::Exit {
                transform_depth,
                clip_depth,
                layout,
            } => {
                let Some(view) = views.get(frame.view_id) else {
                    continue;
                };

                if transform_depth.is_some() {
                    render.pop_transform(transform_depth)?;
                }
                render.pop_clip(Some(clip_depth))?;

                view.write()
                    .on_draw_overlay(render, frame.abs_location, layout)?;
            }
        }
    }
    Ok(())
}
