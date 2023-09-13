use std::borrow::Cow;

use squire_sdk::{
    api::TournamentSummary,
    model::{
        identifiers::TournamentId,
        tournament::{TournamentPreset, TournamentSeed},
    },
};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{utils::TextInput, Route, CLIENT};

#[derive(Debug, PartialEq, Clone)]
pub struct TournamentCreator {
    pub tourn_list: Option<Vec<TournamentSummary>>,
    pub send_create_tourn: Callback<TournamentId>,
    pub new_tourn_name: String,
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct TournamentCreatorProps {}

pub enum TournamentCreatorMessage {
    TournsReady(Option<Vec<TournamentSummary>>),
    CreateTourn,
    TournCreated(TournamentId),
    TournNameInput(String),
}

impl Component for TournamentCreator {
    type Message = TournamentCreatorMessage;
    type Properties = TournamentCreatorProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            TournamentCreatorMessage::TournsReady(CLIENT.get().unwrap().get_tourn_summaries().await)
        });
        Self {
            tourn_list: None,
            send_create_tourn: ctx.link().callback(TournamentCreatorMessage::TournCreated),
            new_tourn_name: TournamentSeed::default_name(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TournamentCreatorMessage::TournsReady(t_list) => {
                self.tourn_list = t_list;
                true
            }
            TournamentCreatorMessage::CreateTourn => {
                let new_tourn_name = self.new_tourn_name.clone();
                ctx.link().send_future(async {
                    let t_seed = TournamentSeed::new(
                        new_tourn_name,
                        TournamentPreset::Swiss,
                        "EDH".to_owned(),
                    )
                    .unwrap();
                    let client = CLIENT.get().unwrap();
                    // TODO: This can fail because the seed might be bad or because the user is not
                    // logged in. We should properly handle the error case.
                    let id = client.create_tournament(t_seed).await.unwrap();
                    let _ = client.persist_tourn_to_backend(id).await;
                    TournamentCreatorMessage::TournCreated(id)
                });
                false
            }
            TournamentCreatorMessage::TournCreated(id) => {
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::Tourn { id });
                false
            }
            TournamentCreatorMessage::TournNameInput(input) => {
                self.new_tourn_name = input;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let list = if let Some(tourns) = &self.tourn_list {
            tourns
                .iter()
                .map(
                    |TournamentSummary {
                         id,
                         name,
                         status,
                         format,
                     }| {
                        let id = *id;
                        let nav = ctx.link().navigator().unwrap();
                        let cb = Callback::from(move |_| {
                            nav.push(&Route::Tourn { id });
                        });
                        html! {
                            <>
                            <tr onclick = { cb }>
                                <td>{ name }</td><td>{ format }</td><td>{ status }</td>
                            </tr>
                            </>
                        }
                    },
                )
                .collect::<Html>()
        } else {
            Html::default()
        };
        let onclick = ctx
            .link()
            .callback(|_| TournamentCreatorMessage::CreateTourn);
        html! {
            <div class="container">
                <div class="py-3">
                    <TextInput label = {Cow::from("Tournament Name: ")} process = {ctx.link().callback(TournamentCreatorMessage::TournNameInput)} default_text={ self.new_tourn_name.clone() } />
                    <button {onclick}>{"Create Tournament"}</button>
                </div>
                <hr />
                <div class="py-3">
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{ "Name" }</th>
                                <th>{ "Format" }</th>
                                <th>{ "Status" }</th>
                            </tr>
                        </thead>
                        <tbody>{ list }</tbody>
                    </table>
                </div>
            </div>
        }
    }
}
