use squire_sdk::{
    model::{
        identifiers::{PlayerIdentifier, TypeId},
        rounds::RoundId,
    },
    players::{Deck, Player, PlayerId, Round},
    tournaments::{Tournament, TournamentId, TournamentManager},
};
use yew::prelude::*;

use crate::{
    tournament::rounds::{round_info_display, RoundSummary},
    CLIENT,
};

use super::{PlayerView, PlayerViewMessage};

#[derive(Debug, PartialEq, Clone)]
pub struct PlayerProfile {
    id: PlayerId,
    name: String,
    gamer_tag: Option<String>,
    can_play: bool,
    rounds: Vec<RoundSummary>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DeckProfile {
    name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoundProfile {
    pub id: RoundId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubviewProfile {
    Round(RoundProfile),
    Deck(DeckProfile),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubviewInfo {
    Round(RoundId),
    Deck(PlayerId, String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectedPlayerMessage {
    PlayerSelected(PlayerId),
    SubviewSelected(SubviewInfo),
    PlayerQueryReady(Option<PlayerProfile>), // Optional because the lookup "may" fail
    SubviewQueryReady(Option<SubviewProfile>), // Optional because the lookup "may" fail
}

pub struct SelectedPlayer {
    pub process: Callback<SelectedPlayerMessage>,
    pub id: TournamentId,
    player: Option<PlayerProfile>,
    subview: Option<SubviewProfile>,
}

impl SelectedPlayer {
    pub fn new(process: Callback<SelectedPlayerMessage>, id: TournamentId) -> Self {
        Self {
            process,
            id,
            player: None,
            subview: None,
        }
    }

    pub fn load_player_data(&mut self, data: PlayerProfile) -> bool {
        self.player = Some(data);
        true
    }

    pub fn load_subview_data(&mut self, data: SubviewProfile) -> bool {
        self.subview = Some(data);
        true
    }

    // TODO: This should probably be generic over the context's type. Roughly, where T:
    // Component<Message = M>, M: From<... something>
    pub fn update(&mut self, ctx: &Context<PlayerView>, msg: SelectedPlayerMessage) -> bool {
        match msg {
            SelectedPlayerMessage::PlayerSelected(p_id) => {
                if self.player.as_ref().map(|p| p.id != p_id).unwrap_or(true) {
                    let id = self.id;
                    ctx.link().send_future(async move {
                        let data = CLIENT
                            .get()
                            .unwrap()
                            .query_tourn(id, move |t| {
                                t.tourn()
                                    .player_reg
                                    .get_player(&p_id)
                                    .map(PlayerProfile::new)
                            })
                            .process()
                            .await
                            .transpose()
                            .ok()
                            .flatten();
                        PlayerViewMessage::SelectedPlayer(SelectedPlayerMessage::PlayerQueryReady(
                            data,
                        ))
                    });
                }
                false
            }
            SelectedPlayerMessage::SubviewSelected(info) => {
                if self
                    .subview
                    .as_ref()
                    .map(|sv| !sv.matches(&info))
                    .unwrap_or(true)
                {
                    let id = self.id;
                    ctx.link().send_future(async move {
                        let data = CLIENT
                            .get()
                            .unwrap()
                            .query_tourn(id, |t| info.to_profile(t.tourn()))
                            .process()
                            .await
                            .flatten();
                        PlayerViewMessage::SelectedPlayer(SelectedPlayerMessage::SubviewQueryReady(
                            data,
                        ))
                    })
                }
                false
            }
            SelectedPlayerMessage::PlayerQueryReady(Some(data)) => self.load_player_data(data),
            SelectedPlayerMessage::SubviewQueryReady(Some(data)) => self.load_subview_data(data),
            SelectedPlayerMessage::PlayerQueryReady(None)
            | SelectedPlayerMessage::SubviewQueryReady(None) => false,
        }
    }

    fn subview(&self) -> Html {
        match &self.subview {
            None => {
                html! { <h3>{" No info selected "}</h3> }
            }
            Some(SubviewProfile::Round(rnd)) => rnd.view(),
            Some(SubviewProfile::Deck(deck)) => deck.view(),
        }
    }

    pub fn view(&self) -> Html {
        html! {
            <div class="m-2">
                <div class="row">
                    <div class="col"> { self.player.as_ref().map(|p| p.view()).unwrap_or_default() }</div>
                    <div class="col">{ self.subview() }</div>
                </div>
            </div>
        }
    }
}

impl PlayerProfile {
    pub fn new(plyr: &Player) -> Self {
        Self {
            id: plyr.id,
            name: plyr.name.clone(),
            gamer_tag: plyr.game_name.clone(),
            can_play: plyr.can_play(),
            rounds: Vec::new(), // TODO: This needs to be the player's list of rounds
        }
    }

    pub fn view(&self) -> Html {
        html! {
            <>
                <>
                    <>
                        <h4>{ self.name.as_str() }</h4>
                        <p>{ format!("Gamertag : {}", self.gamer_tag.clone().unwrap_or_default() ) }</p>
                        <p>{ format!("Can play : {}", self.can_play) }</p>
                        <p>{ format!("Rounds : {}", self.rounds.len()) }</p>
                    </>
                </>
                <ul>
                {
                    html! { <h4> { "Player's round view not implemented yet..." } </h4> }
                    /*
                    self.rounds
                    .iter()
                    .map(|r| {
                        let cb = self.process.clone();
                        html! {<li class="sub_option" onclick={ move |_| cb.emit(SubviewInfo::Round(r.id)) }>{ format!("Match {} at table {}", r.match_number, r.table_number) }</li>}
                    })
                    .collect::<Html>()
                    */
                }
                </ul>
            </>
        }
    }
}

impl DeckProfile {
    fn new(deck: &Deck) -> Self {
        Self {
            name: deck.name.clone().unwrap_or_default(),
        }
    }

    fn view(&self) -> Html {
        html! { <h4>{ "Not implemented yet... sorry" }</h4> }
    }
}

impl RoundProfile {
    pub fn new(rnd: &Round) -> Self {
        Self { id: rnd.id }
    }

    pub fn view(&self) -> Html {
        todo!()
    }
}

impl SubviewProfile {
    fn matches(&self, info: &SubviewInfo) -> bool {
        match (self, info) {
            (SubviewProfile::Round(rnd), SubviewInfo::Round(id)) => rnd.id == *id,
            (SubviewProfile::Deck(deck), SubviewInfo::Deck(_, name)) => &deck.name == name,
            _ => false,
        }
    }
}

impl SubviewInfo {
    fn to_profile(self, tourn: &Tournament) -> Option<SubviewProfile> {
        match self {
            SubviewInfo::Round(r_id) => tourn
                .round_reg
                .rounds
                .get(&r_id)
                .map(|rnd| RoundProfile::new(rnd).into()),
            SubviewInfo::Deck(p_id, name) => tourn
                .player_reg
                .players
                .get(&p_id)?
                .decks
                .get(&name)
                .map(|deck| DeckProfile::new(deck).into()),
        }
    }
}

impl From<DeckProfile> for SubviewProfile {
    fn from(deck: DeckProfile) -> Self {
        Self::Deck(deck)
    }
}

impl From<RoundProfile> for SubviewProfile {
    fn from(rnd: RoundProfile) -> Self {
        Self::Round(rnd)
    }
}
