use std::{marker::PhantomData, str::FromStr, rc::Rc, fmt::Display};
use yew::prelude::*;

pub struct RoundUpDown {
    label: &'static str,
    convert: Rc<dyn Fn(u32) -> Option<RoundResult>>,
    emitter: Callback<RoundResult>,
    stored_value: u32,
    stored_result: Option<RoundResult>,
}

fn make_chain<T, F, S>(f: F) -> impl Clone + Fn(u32) -> Option<RoundResult>
where
    T: 'static + FromStr,
    F: 'static + Copy + Fn(T) -> S,
    S: 'static + Into<RoundResult>,
{
    move |s: u32| s.parse().ok().map(f).map(Into::into)
}

impl RoundUpDown {

    pub fn new<T, G, S>(
        label: &'static str,
        convert: G,
        emitter: Callback<RoundResult>,
    ) -> RoundUpDown
    where
        T: 'static + FromStr,
        G: 'static + Copy + Fn(T) -> S,
        S: 'static + Into<RoundResult>,
    {
        RoundUpDown {
            label,
            convert: Rc::new(make_chain(convert)),
            emitter,
        }
    }

    pub fn update(&mut self, dir: u8) -> bool {
        self.stored_value += dir;
        true
    }

    #[allow(clippy::option_map_unit_fn)]
    pub fn view<T: Display>(&self, data: T) -> Html {
        let convert = self.convert.clone();
        let emitter = self.emitter.clone();
        let process = move |s| {
            // convert(s).map(|out| emitter.emit(out));
            // TODO : Make the stored u32 go through the conversion
            todo!();
        };
        let up = move |s| {
            // TODO : Emit message ticking stored value up;
            // Convert then store in stored result
            todo();
        };
        let down = move |s| {
            // TODO : Emit message ticking stored value down;
            // Convert then store in stored result
            todo();
        };
        html! {
            <>
                <>{format!("{} {data}", self.label)}</>
                <button onclick=up>+</button>
                <button onclick=down>-</button> 
            </>
        }
    }

}