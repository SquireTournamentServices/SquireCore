use std::{collections::HashMap, sync::RwLock};

use yew::prelude::*;

use itertools::Itertools;
use once_cell::sync::OnceCell;
use uuid::Uuid;

use squire_lib::{
    accounts::{SharingPermissions, SquireAccount},
    admin::Admin,
    identifiers::{PlayerId, TournamentId, UserAccountId},
    operations::TournOp,
    player::Player,
    round::{Round, RoundId},
    settings::{PairingSetting, TournamentSetting},
    tournament::{Tournament, TournamentPreset},
};

static TOURNAMENTS: OnceCell<RwLock<HashMap<TournamentId, Tournament>>> = OnceCell::new();

#[derive(Clone, Copy, PartialEq)]
enum SetCursor {
    Player(PlayerId),
    Round(RoundId),
}

#[derive(Clone, PartialEq)]
struct Cursor<T: Clone + PartialEq>(T);

#[derive(Clone, Properties, PartialEq)]
struct TournamentView {
    id: TournamentId,
    plyr_cursor: Cursor<PlayerId>,
    rnd_cursor: Cursor<RoundId>,
}

impl Component for TournamentView {
    type Message = SetCursor;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let id = TOURNAMENTS
            .get()
            .unwrap()
            .read()
            .unwrap()
            .keys()
            .cloned()
            .next()
            .unwrap();
        Self {
            id,
            plyr_cursor: Cursor(PlayerId::new(Uuid::default())),
            rnd_cursor: Cursor(RoundId::new(Uuid::default())),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SetCursor::Player(plyr) => {
                self.plyr_cursor = Cursor(plyr);
            }
            SetCursor::Round(rnd) => {
                self.rnd_cursor = Cursor(rnd);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let lock = TOURNAMENTS.get().unwrap().read().unwrap();
        let tourn = lock.get(&self.id).unwrap();
        let tracking_player = tourn
            .player_reg
            .get_player(&(self.plyr_cursor.0.into()))
            .map(|p| player_printout(p))
            .unwrap_or_default();
        let player_summaries: Html = tourn
            .player_reg
            .players
            .values()
            .map(|plyr| {
                let content = format!(
                    "Player name: {}\tPlayer Id: {}\t Player status: {}",
                    plyr.name, plyr.id, plyr.status
                );
                let msg = SetCursor::Player(plyr.id.clone());
                let onclick = ctx.link().callback(move |_| msg);
                html! {
                    <p {onclick}> { content } </p>
                }
            })
            .collect();
        let tracking_round = tourn
            .round_reg
            .get_round(&(self.rnd_cursor.0.into()))
            .map(|r| round_printout(r))
            .unwrap_or_default();
        let round_summaries: Html = tourn
            .round_reg
            .rounds
            .values()
            .map(|rnd| {
                let msg = SetCursor::Round(rnd.id.clone());
                let onclick = ctx.link().callback(move |_| msg);
                html! {
                    <p {onclick}> { format!("Round number: {}\tRound Id: {}\tRound status: {}", rnd.match_number, rnd.id, rnd.status) } </p>
                }}
                )
            .collect();
        html! {
        <>
            <h1>{ "Tournament Manager" }</h1>
            <div>
                <p>{ format!("Name: {}", tourn.name) }</p>
                <p>{ format!("Id: {}", tourn.id) }</p>
                <p>{ format!("Status: {}", tourn.status) }</p>
                <hr/>
                <p>{ "Players:" }</p>
                { tracking_player }
                { player_summaries }
                <hr/>
                <p>{ "Matches:" }</p>
                { tracking_round }
                { round_summaries }
            </div>
        </>
        }
    }
}

fn main() {
    let mut tourns = HashMap::new();
    //let t = Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Pioneer".into());
    let t = load();
    tourns.insert(t.id, t);
    TOURNAMENTS
        .set(RwLock::new(tourns))
        .expect("Could not populate Tournaments' OnceCell");
    yew::start_app::<TournamentView>();
}

fn player_printout(plyr: &Player) -> Html {
    html! {
        <div>
            <hr/>
            <h3> { format!("{}", plyr.name) } </h3>
            <p> { format!("Status: {}", plyr.name) } </p>
            <p> { format!("Id: {}", plyr.id) } </p>
            <p> { format!("Gamer Tag: {}", plyr.name) } </p>
            <p> { format!("Decks: {}", plyr.deck_ordering.iter().join("\t")) } </p>
            <hr/>
        </div>
    }
}

fn round_printout(rnd: &Round) -> Html {
    html! {
        <div>
            <hr/>
            <h3> { format!("Round #{}", rnd.match_number) } </h3>
            <p> { format!("Id: {}", rnd.id) } </p>
            <p> { format!("Table number: {}", rnd.table_number) } </p>
            <p> { format!("Status: {}", rnd.status) } </p>
            <p> { format!("Is a bye?: {}", if rnd.is_bye() { "True" } else { "False" }) } </p>
            <p> { format!("Winner: {}", rnd.winner.map(|id| id.to_string()).unwrap_or_else(|| "None".into())) } </p>
            <p> { format!("Players: {}", rnd.players.iter().map(|id| id.to_string()).join("\t")) } </p>
            <p> { format!("Drops: {}", rnd.drops.iter().map(|id| id.to_string()).join("\t")) } </p>
            <p> { format!("Results: {}", rnd.results.iter().map(|(id, wins)| format!("{}: {wins}", *id)).join("\t")) } </p>
            <p> { format!("Draws: {}", rnd.draws) } </p>
            <p> { format!("Confirmations: {}", rnd.confirmations.iter().map(|id| id.to_string()).join("\t")) } </p>
            <hr/>
        </div>
    }
}

fn load() -> Tournament {
    let mut tourn = spoof_data(20);
    let admin_id = *tourn.admins.keys().next().unwrap();
    tourn.apply_op(TournOp::Start(admin_id)).unwrap();
    let plyrs: Vec<_> = tourn.player_reg.players.keys().cloned().collect();
    for id in plyrs {
        tourn.apply_op(TournOp::ReadyPlayer(id.into())).unwrap();
    }
    tourn.apply_op(TournOp::PairRound(admin_id)).unwrap();
    tourn
}

fn spoof_data(count: usize) -> Tournament {
    use TournOp::*;
    let mut t = Tournament::from_preset("Test".into(), TournamentPreset::Swiss, "Pioneer".into());
    let account = spoof_account();
    let admin = Admin {
        name: account.user_name,
        id: (*account.user_id).into(),
    };
    let admin_id = admin.id;
    t.admins.insert(admin_id, admin);
    for _ in 0..count {
        let _ = t.apply_op(RegisterPlayer(spoof_account()));
    }

    let _ = t.apply_op(UpdateTournSetting(
        admin_id,
        TournamentSetting::PairingSetting(PairingSetting::MatchSize(4)),
    ));
    t
}

fn spoof_account() -> SquireAccount {
    let id: UserAccountId = Uuid::new_v4().into();
    SquireAccount {
        user_name: id.to_string(),
        display_name: id.to_string(),
        gamer_tags: HashMap::new(),
        user_id: id,
        permissions: SharingPermissions::Everything,
    }
}
