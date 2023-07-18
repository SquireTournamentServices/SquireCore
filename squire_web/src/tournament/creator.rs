use std::borrow::Cow;

use squire_sdk::{tournaments::{TournamentId, TournamentPreset, TournamentSummary}, model::tournament::TournamentSeed};
use yew::{prelude::*};
use yew_router::prelude::*;

use crate::{Route, CLIENT, utils::{TextInput, console_log}};


// NOTES :
// Remake this whole thing as a struct component with updates and whatnot
// https://yew.rs/docs/next/concepts/router#struct-components

/*
#[function_component(TournamentCreator)]
pub fn tournament_creator() -> Html {
    todo!()
    /*
    let navigator = use_navigator().unwrap();

    let onclick_callback = Callback::from(move |_| {
        let client = CLIENT.get().unwrap();
        let id = client.create_tournament(
            "Some Tournament".to_owned(),
            TournamentPreset::Swiss,
            "Some Tournament".to_owned(),
        ).await;
        navigator.push(&Route::Tourn { id })
    });
    html! {
        <div>
            <button onclick={onclick_callback}>{ "Create tournament!" }</button>
        </div>
    }
    */
}
*/

#[derive(Debug, PartialEq, Clone)]
pub struct TournamentCreator {
    pub tourn_list : Option<Vec<TournamentSummary>>,
    pub send_tourn_list: Callback<Vec<TournamentSummary>>,
    pub send_create_tourn: Callback<TournamentId>,
    pub new_tourn_name: String,
}

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct TournamentCreatorProps {
}

pub enum TournamentCreatorMessage {
    TournsReady(Vec<TournamentSummary>),
    CreateTourn,
    TournCreated(TournamentId),
    TournNameInput(String),
}

impl Component for TournamentCreator {
    type Message = TournamentCreatorMessage;
    type Properties = TournamentCreatorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut to_return = Self {
            tourn_list : None,
            send_tourn_list : ctx.link().callback(TournamentCreatorMessage::TournsReady),
            send_create_tourn : ctx.link().callback(TournamentCreatorMessage::TournCreated),
            new_tourn_name: "Tournament Name".to_owned(),
        };
        to_return.query_tourns(ctx);
        to_return
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TournamentCreatorMessage::TournsReady(t_list) => {
                self.tourn_list = Some(t_list);
                true
            },
            TournamentCreatorMessage::CreateTourn => {
                let new_tourn_name = self.new_tourn_name.clone();
                ctx.link().send_future(async {
                    console_log("Sending future...");
                    let _account = CLIENT.get().unwrap().get_user().clone();
                    let t_seed = TournamentSeed::new(
                        new_tourn_name,
                        TournamentPreset::Swiss,
                        "EDH".to_owned(),
                    ).unwrap();
                    console_log("Seed created!");
                    let id = CLIENT
                    .get()
                    .unwrap()
                    .create_tournament(t_seed)
                    .await;
                    console_log("Tourn created!!!");
                    TournamentCreatorMessage::TournCreated(id)
                });
                false
            },
            TournamentCreatorMessage::TournCreated(id) => {
                console_log("Tourn created! Routing...");
                let navigator = ctx.link().navigator().unwrap();
                navigator.push(&Route::Tourn {id});
                false
            },
            TournamentCreatorMessage::TournNameInput(input) => {
                self.new_tourn_name = input;
                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let list = html!{
            <>
                <tr>
                    <td>{ "Test Tourny" }</td><td>{ "EDH" }</td><td>{ "Active" }</td>
                </tr>
                <tr>
                    <td>{ "Test Tourny" }</td><td>{ "EDH" }</td><td>{ "Active" }</td>
                </tr>
                <tr>
                    <td>{ "Test Tourny" }</td><td>{ "EDH" }</td><td>{ "Active" }</td>
                </tr>
            </>
        };
        let onclick = ctx.link().callback(|_| TournamentCreatorMessage::CreateTourn);
        html! {
            <div class="container">
                <div class="py-3">
                    <TextInput label = {Cow::from("Tournament Name: ")} process = {ctx.link().callback(move |s| TournamentCreatorMessage::TournNameInput(s))} default_text={"Default Name".to_owned()} />
                    <br />
                    <label for="format">{ "Format:" }</label>
                    <select name="format" id="format">
                        <option value="EDH">{ "EDH" }</option>
                        <option value="Standard">{ "Standard" }</option>
                        <option value="Pioneer">{ "Pioneer" }</option>
                        <option value="Something else">{ "Something else" }</option>
                    </select>
                    <br />
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

impl TournamentCreator {
    fn query_tourns(&mut self, ctx: &Context<Self>) {
        /*
        let tracker: QueryTracker<Vec<TournamentSummary>> = todo!(); // Get list of active tourns
        let send_tourn_list = self.send_tourn_list.clone();
        spawn_local(async move {
            console_log("Waiting for update to finish!");
            send_tourn_list.emit(tracker.process().await.unwrap())
        });
        */
        ctx.link().send_future(async {
            TournamentCreatorMessage::TournsReady(Vec::new())
        })
    }
}
