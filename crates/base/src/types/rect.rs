/// 通用矩形类型，支持不同的坐标和尺寸类型
///
/// # 泛型参数
/// - `XY`: 坐标类型 (x, y)，通常是 `f32` 或 `i32`
/// - `WH`: 尺寸类型 (w, h)，通常是 `f32`、`i32` 或 `u32`
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect<XY = f32, WH = f32> {
    pub x: XY,
    pub y: XY,
    pub w: WH,
    pub h: WH,
}

// ==================== 构造方法 ====================

impl<XY, WH> Rect<XY, WH> {
    /// 创建一个新的矩形
    #[inline]
    pub const fn new(x: XY, y: XY, w: WH, h: WH) -> Self {
        Self { x, y, w, h }
    }
}

impl<XY: Default, WH: Default> Default for Rect<XY, WH> {
    fn default() -> Self {
        Self {
            x: XY::default(),
            y: XY::default(),
            w: WH::default(),
            h: WH::default(),
        }
    }
}

// ==================== f32, f32 实现 ====================

impl Rect<f32, f32> {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 0.0,
        h: 0.0,
    };

    #[inline]
    pub fn zero_f32() -> Self {
        Self::ZERO
    }

    /// 从位置和尺寸创建
    #[inline]
    pub fn from(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// 从两个对角点创建 (左上角和右下角)
    #[inline]
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            x: left,
            y: top,
            w: right - left,
            h: bottom - top,
        }
    }

    // ---- 边界访问器 ----

    #[inline]
    pub fn left(&self) -> f32 {
        self.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.y
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.w
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.h
    }

    /// 中心点 X
    #[inline]
    pub fn center_x(&self) -> f32 {
        self.x + self.w * 0.5
    }

    /// 中心点 Y
    #[inline]
    pub fn center_y(&self) -> f32 {
        self.y + self.h * 0.5
    }

    /// 中心点
    #[inline]
    pub fn center(&self) -> (f32, f32) {
        (self.center_x(), self.center_y())
    }

    // ---- 命中测试 ----

    /// 检测点是否在矩形内（包含边界）
    #[inline]
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }

    /// 检测点是否在矩形内（不包含边界）
    #[inline]
    pub fn contains_point_exclusive(&self, px: f32, py: f32) -> bool {
        px > self.x && px < self.right() && py > self.y && py < self.bottom()
    }

    /// 检测另一个矩形是否完全在此矩形内
    #[inline]
    pub fn contains_rect(&self, other: &Rect<f32, f32>) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    // ---- 相交检测 ----

    /// 检测是否与另一个矩形相交
    #[inline]
    pub fn intersects(&self, other: &Rect<f32, f32>) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// 计算与另一个矩形的交集，如果不相交则返回 None
    #[inline]
    pub fn intersection(&self, other: &Rect<f32, f32>) -> Option<Rect<f32, f32>> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if x < right && y < bottom {
            Some(Rect::from_ltrb(x, y, right, bottom))
        } else {
            None
        }
    }

    /// 计算包含两个矩形的最小矩形（并集）
    #[inline]
    pub fn union(&self, other: &Rect<f32, f32>) -> Rect<f32, f32> {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::from_ltrb(x, y, right, bottom)
    }

    // ---- 变换 ----

    /// 平移矩形
    #[inline]
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            w: self.w,
            h: self.h,
        }
    }

    /// 向内收缩（padding 效果）
    #[inline]
    pub fn inset(&self, amount: f32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            w: (self.w - amount * 2.0).max(0.0),
            h: (self.h - amount * 2.0).max(0.0),
        }
    }

    /// 向内收缩（分别指定四个方向）
    #[inline]
    pub fn inset_ltrb(&self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            x: self.x + left,
            y: self.y + top,
            w: (self.w - left - right).max(0.0),
            h: (self.h - top - bottom).max(0.0),
        }
    }

    /// 向外扩展
    #[inline]
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            w: self.w + amount * 2.0,
            h: self.h + amount * 2.0,
        }
    }

    /// 缩放矩形（以左上角为锚点）
    #[inline]
    pub fn scale(&self, sx: f32, sy: f32) -> Self {
        Self {
            x: self.x * sx,
            y: self.y * sy,
            w: self.w * sx,
            h: self.h * sy,
        }
    }

    /// 缩放矩形（以中心为锚点）
    #[inline]
    pub fn scale_from_center(&self, sx: f32, sy: f32) -> Self {
        let cx = self.center_x();
        let cy = self.center_y();
        let new_w = self.w * sx;
        let new_h = self.h * sy;
        Self {
            x: cx - new_w * 0.5,
            y: cy - new_h * 0.5,
            w: new_w,
            h: new_h,
        }
    }

    // ---- 状态检查 ----

    /// 矩形是否为空（宽或高为 0 或负数）
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    /// 矩形面积
    #[inline]
    pub fn area(&self) -> f32 {
        self.w * self.h
    }

    // ---- 转换 ----

    /// 转换为元组 (x, y, w, h)
    #[inline]
    pub fn to_tuple(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }

    /// 转换为 i32 矩形（四舍五入）
    #[inline]
    pub fn to_i32(&self) -> Rect<i32, i32> {
        Rect {
            x: self.x.round() as i32,
            y: self.y.round() as i32,
            w: self.w.round() as i32,
            h: self.h.round() as i32,
        }
    }

    /// 转换为 i32, u32 矩形
    #[inline]
    pub fn to_iu32(&self) -> Rect<i32, u32> {
        Rect {
            x: self.x.round() as i32,
            y: self.y.round() as i32,
            w: self.w.round().max(0.0) as u32,
            h: self.h.round().max(0.0) as u32,
        }
    }
}

