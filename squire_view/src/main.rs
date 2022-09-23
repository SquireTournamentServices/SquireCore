use std::{collections::HashMap, sync::RwLock};

use once_cell::sync::OnceCell;
use yew::prelude::*;

use squire_lib::{
    identifiers::TournamentId,
    tournament::{Tournament, TournamentPreset},
};

static TOURNAMENTS: OnceCell<RwLock<HashMap<TournamentId, Tournament>>> = OnceCell::new();

struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        let mut lock = TOURNAMENTS.get().unwrap().write().unwrap();
        let t = Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Pioneer".into());
        lock.insert(t.id, t);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let lock = TOURNAMENTS.get().unwrap().read().unwrap();
        let tourns = lock
            .values()
            .map(|tourn| render_tournament(tourn))
            .collect::<Html>();
        html! {
            <div>
                <button onclick={ctx.link().callback(|_| ())}>{ "Refresh" }</button>
                <p>{ tourns }</p>
            </div>
        }
    }
}

fn main() {
    let mut tourns = HashMap::new();
    let t = Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Pioneer".into());
    tourns.insert(t.id, t);
    TOURNAMENTS
        .set(RwLock::new(tourns))
        .expect("Could not populate Tournaments' OnceCell");
    yew::start_app::<Model>();
}

fn render_tournament(tourn: &Tournament) -> Html {
    todo!()
}
