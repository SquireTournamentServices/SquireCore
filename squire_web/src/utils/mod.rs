pub mod input;
pub mod requests;
pub mod popout;

pub use input::*;
pub use requests::*;
pub use popout::*;
use wasm_bindgen::__rt::RefMut;

/// A wrapper around web_sys console log_1
pub fn console_log(info: &str) {
    web_sys::console::log_1(&info.into())
}

pub fn digest_if_different<T>(data: T, storage: &mut T) -> bool
where
    T: PartialEq,
{
    let digest = *storage != data;
    *storage = data;
    digest
}
