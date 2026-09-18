//! Item search: API queries, optional hovered-item capture and search control.

mod api;
mod capture;
mod hotkey;

pub(crate) use api::{__cmd__search_mxl_items, search_mxl_items, MxlItemApiState};
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub(crate) use capture::{read_hovered_item_detail, HoveredItemHook};
pub(crate) use hotkey::{
    __cmd__update_item_search_hotkey, update_item_search_hotkey, ItemSearchHotkeyState,
};
