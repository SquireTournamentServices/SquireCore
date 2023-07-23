pub mod input;
pub mod popout;
pub mod requests;

pub use input::*;
pub use popout::*;
pub use requests::*;

/// A wrapper around web_sys console log_1
#[allow(unused)]
pub fn console_log(info: &str) {
    web_sys::console::log_1(&info.into())
}

/*
pub fn digest_if_different<T>(data: T, storage: &mut T) -> bool
where
    T: PartialEq,
{
    let digest = *storage != data;
    *storage = data;
    digest
}
*/
