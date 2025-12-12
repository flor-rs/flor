use crate::{
    FontStretch, FontStyle, FontWeight, ParagraphAlignment, TextAlignment, TextTrimming,
    WordWrapping,
};

#[macro_export]
macro_rules! define_handle_traits {
    ( $($name:ident),* $(,)? ) => {
        $(
            pub trait $name {}
        )*
    };
}

define_handle_traits!(SurfaceId, SvgHandle, BrushHandle);

pub trait ImageHandle {
    fn frame_count(&self) -> usize;

    fn delays(&self) -> &[u16];

    fn total_delays(&self) -> u128;

    fn get_size(&self) -> (u32, u32);

    fn get_width(&self) -> u32;

    fn get_height(&self) -> u32;
}

pub trait TextFormatHandle {
    // =========================================================
    // 字体属性 (Font Properties)
    // =========================================================

    /// 设置字体大小 (单位：逻辑像素/Points)
    fn set_font_size(&mut self, size: f32) -> &mut Self;
    /// 获取字体大小
    fn font_size(&self) -> f32;

    /// 设置字重 (粗细)
    fn set_font_weight(&mut self, weight: FontWeight) -> &mut Self;
    /// 获取字重
    fn font_weight(&self) -> FontWeight;

    /// 设置字体风格 (斜体等)
    fn set_font_style(&mut self, style: FontStyle) -> &mut Self;
    /// 获取字体风格
    fn font_style(&self) -> FontStyle;

    /// 设置字体宽窄
    fn set_font_stretch(&mut self, stretch: FontStretch) -> &mut Self;
    /// 获取字体宽窄
    fn font_stretch(&self) -> FontStretch;

    /// 获取字体家族名称 (如 "Microsoft YaHei")
    /// 注意：通常只读，或者在创建时指定。
    /// 返回 String 是为了避免多线程锁(Mutex)导致的引用生命周期问题。
    fn font_family_name(&self) -> String;

    // =========================================================
    // 段落排版 (Paragraph & Layout)
    // =========================================================

    /// 设置水平对齐方式
    fn set_text_alignment(&mut self, align: TextAlignment) -> &mut Self;
    /// 获取水平对齐方式
    fn text_alignment(&self) -> TextAlignment;

    /// 设置垂直对齐方式
    fn set_paragraph_alignment(&mut self, align: ParagraphAlignment) -> &mut Self;
    /// 获取垂直对齐方式
    fn paragraph_alignment(&self) -> ParagraphAlignment;

    /// 设置换行模式
    fn set_word_wrapping(&mut self, wrapping: WordWrapping) -> &mut Self;
    /// 获取换行模式
    fn word_wrapping(&self) -> WordWrapping;

    /// 设置行高倍数 (1.0 为默认)
    fn set_line_height(&mut self, line_height_factor: f32) -> &mut Self;
    /// 获取行高倍数
    fn line_height(&self) -> f32;

    // =========================================================
    // 溢出处理 (Overflow)
    // =========================================================

    /// 设置文本溢出时的截断方式
    fn set_text_trimming(&mut self, trimming: TextTrimming) -> &mut Self;
    /// 获取文本溢出时的截断方式
    fn text_trimming(&self) -> TextTrimming;
    /// 藏检测标识
    fn dirty(&self) -> bool;

    /// 重建
    fn clear_dirty(&mut self);
}
