//! Item Extra Info configuration state (Pure read-only, no code injection)

pub struct ItemExtraInfoHook {
    pub enabled: bool,
    pub show_sockets_and_eth: bool,
}

impl ItemExtraInfoHook {
    pub fn new() -> Self {
        Self {
            enabled: true,
            show_sockets_and_eth: true,
        }
    }

    pub fn reset_injection_state(&mut self) {}

    pub fn is_injected(&self) -> bool {
        false
    }
}
