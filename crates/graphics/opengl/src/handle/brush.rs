use flor_base::graphics::{BrushHandle, Gradient};
use flor_base::types::Color;

#[derive(Debug, Clone)]
pub struct GlGradientData {
    pub gradient_type: i32, // 0: Solid, 1: Linear, 2: Radial
    pub stop_count: i32,
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub stops: [f32; 32],
    pub colors: [f32; 128], // 32 * 4 (RGBA)
    // 超过32个断点时，使用浮点纹理数据: 依次为 [stop, r, g, b, a] 连续排列
    pub overflow_texture_data: Vec<f32>,
}

impl GlGradientData {
    /// 超过32个断点时，由CPU插值生成一维颜色纹理条的数据
    pub fn get_texture_pixels(&self, num_pixels: usize) -> Vec<f32> {
        if self.overflow_texture_data.is_empty() {
            return Vec::new();
        }

        let stops = self.stop_count as usize;
        if stops == 0 {
            return Vec::new();
        }
        let mut pixels = vec![0.0f32; num_pixels * 4];
        let tex_data = &self.overflow_texture_data;
        for i in 0..num_pixels {
            let t = i as f32 / (num_pixels - 1) as f32;
            let mut color = [1.0, 1.0, 1.0, 1.0];
            if t <= tex_data[0] {
                color = [tex_data[1], tex_data[2], tex_data[3], tex_data[4]];
            } else if t >= tex_data[(stops - 1) * 5] {
                let base = (stops - 1) * 5;
                color = [
                    tex_data[base + 1],
                    tex_data[base + 2],
                    tex_data[base + 3],
                    tex_data[base + 4],
                ];
            } else {
                for j in 0..stops - 1 {
                    let t1 = tex_data[j * 5];
                    let t2 = tex_data[(j + 1) * 5];
                    if t >= t1 && t <= t2 {
                        let dt = t2 - t1;
                        let f = if dt > 0.0 { (t - t1) / dt } else { 0.0 };
                        for k in 0..4 {
                            color[k] = tex_data[j * 5 + 1 + k] * (1.0 - f)
                                + tex_data[(j + 1) * 5 + 1 + k] * f;
                        }
                        break;
                    }
                }
            }
            pixels[i * 4..i * 4 + 4].copy_from_slice(&color);
        }

        pixels
    }
}

#[derive(Debug, Clone)]
pub enum GlBrushHandle {
    Solid(Color),
    Gradient { gradient: Gradient },
}

impl GlBrushHandle {
    pub(crate) fn to_shader_data(&self) -> GlGradientData {
        let mut data = GlGradientData {
            gradient_type: 0,
            stop_count: 1,
            start: [0.0, 0.0],
            end: [1.0, 0.0],
            stops: [0.0; 32],
            colors: [0.0; 128],
            overflow_texture_data: Vec::new(),
        };

        let fill_data = |data: &mut GlGradientData, colors: &Vec<(f32, Color)>| {
            if colors.len() <= 32 {
                for (i, (stop, color)) in colors.iter().enumerate() {
                    data.stops[i] = *stop;
                    data.colors[i * 4] = color.r as f32 / 255.0;
                    data.colors[i * 4 + 1] = color.g as f32 / 255.0;
                    data.colors[i * 4 + 2] = color.b as f32 / 255.0;
                    data.colors[i * 4 + 3] = color.a as f32 / 255.0;
                }
            } else {
                let mut tex_data = Vec::with_capacity(colors.len() * 5);
                for (stop, color) in colors {
                    tex_data.push(*stop);
                    tex_data.push(color.r as f32 / 255.0);
                    tex_data.push(color.g as f32 / 255.0);
                    tex_data.push(color.b as f32 / 255.0);
                    tex_data.push(color.a as f32 / 255.0);
                }
                data.overflow_texture_data = tex_data;
            }
        };

        match self {
            GlBrushHandle::Solid(c) => {
                data.gradient_type = 0; // 纯色
                data.stop_count = 1;
                data.colors[0] = c.r as f32 / 255.0;
                data.colors[1] = c.g as f32 / 255.0;
                data.colors[2] = c.b as f32 / 255.0;
                data.colors[3] = c.a as f32 / 255.0;
            }
            GlBrushHandle::Gradient {
                gradient: Gradient::Linear { start, end, colors },
            } => {
                data.gradient_type = 1; // 线性
                data.start = [start.0, start.1];
                data.end = [end.0, end.1];
                data.stop_count = colors.len() as i32;

                fill_data(&mut data, colors);
            }
            GlBrushHandle::Gradient {
                gradient:
                    Gradient::Radial {
                        center,
                        radius,
                        colors,
                    },
            } => {
                data.gradient_type = 2; // 径向
                data.start = [center.0, center.1];
                data.end = [*radius, 0.0];
                data.stop_count = colors.len() as i32;

                fill_data(&mut data, colors);
            }
        }
        data
    }
}

impl BrushHandle for GlBrushHandle {}
