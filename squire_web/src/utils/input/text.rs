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
    #[prop_or_default]
    pub default_text: String,
}

pub struct TextInput<T = ()> {
    pub label: Cow<'static, str>,
    pub process: Callback<String, T>,
    pub current_text: String,
    pub input: NodeRef,
}

impl<T> TextInput<T> {
    /*
    pub fn new(label: Cow<'static, str>, process: Callback<String, T>, default_text: String) -> Self {
        Self {
            label,
            process,
            current_text : default_text,
            input: Default::default(),
        }
    }
    */

    pub fn get_data(&self) -> String {
        self.input
            .cast::<HtmlInputElement>()
            .map(|e| e.value())
            .unwrap_or_default()
    }

    pub fn process(&mut self) -> T {
        self.current_text = self.get_data();
        self.process.emit(self.current_text.clone())
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
            current_text: props.default_text.clone(),
            input: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, _msg: Self::Message) -> bool {
        self.process();
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_input = ctx.link().callback(|_| ());
        html! {
            <>
                <label>{ self.label.as_ref() }</label>
                <input ref={self.input.clone()} type="text" oninput={ on_input } value={ self.current_text.clone() } />
            </>
        }
    }
}
