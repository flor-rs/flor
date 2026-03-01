use flor_base::types::Transform2D;
use glow::HasContext;

pub mod builtin;
pub mod tessellator;

pub struct ShaderProgram {
    pub id: glow::Program,
}

impl ShaderProgram {
    pub fn new(gl: &glow::Context, vertex_src: &str, fragment_src: &str) -> Result<Self, String> {
        unsafe {
            let program = gl.create_program()?;

            let vertex_shader = Self::compile_shader(gl, glow::VERTEX_SHADER, vertex_src)?;
            let fragment_shader = Self::compile_shader(gl, glow::FRAGMENT_SHADER, fragment_src)?;

            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }

            // After linking, shaders can be detached and deleted
            gl.detach_shader(program, vertex_shader);
            gl.detach_shader(program, fragment_shader);
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            Ok(Self { id: program })
        }
    }

    unsafe fn compile_shader(
        gl: &glow::Context,
        shader_type: u32,
        source: &str,
    ) -> Result<glow::Shader, String> {
        let shader = gl.create_shader(shader_type)?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            Err(gl.get_shader_info_log(shader))
        } else {
            Ok(shader)
        }
    }

    pub fn use_program(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.id));
        }
    }

    /// 解绑当前程序与相关状态，恢复默认渲染管线状态
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            // 解绑纹理
            gl.bind_texture(glow::TEXTURE_2D, None);
            // 解绑 Shader Program
            gl.use_program(None);
        }
    }

    pub fn bind_transform(&self, gl: &glow::Context, transform: Transform2D) {
        let mat3 = [
            transform.m11,
            transform.m12,
            0.0,
            transform.m21,
            transform.m22,
            0.0,
            transform.dx,
            transform.dy,
            1.0,
        ];
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_transform") {
                gl.uniform_matrix_3_f32_slice(Some(&loc), false, &mat3);
            }
        }
    }

    pub fn bind_texture(&self, gl: &glow::Context, texture_unit_index: i32) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_texture") {
                gl.uniform_1_i32(Some(&loc), texture_unit_index);
            }
        }
    }

    pub fn bind_opacity(&self, gl: &glow::Context, opacity: Option<f32>) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_opacity") {
                gl.uniform_1_f32(Some(&loc), opacity.unwrap_or(1.0));
            }
        }
    }

    pub fn bind_text_color(&self, gl: &glow::Context, color: [f32; 4]) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_textColor") {
                gl.uniform_4_f32(Some(&loc), color[0], color[1], color[2], color[3]);
            }
        }
    }

    pub fn bind_resolution(&self, gl: &glow::Context, w: f32, h: f32) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_resolution") {
                gl.uniform_2_f32(Some(&loc), w, h);
            }
        }
    }

    pub fn bind_blur_radius(&self, gl: &glow::Context, radius: f32) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_blurRadius") {
                gl.uniform_1_f32(Some(&loc), radius);
            }
        }
    }

    pub fn bind_direction(&self, gl: &glow::Context, x: f32, y: f32) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, "u_direction") {
                gl.uniform_2_f32(Some(&loc), x, y);
            }
        }
    }

    pub fn delete(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.id);
        }
    }
}
