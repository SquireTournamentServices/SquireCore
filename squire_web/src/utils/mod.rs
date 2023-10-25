pub mod input;
pub mod popout;

pub use input::*;
pub use popout::*;
use yew::{html, Html};

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

/* Standings example:
 * [ (name, score), ... ]
 * -> Vec<(String, StandardScore)>
 *
 * Place | Name | score.match_points | ..
 * Place | Name | score.match_points | ..
 * Place | Name | score.match_points | ..
 * Place | Name | score.match_points | ..
 */
/* Pairings example:
 * [ [name, name, ...], ...]
 * -> Vec<Vec<String>>
 *
 * Name1 | Name2 | ..
 * Name1 | Name2 | ..
 * Name1 | Name2 | ..
 * Name1 | Name2 | ..
 */
#[allow(dead_code)]
pub fn tabularize<H, OUTER, INNER, M>(header: H, outer: OUTER) -> Html
where
    H: Iterator<Item = String>,
    OUTER: Iterator<Item = INNER>,
    INNER: IntoIterator<Item = String>,
{
    /*
     * <tr> <td> str11 </td> | str12 | str13 | ... </tr>
     * str21 | str22 | str23 | ...
     * ...
     *
     */
    let header = html! {
    <thead>
        <tr>
            { header.map(|h| html! { <th> { h } </th> }).collect::<Html>() }
        </tr>
    </thead>
    };
    let body = outer.map(|inner: INNER| html!{ <tr> { for inner.into_iter().map(|item| html! { <td>{item}</td> }) } </tr> });
    html! {
        <>
        <table class="table">
            { header }
            <tbody> { for body } </tbody>
        </table>
        </>
    }
}
