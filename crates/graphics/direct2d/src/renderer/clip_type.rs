use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows::Win32::Graphics::Direct2D::{ID2D1Layer, D2D1_LAYER_PARAMETERS1};

/// 剪裁类型，保存完整数据以支持 suspend/resume
pub enum ClipType {
    /// 普通矩形剪裁（硬件加速）
    AxisAligned { rect: D2D_RECT_F },
    /// 复杂形状剪裁（使用 Layer）
    Layer {
        layer: ID2D1Layer,
        params: D2D1_LAYER_PARAMETERS1,
    },
}
