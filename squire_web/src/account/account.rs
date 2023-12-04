use derive_more::From;
use yew::Html;
use yew::prelude::*;

#[derive(Debug, From)]
pub enum AccountMessage {
}

pub struct Account {
}

impl Account {
}

impl Component for Account {
    type Message = AccountMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        //match msg {}
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="m-lg-0 m-md-4 my-3">
                <div class="p-5 bg-light rounded-3">
                    <div class="container-fluid p-md-5">
                        <h1 class="display-5 fw-bold">{ "Account" }</h1>
                        <hr class="my-4"/>
                    </div>
                </div>
            </div>
        }
    }
}
