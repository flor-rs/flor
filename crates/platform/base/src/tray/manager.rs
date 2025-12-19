use crate::tray::event::TrayEvent;
use crate::tray::id::TrayId;
use crate::tray::options::TrayOptions;

pub trait TrayManagerEntry {
    type TrayId: TrayId;
    type Error;
    fn init() -> Result<(), Self::Error>;

    /// 添加托盘图标
    /// id: 业务层分配的唯一 ID
    /// options: 图标路径、提示文字等
    fn add(options: &TrayOptions) -> Result<Self::TrayId, Self::Error>;

    fn update(tray_id: Self::TrayId, options: &TrayOptions) -> Result<(), Self::Error>;

    fn remove(tray_id: Self::TrayId) -> Result<(), Self::Error>;

    /// 本质上是set
    fn on_callback(f: impl Fn(Self::TrayId, TrayEvent) + Send + Sync + 'static);
}