// ==================== i32, i32 实现 ====================

impl Rect<i32, i32> {
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };

    #[inline]
    pub fn zero_i32() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    #[inline]
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    /// 检测点是否在矩形内
    #[inline]
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }

    /// 检测点是否在矩形内（f32 坐标版本）
    #[inline]
    pub fn contains_point_f32(&self, px: f32, py: f32) -> bool {
        px >= self.x as f32
            && px <= self.right() as f32
            && py >= self.y as f32
            && py <= self.bottom() as f32
    }

    /// 检测是否与另一个矩形相交
    #[inline]
    pub fn intersects(&self, other: &Rect<i32, i32>) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// 转换为 f32 矩形
    #[inline]
    pub fn to_f32(&self) -> Rect<f32, f32> {
        Rect {
            x: self.x as f32,
            y: self.y as f32,
            w: self.w as f32,
            h: self.h as f32,
        }
    }
}

// ==================== i32, u32 实现 ====================

impl Rect<i32, u32> {
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };

    #[inline]
    pub fn zero_iu32() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn right(&self) -> i32 {
        self.x + self.w as i32
    }

    #[inline]
    pub fn bottom(&self) -> i32 {
        self.y + self.h as i32
    }

    /// 检测点是否在矩形内
    #[inline]
    pub fn contains_point(&self, px: i32, py: i32) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }

    /// 检测点是否在矩形内（f32 坐标版本）
    #[inline]
    pub fn contains_point_f32(&self, px: f32, py: f32) -> bool {
        px >= self.x as f32
            && px <= self.right() as f32
            && py >= self.y as f32
            && py <= self.bottom() as f32
    }

    /// 转换为 f32 矩形
    #[inline]
    pub fn to_f32(&self) -> Rect<f32, f32> {
        Rect {
            x: self.x as f32,
            y: self.y as f32,
            w: self.w as f32,
            h: self.h as f32,
        }
    }

    /// 转换为元组 (x, y, w, h)
    #[inline]
    pub fn to_tuple(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.w, self.h)
    }
}

// ==================== From 实现 ====================

// ---- 元组 → Rect ----

impl From<(f32, f32, f32, f32)> for Rect<f32, f32> {
    #[inline]
    fn from((x, y, w, h): (f32, f32, f32, f32)) -> Self {
        Self { x, y, w, h }
    }
}

impl From<(i32, i32, i32, i32)> for Rect<i32, i32> {
    #[inline]
    fn from((x, y, w, h): (i32, i32, i32, i32)) -> Self {
        Self { x, y, w, h }
    }
}

impl From<(i32, i32, u32, u32)> for Rect<i32, u32> {
    #[inline]
    fn from((x, y, w, h): (i32, i32, u32, u32)) -> Self {
        Self { x, y, w, h }
    }
}

// ---- Rect → 元组 ----

impl From<Rect<f32, f32>> for (f32, f32, f32, f32) {
    #[inline]
    fn from(r: Rect<f32, f32>) -> Self {
        (r.x, r.y, r.w, r.h)
    }
}

impl From<Rect<i32, i32>> for (i32, i32, i32, i32) {
    #[inline]
    fn from(r: Rect<i32, i32>) -> Self {
        (r.x, r.y, r.w, r.h)
    }
}

impl From<Rect<i32, u32>> for (i32, i32, u32, u32) {
    #[inline]
    fn from(r: Rect<i32, u32>) -> Self {
        (r.x, r.y, r.w, r.h)
    }
}
