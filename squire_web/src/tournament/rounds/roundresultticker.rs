use std::{marker::PhantomData, str::FromStr, rc::Rc, fmt::Display};
use squire_sdk::{model::rounds::RoundResult, players::Round};
use yew::prelude::*;

pub struct RoundResultTicker {
    pub label: &'static str,
    // TODO : Some kind of callback to the RoundsView context with a RoundResult
    pub result_type: RoundResult,
    pub stored_value: u32,
}

impl RoundResultTicker {

    pub fn new(
        label: &'static str,
        // TODO : Some kind of callback to the RoundsView context with a RoundResult
        result_type: RoundResult,
        stored_value: u32,

    ) -> Self
    {
        Self {
            label,
            result_type,
            stored_value,
        }
    }

    pub fn update(&mut self, dir: u32) -> bool {
        self.stored_value += dir;
        true
    }

    #[allow(clippy::option_map_unit_fn)]
    pub fn view<T: Display>(&self, data: T) -> Html {
        // Here we would clone the callback
        let up = move |s| {
            // TODO : Emit message ticking stored value up;
            todo!();
        };
        let down = move |s| {
            // TODO : Emit message ticking stored value down;
            todo!();
        };
        html! {
            <>
                <>{format!("{} {data}", self.label)}</>
                <button onclick={up}>{"+"}</button>
                <button onclick={down}>{"-"}</button> 
            </>
        }
    }

}