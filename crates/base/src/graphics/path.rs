use std::hash::{Hash, Hasher};

/// Path 命令类型
#[derive(Clone, Debug)]
pub enum PathCommand {
    /// 移动到指定点，不绘制
    MoveTo(f32, f32),
    /// 绘制直线到指定点
    LineTo(f32, f32),
    /// 动态数量的贝塞尔曲线
    /// `points`：控制点列表，最后一个为终点
    /// 至少需要 2 个点（1 个控制点 + 终点）
    Bezier(Vec<(f32, f32)>),
    /// 闭合子路径
    Close,
}

// It is known that comparisons of values such as NaN cannot be handled here, but this does not have an impact, so no action is taken.
impl Hash for PathCommand {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PathCommand::MoveTo(x, y) => {
                0u8.hash(state);
                x.to_bits().hash(state);
                y.to_bits().hash(state);
            }
            PathCommand::LineTo(x, y) => {
                1u8.hash(state);
                x.to_bits().hash(state);
                y.to_bits().hash(state);
            }
            PathCommand::Bezier(vec) => {
                2u8.hash(state);
                vec.len().hash(state);
                for (x, y) in vec {
                    x.to_bits().hash(state);
                    y.to_bits().hash(state);
                }
            }
            PathCommand::Close => {
                3u8.hash(state);
            }
        }
    }
}

impl PartialEq for PathCommand {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::MoveTo(x1, y1), Self::MoveTo(x2, y2)) => x1 == x2 && y1 == y2,
            (Self::LineTo(x1, y1), Self::LineTo(x2, y2)) => x1 == x2 && y1 == y2,
            (Self::Bezier(points1), Self::Bezier(points2)) => {
                points1.len() == points2.len()
                    && points1
                        .iter()
                        .zip(points2)
                        .all(|(&(x1, y1), &(x2, y2))| x1 == x2 && y1 == y2)
            }
            (Self::Close, Self::Close) => true,
            _ => false,
        }
    }
}

impl Eq for PathCommand {}

/// Path 对象
#[derive(Debug, Hash, Eq, PartialEq, Clone, Default)]
pub struct Path {
    commands: Vec<PathCommand>,
}

