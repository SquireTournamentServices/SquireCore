use std::borrow::Cow;

use web_sys::HtmlInputElement;

use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TextInputProps<T = ()>
where
    T: PartialEq,
{
    pub label: Cow<'static, str>,
    pub process: Callback<String, T>,
}

pub struct TextInput<T = ()> {
    pub label: Cow<'static, str>,
    pub process: Callback<String, T>,
    pub input: NodeRef,
}

impl<T> TextInput<T> {
    pub fn new(label: Cow<'static, str>, process: Callback<String, T>) -> Self {
        Self {
            label,
            process,
            input: Default::default(),
        }
    }

    pub fn get_data(&self) -> String {
        self.input
            .cast::<HtmlInputElement>()
            .map(|e| e.value())
            .unwrap_or_default()
    }

    pub fn process(&self) -> T {
        self.process.emit(self.get_data())
    }
}

impl<T> Component for TextInput<T>
where
    T: 'static + PartialEq,
{
    type Message = ();
    type Properties = TextInputProps<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            label: props.label.clone(),
            process: props.process.clone(),
            input: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.process();
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_input = ctx.link().callback(|_| ());
        html! {
            <div>
                <label>{ self.label.clone() }</label>
                <input ref={self.input.clone()} type="text" oninput={ on_input } />
            </div>
        }
    }
}
