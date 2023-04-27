pub mod input;
pub mod requests;

pub use input::*;
pub use requests::*;

/// A wrapper around web_sys console log_1
pub fn console_log(info: &str) {
    web_sys::console::log_1(&info.into())
}
