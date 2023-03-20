use std::marker::PhantomData;

use web_sys::HtmlInputElement;

use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TransformerProps<A, B, F, O>
where
    F: PartialEq + FnMut(A) -> B,
    O: PartialEq,
    B: PartialEq,
    A: PartialEq,
{
    pub process: F,
    pub callback: Callback<B, O>,
    marker: PhantomData<(A)>,
}

pub struct Transformer<A, B, F, O> {
    pub process: F,
    pub callback: Callback<B, O>,
    marker: PhantomData<(A)>,
}

fn ignore<A, B>(_: A) -> B
where
    B: Default,
{
    B::default()
}

impl<A, B, F, O> Component for Transformer<A, B, F, O>
where
    F: 'static + PartialEq + FnMut(A) -> B,
    O: 'static + PartialEq,
    B: 'static + PartialEq + Default,
    A: 'static + PartialEq,
{
    type Message = String;
    type Properties = TransformerProps<A, B, F, O>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            process: ignore,
            callback: Callback::default(),
            marker: PhantomData,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.process();
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        todo!();
        html! {}
    }
}
