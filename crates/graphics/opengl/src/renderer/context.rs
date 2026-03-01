use crate::error::GlError;
use glow::{Context, HasContext, NativeTexture, PixelUnpackData};
use std::ops::{Deref, DerefMut};

pub struct GlContext {
    gl_context: Context,
}

impl GlContext {
    pub fn from_context(gl_context: Context) -> GlContext {
        Self { gl_context }
    }

    /// width and height is physical unit
    pub fn create_texture_tex_image_2d(
        &self,
        width: i32,
        height: i32,
        pixel_unpack_data: PixelUnpackData,
    ) -> Result<NativeTexture, GlError> {
        unsafe {
            let tex = self.gl_context.create_texture()?;
            self.bind_texture(glow::TEXTURE_2D, Some(tex));
            self.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                pixel_unpack_data,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.bind_texture(glow::TEXTURE_2D, None);
            Ok(tex)
        }
    }

    pub fn create_texture(&self) -> Result<NativeTexture, GlError> {
        unsafe {
            let tex = self.gl_context.create_texture()?;
            self.bind_texture(glow::TEXTURE_2D, Some(tex));
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            self.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            Ok(tex)
        }
    }
}

impl Deref for GlContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.gl_context
    }
}

impl DerefMut for GlContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gl_context
    }
}
