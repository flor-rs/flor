//! 用于将 `flor_base::graphics::Path` 转为可通过 OpenGL 绘制的顶点数组。
use flor_base::graphics::PathCommand;
use lyon_tessellation::{
    math::Point, path::Path as LyonPath, BuffersBuilder, FillOptions, FillTessellator,
    StrokeOptions, StrokeTessellator, VertexBuffers,
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TexVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

pub struct Tessellator {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
}

impl Tessellator {
    pub fn new() -> Self {
        Self {
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
        }
    }

    pub fn build_lyon_path(flor_path: &flor_base::graphics::Path) -> LyonPath {
        let mut builder = LyonPath::builder().with_svg();
        for cmd in flor_path.commands() {
            match cmd {
                PathCommand::MoveTo(x, y) => {
                    builder.move_to(Point::new(*x, *y));
                }
                PathCommand::LineTo(x, y) => {
                    builder.line_to(Point::new(*x, *y));
                }
                PathCommand::Bezier(pts) => match pts.len() {
                    2 => {
                        builder.quadratic_bezier_to(
                            Point::new(pts[0].0, pts[0].1),
                            Point::new(pts[1].0, pts[1].1),
                        );
                    }
                    3 => {
                        builder.cubic_bezier_to(
                            Point::new(pts[0].0, pts[0].1),
                            Point::new(pts[1].0, pts[1].1),
                            Point::new(pts[2].0, pts[2].1),
                        );
                    }
                    _ => {
                        if let Some(last) = pts.last() {
                            builder.line_to(Point::new(last.0, last.1));
                        }
                    }
                },
                PathCommand::Close => builder.close(),
            }
        }
        builder.build()
    }

    pub fn tessellate_fill(
        &mut self,
        path: &flor_base::graphics::Path,
        dpi_scale_x: f32,
        dpi_scale_y: f32,
    ) -> Result<VertexBuffers<Vertex, u32>, String> {
        let mut geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let lyon_path = Self::build_lyon_path(path);

        // 取 x 和 y 缩放的较大值来作为容差基准，确保在任何方向上都不会出现锯齿
        let max_scale = dpi_scale_x.max(dpi_scale_y).max(1.0);

        self.fill_tess
            .tessellate_path(
                &lyon_path,
                &FillOptions::default().with_tolerance(0.01 / max_scale),
                &mut BuffersBuilder::new(&mut geometry, |vertex: lyon_tessellation::FillVertex| {
                    Vertex {
                        position: vertex.position().to_array(),
                    }
                }),
            )
            .map_err(|e| format!("Fill Tessellation Error: {:?}", e))?;

        Ok(geometry)
    }

    pub fn tessellate_stroke(
        &mut self,
        path: &flor_base::graphics::Path,
        stroke_width: f32,
        dpi_scale_x: f32,
        dpi_scale_y: f32,
    ) -> Result<VertexBuffers<Vertex, u32>, String> {
        let mut geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();
        let lyon_path = Self::build_lyon_path(path);

        // 取 x 和 y 缩放的较大值来作为容差基准
        let max_scale = dpi_scale_x.max(dpi_scale_y).max(1.0);

        // Setup stroke options, handling line width and curve smoothness
        let options = StrokeOptions::default()
            .with_line_width(stroke_width)
            .with_tolerance(0.01 / max_scale);

        self.stroke_tess
            .tessellate_path(
                &lyon_path,
                &options,
                &mut BuffersBuilder::new(
                    &mut geometry,
                    |vertex: lyon_tessellation::StrokeVertex| Vertex {
                        position: vertex.position().to_array(),
                    },
                ),
            )
            .map_err(|e| format!("Stroke Tessellation Error: {:?}", e))?;

        Ok(geometry)
    }
}
