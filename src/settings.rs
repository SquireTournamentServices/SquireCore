use std::collections::HashMap;
use std::iter::Iterator;

/// A struct that contains generic settings info
pub struct Settings {
    pub settings: HashMap<String, String>
}

impl Settings {
    /// Creates a new Settings object
    pub fn new() -> Self {
        Settings { settings: HashMap::new() }
    }

    /// Returns a new settings objects whose settings are a subset of this Settings's settings. The
    /// give iterator defines the keys for the subset of settings
    pub fn collect(&self, iter: impl Iterator<Item=String>) -> Self {
        let mut settings = HashMap::new();
        for s in iter {
            if let Some(val) = self.settings.get(&s) {
                settings.insert(s, val.clone());
            }
        }
        Settings { settings }
    }

    /// Does what collect does, but removes the elements instead of cloning them
    pub fn divide(&mut self, iter: impl Iterator<Item=String>) -> Self {
        let mut settings = HashMap::new();
        for s in iter {
            if let Some(val) = self.settings.remove(&s) {
                settings.insert(s, val.clone());
            }
        }
        Settings { settings }
    }
}

#[cfg(feature = "ffi")]
impl Settings {
    /// Returns a new settings objects whose settings are a subset of this Settings's settings. The
    /// give iterator defines the keys for the subset of settings
    #[no_mangle]
    pub extern "C" fn collect_c(&self, iter: Vec<String>) -> Self {
        let mut settings = HashMap::new();
        for s in iter {
            if let Some(val) = self.settings.get(&s) {
                settings.insert(s, val.clone());
            }
        }
        Settings { settings }
    }

    /// Does what collect does, but removes the elements instead of cloning them
    #[no_mangle]
    pub extern "C" fn divide_c(&mut self, iter: Vec<String>) -> Self {
        let mut settings = HashMap::new();
        for s in iter {
            if let Some(val) = self.settings.remove(&s) {
                settings.insert(s, val.clone());
            }
        }
        Settings { settings }
    }
}

/// A struct used to communicate the application of a Settings objects
#[repr(C)]
pub struct SettingsResult {
    pub accepted: Settings,
    pub rejected: Settings,
    pub errored: Settings,
}

impl SettingsResult {
    /// Creates a new SettingsResult
    pub fn new(accepted: Settings, rejected: Settings, errored: Settings) -> Self {
        SettingsResult {
            accepted,
            rejected,
            errored
        }
    }

    /// Checks if there were any "bad" settings
    pub fn was_success(&self) -> bool {
        self.rejected.settings.len() + self.errored.settings.len() == 0
    }
}

#[cfg(feature = "ffi")]
impl SettingsResult {
    /// Checks if there were any "bad" settings
    #[no_mangle]
    pub extern "C" fn was_success_c(&self) -> bool {
        self.rejected.settings.len() + self.errored.settings.len() == 0
    }
}
