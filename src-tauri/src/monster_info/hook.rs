//! Monster Info configuration state (Pure read-only, no code injection)

pub struct MonsterInfoHook {
    pub enabled: bool,
    pub show_id: bool,
}

impl MonsterInfoHook {
    pub fn new() -> Self {
        Self {
            enabled: true,
            show_id: true,
        }
    }

    pub fn reset_injection_state(&mut self) {}

    pub fn is_injected(&self) -> bool {
        false
    }
}
