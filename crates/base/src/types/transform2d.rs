use std::ops::Mul;

/// 2D 仿射变换矩阵
/// 采用 "Row-Vector" (行向量) 逻辑，即: Vector * Matrix
/// 内存布局对应 Direct2D / Windows.Foundation.Numerics.Matrix3x2
///
/// [ m11 m12 0 ]
/// [ m21 m22 0 ]
/// [ dx  dy  1 ]
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Transform2D {
    /// X 轴缩放 (Scale X) / 旋转余弦 (Cos)
    pub m11: f32,
    /// Y 轴倾斜 (Skew Y) / 旋转正弦 (Sin)
    pub m12: f32,
    /// X 轴倾斜 (Skew X) / 旋转负正弦 (-Sin)
    pub m21: f32,
    /// Y 轴缩放 (Scale Y) / 旋转余弦 (Cos)
    pub m22: f32,
    /// X 轴平移
    pub dx: f32,
    /// Y 轴平移
    pub dy: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl PartialEq for Transform2D {
    fn eq(&self, other: &Self) -> bool {
        // 使用宽松的浮点数比较，防止精度误差导致动画抖动
        let eps = 1e-5;
        (self.m11 - other.m11).abs() < eps
            && (self.m12 - other.m12).abs() < eps
            && (self.m21 - other.m21).abs() < eps
            && (self.m22 - other.m22).abs() < eps
            && (self.dx - other.dx).abs() < eps
            && (self.dy - other.dy).abs() < eps
    }
}

impl Transform2D {
    /// 单位矩阵
    pub const IDENTITY: Transform2D = Transform2D {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        dx: 0.0,
        dy: 0.0,
    };

    /// 自定义构建 (对应 CSS matrix)
    pub fn new(m11: f32, m12: f32, m21: f32, m22: f32, dx: f32, dy: f32) -> Self {
        Transform2D {
            m11,
            m12,
            m21,
            m22,
            dx,
            dy,
        }
    }

    // =========================================================
    // 基础变换构造 (Constructors)
    // =========================================================

    /// 平移
    pub fn translation(x: f32, y: f32) -> Self {
        Transform2D {
            dx: x,
            dy: y,
            ..Self::IDENTITY
        }
    }

    /// 缩放
    pub fn scale(sx: f32, sy: f32) -> Self {
        Transform2D {
            m11: sx,
            m22: sy,
            ..Self::IDENTITY
        }
    }

    /// 旋转 (弧度)
    /// 正值 = 顺时针旋转 (在 Y 轴向下的屏幕坐标系中)
    pub fn rotation(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Transform2D {
            m11: cos,
            m12: sin,
            m21: -sin,
            m22: cos,
            ..Self::IDENTITY
        }
    }

    /// 旋转 (角度)
    pub fn rotation_degrees(degrees: f32) -> Self {
        Self::rotation(degrees.to_radians())
    }

    /// 倾斜 (弧度) - 对应 CSS skew(ax, ay)
    pub fn skew(skew_x_rad: f32, skew_y_rad: f32) -> Self {
        Transform2D {
            m21: skew_x_rad.tan(), // m21 控制 X 如何随 Y 改变 (Skew X)
            m12: skew_y_rad.tan(), // m12 控制 Y 如何随 X 改变 (Skew Y)
            ..Self::IDENTITY
        }
    }

    /// 倾斜 (角度)
    pub fn skew_degrees(skew_x_deg: f32, skew_y_deg: f32) -> Self {
        Self::skew(skew_x_deg.to_radians(), skew_y_deg.to_radians())
    }

    // =========================================================
    // 链式操作 (Chaining)
    // 逻辑：CurrentMatrix * NewMatrix
    // 效果：先应用当前的变换，再应用新的变换
    // =========================================================

    pub fn then_translate(self, x: f32, y: f32) -> Self {
        self * Self::translation(x, y)
    }

    pub fn then_scale(self, sx: f32, sy: f32) -> Self {
        self * Self::scale(sx, sy)
    }

    pub fn then_rotate(self, radians: f32) -> Self {
        self * Self::rotation(radians)
    }

    pub fn then_rotate_degrees(self, degrees: f32) -> Self {
        self * Self::rotation_degrees(degrees)
    }

    pub fn then_skew(self, kx_rad: f32, ky_rad: f32) -> Self {
        self * Self::skew(kx_rad, ky_rad)
    }

    // =========================================================
    // 复合辅助函数 (Complex Helpers)
    // =========================================================

    /// 绕指定点 (cx, cy) 旋转 (对应 CSS transform-origin)
    pub fn rotate_at(radians: f32, cx: f32, cy: f32) -> Self {
        // 修正后的正确逻辑：
        // 1. 移回原点 (-cx, -cy)
        // 2. 旋转
        // 3. 移回原位 (cx, cy)
        Self::translation(-cx, -cy)
            .then_rotate(radians)
            .then_translate(cx, cy)
    }

    /// 绕指定点 (cx, cy) 旋转 (角度版)
    pub fn rotate_at_degrees(degrees: f32, cx: f32, cy: f32) -> Self {
        Self::rotate_at(degrees.to_radians(), cx, cy)
    }

    /// 绕指定点缩放
    pub fn scale_at(sx: f32, sy: f32, cx: f32, cy: f32) -> Self {
        Self::translation(-cx, -cy)
            .then_scale(sx, sy)
            .then_translate(cx, cy)
    }

    // =========================================================
    // 核心运算 (Core Math)
    // =========================================================

    /// 矩阵乘法
    /// 遵循 Row-Vector 乘法: Result = Self * Other
    /// 几何意义：先进行 Self 变换，再进行 Other 变换
    pub fn multiply(&self, other: &Self) -> Self {
        Transform2D {
            m11: self.m11 * other.m11 + self.m12 * other.m21,
            m12: self.m11 * other.m12 + self.m12 * other.m22,
            m21: self.m21 * other.m11 + self.m22 * other.m21,
            m22: self.m21 * other.m12 + self.m22 * other.m22,
            dx: self.dx * other.m11 + self.dy * other.m21 + other.dx,
            dy: self.dx * other.m12 + self.dy * other.m22 + other.dy,
        }
    }

    /// 计算行列式 (用于判断是否可逆)
    pub fn determinant(&self) -> f32 {
        self.m11 * self.m22 - self.m12 * self.m21
    }

    /// 求逆矩阵
    /// 用于将 屏幕坐标 转换为 控件内部局部坐标 (Hit Testing)
    pub fn invert(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-6 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Transform2D {
            m11: self.m22 * inv_det,
            m12: -self.m12 * inv_det,
            m21: -self.m21 * inv_det,
            m22: self.m11 * inv_det,
            // 逆矩阵的平移部分推导较为复杂，这是标准公式
            dx: (self.m21 * self.dy - self.m22 * self.dx) * inv_det,
            dy: (self.m12 * self.dx - self.m11 * self.dy) * inv_det,
        })
    }

    /// 判断是否是单位矩阵（无变换）
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// 判断变换是否保持轴对齐（即没有旋转或倾斜）
    /// 如果 m12 和 m21 均为 0，则变换后的矩形仍然是轴对齐的矩形。
    pub fn is_axis_aligned(&self) -> bool {
        let eps = 1e-5;
        self.m12.abs() < eps && self.m21.abs() < eps
    }

    // =========================================================
    // 应用变换 (Apply)
    // =========================================================

    /// 变换点 (受平移影响)
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x * self.m11 + y * self.m21 + self.dx,
            x * self.m12 + y * self.m22 + self.dy,
        )
    }

    /// 变换向量 (忽略平移，只受缩放/旋转影响)
    pub fn transform_vector(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.m11 + y * self.m21, x * self.m12 + y * self.m22)
    }

    /// 变换矩形 (AABB)
    /// 计算变换后的矩形所占用的最大轴对齐包围盒
    /// 对于 UI 脏矩形计算非常重要
    pub fn transform_rect(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
        // 计算矩形四个角的变换后坐标
        let (x1, y1) = self.transform_point(x, y); // Top-Left
        let (x2, y2) = self.transform_point(x + w, y); // Top-Right
        let (x3, y3) = self.transform_point(x, y + h); // Bottom-Left
        let (x4, y4) = self.transform_point(x + w, y + h); // Bottom-Right

        // 找出最小和最大值
        let min_x = x1.min(x2).min(x3).min(x4);
        let min_y = y1.min(y2).min(y3).min(y4);
        let max_x = x1.max(x2).max(x3).max(x4);
        let max_y = y1.max(y2).max(y3).max(y4);

        (min_x, min_y, max_x - min_x, max_y - min_y) // Returns (x, y, w, h)
    }

    /// 逆变换点（将全局/屏幕坐标转换回局部坐标）
    ///
    /// 用于命中测试：将鼠标点击的屏幕坐标转换为控件内部坐标
    ///
    /// # 返回值
    /// - `Some((local_x, local_y))` - 成功转换后的局部坐标
    /// - `None` - 矩阵不可逆（行列式为 0）
    ///
    /// # 示例
    /// ```ignore
    /// let transform = Transform2D::translation(100.0, 50.0).then_scale(2.0, 2.0);
    /// // 屏幕坐标 (120, 70) 对应的局部坐标
    /// let local = transform.inverse_transform_point(120.0, 70.0);
    /// // local = Some((10.0, 10.0))
    /// ```
    #[inline]
    pub fn inverse_transform_point(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let det = self.determinant();
        if det.abs() < 1e-6 {
            return None;
        }
        let inv_det = 1.0 / det;

        // 先减去平移，再应用逆线性变换
        let x_no_translate = x - self.dx;
        let y_no_translate = y - self.dy;

        Some((
            (x_no_translate * self.m22 - y_no_translate * self.m21) * inv_det,
            (-x_no_translate * self.m12 + y_no_translate * self.m11) * inv_det,
        ))
    }

    /// 逆变换点（不检查，直接返回结果）
    ///
    /// 如果你确定矩阵是可逆的，可以使用此方法避免 Option 开销
    ///
    /// # Panics
    /// 如果矩阵不可逆（行列式接近 0），结果将是 NaN 或 Infinity
    #[inline]
    pub fn inverse_transform_point_unchecked(&self, x: f32, y: f32) -> (f32, f32) {
        let inv_det = 1.0 / self.determinant();
        let x_no_translate = x - self.dx;
        let y_no_translate = y - self.dy;

        (
            (x_no_translate * self.m22 - y_no_translate * self.m21) * inv_det,
            (-x_no_translate * self.m12 + y_no_translate * self.m11) * inv_det,
        )
    }
}

// 运算符重载：matrix_a * matrix_b
impl Mul for Transform2D {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.multiply(&rhs)
    }
}
