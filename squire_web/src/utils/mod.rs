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

/*
pub fn tabularize<H, I, T, M>(header: H, iter: I, map: M) -> Html
where
H: Iterator<Item=String>,
I: Iterator<Item=T>,
M: FnMut(T) -> IntoIterator<Item=String, IntoIter=String>
{
    let list = iter
        .map(map)
        .map(|inner| html!{ <tr> { inner.map(|item| html! { <td> { item } </td> }) } </tr> })
        .collect::<Html>();
    let header = html! {
        <thead>
            <tr>
                { header.map(|h| html! { <th> { h } </th> }).collect::<Html>() }
            </tr>
        </thead>
        };
    html! {
        <>
        <table class="table">
            { header }
            <tbody> { list } </tbody>
        </table>
        </>
    }
}
*/