impl Path {
    /// 移动到
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::MoveTo(x, y));
        self
    }

    /// 直线到
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::LineTo(x, y));
        self
    }

    /// 添加动态数量贝塞尔曲线
    /// points: 控制点列表，最后一个点为终点
    /// 至少需要 2 个点（1 个控制点 + 终点）
    pub fn bezier(mut self, points: Vec<(f32, f32)>) -> Self {
        if points.len() < 2 {
            panic!("Bezier curve requires at least 2 points (1 control + 1 end point)");
        }
        self.commands.push(PathCommand::Bezier(points));
        self
    }

    /// 闭合子路径
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    /// 获取 Path 的命令列表
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// 创建一个矩形路径
    /// (x, y): 左上角坐标
    /// width: 宽度
    /// height: 高度
    pub fn from_rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::default()
            .move_to(x, y) // 1. 移动到左上角
            .line_to(x + width, y) // 2. 画线到右上角
            .line_to(x + width, y + height) // 3. 画线到右下角
            .line_to(x, y + height) // 4. 画线到左下角
            .close() // 5. 闭合路径 (自动连回左上角)
    }

    /// 计算 Path 的 AABB 包围盒
    /// 返回元组: (x, y, width, height)
    /// 复杂度: O(N) - 极快
    pub fn get_bounds(&self) -> (f32, f32, f32, f32) {
        // 快速路径：空路径
        if self.commands.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        // 遍历所有命令
        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => {
                    // 内联的比较逻辑，编译器会优化
                    if *x < min_x {
                        min_x = *x;
                    }
                    if *x > max_x {
                        max_x = *x;
                    }
                    if *y < min_y {
                        min_y = *y;
                    }
                    if *y > max_y {
                        max_y = *y;
                    }
                }
                PathCommand::Bezier(points) => {
                    // 关键：直接遍历所有控制点和终点
                    // 这是一个“松包围盒”（Loose Bounds），但对于剔除来说是安全且正确的
                    for (x, y) in points {
                        if *x < min_x {
                            min_x = *x;
                        }
                        if *x > max_x {
                            max_x = *x;
                        }
                        if *y < min_y {
                            min_y = *y;
                        }
                        if *y > max_y {
                            max_y = *y;
                        }
                    }
                }
                PathCommand::Close => {}
            }
        }

        // 防御性编程：如果所有点都无效
        if min_x == f32::MAX {
            return (0.0, 0.0, 0.0, 0.0);
        }

        (min_x, min_y, max_x - min_x, max_y - min_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::collections::{HashMap, HashSet};
    use std::hash::{Hash, Hasher};

    // 辅助函数：计算哈希值
    fn hash_value<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_path_equality() {
        // 1. 相同路径应该相等
        let path1 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
                PathCommand::LineTo(0.0, 10.0),
                PathCommand::Close,
            ],
        };

        let path2 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
                PathCommand::LineTo(0.0, 10.0),
                PathCommand::Close,
            ],
        };

        assert_eq!(path1, path2);

        // 2. 不同路径应该不相等
        let path3 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(20.0, 0.0), // 坐标不同
                PathCommand::LineTo(20.0, 10.0),
                PathCommand::LineTo(0.0, 10.0),
                PathCommand::Close,
            ],
        };

        assert_ne!(path1, path3);

        // 3. 命令顺序不同应该不相等
        let path4 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(0.0, 10.0), // 顺序不同
                PathCommand::LineTo(10.0, 10.0),
                PathCommand::LineTo(10.0, 0.0),
                PathCommand::Close,
            ],
        };

        assert_ne!(path1, path4);

        // 4. 空路径应该相等
        let empty1 = Path::default();
        let empty2 = Path { commands: vec![] };
        assert_eq!(empty1, empty2);

        // 5. 包含贝塞尔曲线的路径
        let bezier_path1 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::Bezier(vec![(10.0, 0.0), (20.0, 10.0), (30.0, 0.0)]),
                PathCommand::Close,
            ],
        };

        let bezier_path2 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::Bezier(vec![(10.0, 0.0), (20.0, 10.0), (30.0, 0.0)]),
                PathCommand::Close,
            ],
        };

        let bezier_path3 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::Bezier(vec![(10.0, 0.0), (20.0, 10.0), (30.0, 1.0)]), // 终点不同
                PathCommand::Close,
            ],
        };

        assert_eq!(bezier_path1, bezier_path2);
        assert_ne!(bezier_path1, bezier_path3);
    }

    #[test]
    fn test_path_hash_consistency() {
        // 相同路径应该有相同哈希
        let path1 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
        };

        let path2 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
        };

        assert_eq!(hash_value(&path1), hash_value(&path2));

        // 不同路径应该不同哈希（大概率）
        let path3 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(20.0, 20.0), // 不同坐标
            ],
        };

        assert_ne!(hash_value(&path1), hash_value(&path3));
    }

    #[test]
    fn test_path_hashset_functionality() {
        let mut set = HashSet::new();

        let path1 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
        };

        let path2 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
        };

        let path3 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(20.0, 20.0),
            ],
        };

        set.insert(path1);
        assert_eq!(set.len(), 1);

        // 相同路径不应该重复插入
        set.insert(path2);
        assert_eq!(set.len(), 1);

        // 不同路径应该能插入
        set.insert(path3);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_path_hashmap_functionality() {
        let mut map = HashMap::new();

        let key1 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
        };

        let key2 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(10.0, 10.0),
            ],
        };

        let key3 = Path {
            commands: vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::LineTo(20.0, 20.0),
            ],
        };

        map.insert(key1.clone(), "value1");
        assert_eq!(map.get(&key1), Some(&"value1"));

        // 相同 key 应该获取到相同的 value
        assert_eq!(map.get(&key2), Some(&"value1"));

        // 更新 value
        map.insert(key2, "value2");
        assert_eq!(map.get(&key1), Some(&"value2"));

        // 不同 key 可以插入
        map.insert(key3, "value3");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_hash_eq_consistency() {
        // 哈希和相等必须一致
        // 如果 a == b，那么 hash(a) == hash(b)
        let a = Path {
            commands: vec![PathCommand::MoveTo(1.5, 2.5), PathCommand::LineTo(3.0, 4.0)],
        };

        let b = Path {
            commands: vec![PathCommand::MoveTo(1.5, 2.5), PathCommand::LineTo(3.0, 4.0)],
        };

        let c = Path {
            commands: vec![
                PathCommand::MoveTo(1.5, 2.5),
                PathCommand::LineTo(5.0, 6.0), // 不同
            ],
        };

        assert_eq!(a, b);
        assert_eq!(hash_value(&a), hash_value(&b));

        assert_ne!(a, c);
        // 哈希通常不同，但有可能碰撞
    }
}
