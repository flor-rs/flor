// use windows::Win32::Foundation::HWND;
// use windows::Win32::System::Ole::{IDropTarget, IDropTarget_Impl};
// use crate::util::message_convert::get_key_state;
//
// #[windows::core::implement(IDropTarget)]
// #[derive(Clone)]
// pub struct DropTarget {
//     hwnd: HWND,
// }
//
// impl DropTarget {
//     pub fn new(hwnd: HWND) -> Self {
//         DropTarget { hwnd }
//     }
// }
//
// impl IDropTarget_Impl  for DropTarget {
//     fn DragEnter(&self, pdataobj: Option<&IDataObject>, grfkeystate: MODIFIERKEYS_FLAGS, pt: &POINTL, pdweffect: *mut DROPEFFECT) -> windows::core::Result<()> {
//
//         dbg!("DragEnter");
//         Ok(())
//     }
//
//     fn DragOver(&self, grfkeystate: MODIFIERKEYS_FLAGS, pt: &POINTL, pdweffect: *mut DROPEFFECT) -> windows::core::Result<()> {
//         unsafe {
//             let key_state = get_key_state(grfkeystate);
//             let mut pt = POINT { x: pt.x, y: pt.y };
//             ScreenToClient(self.hwnd, &mut pt);
//             let mouse_position = MousePosition {
//                 x: pt.x,
//                 y: pt.y,
//             };
//             let mut drop_effect = DropEffect::None;
//
//             // PROC_HANDLER(self.hwnd.into(), Message::DragOver { key_state, mouse_position, drop_effect: &mut drop_effect });
//             *pdweffect = match drop_effect {
//                 DropEffect::Copy => { DROPEFFECT_COPY }
//                 DropEffect::Link => { DROPEFFECT_LINK }
//                 DropEffect::Move => { DROPEFFECT_MOVE }
//                 DropEffect::None => { DROPEFFECT_NONE }
//                 DropEffect::Scroll => { DROPEFFECT_SCROLL }
//             };
//         }
//         Ok(())
//     }
//
//     fn DragLeave(&self) -> windows::core::Result<()> {
//         dbg!("DragLeave");
//         Ok(())
//     }
//
//     fn Drop(&self, pdataobj: Option<&IDataObject>, grfkeystate: MODIFIERKEYS_FLAGS, pt: &POINTL, pdweffect: *mut DROPEFFECT) -> windows::core::Result<()> {
//         dbg!("Drop");
//         if let Some(x) = pdataobj {
//             unsafe {
//                 let mut s = FORMATETC::default();
//                 x.GetData(&mut s)?;
//                 let s = (*s.ptd).tdData.clone();
//             }
//         }
//         Ok(())
//     }
// }