use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
    operations::{AdminOp, TournOp},
    players::PlayerId,
    rounds::RoundId,
    tournament::Tournament,
};
use yew::prelude::*;

use super::{PlayerView, PlayerViewMessage};
use crate::{
    tournament::model::{PlayerProfile, RoundProfile},
    CLIENT,
};

/// The set of data needed by the UI to display a deck. Should be capable of rendering itself in
/// HTML.
///
/// NOTE: Under construction
#[derive(Debug, PartialEq, Clone)]
pub struct DeckProfile {
    name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubviewProfile {
    Round(RoundProfile),
    Deck(DeckProfile),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SubviewInfo {
    Round(RoundId),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectedPlayerMessage {
    PlayerSelected(PlayerId),
    SubviewSelected(SubviewInfo),
    /// Optional because the lookup "may" fail
    PlayerQueryReady(Option<PlayerProfile>),
    /// Optional because the lookup "may" fail
    SubviewQueryReady(Option<SubviewProfile>),
    DropPlayer(PlayerId),
}

pub struct SelectedPlayer {
    pub process: Callback<SelectedPlayerMessage>,
    pub id: TournamentId,
    admin_id: AdminId,
    player: Option<PlayerProfile>,
    subview: Option<SubviewProfile>,
}

impl SelectedPlayer {
    pub fn new(
        process: Callback<SelectedPlayerMessage>,
        id: TournamentId,
        admin_id: AdminId,
    ) -> Self {
        Self {
            process,
            id,
            admin_id,
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
                                    .map(|p| PlayerProfile::new(p, t))
                            })
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
            SelectedPlayerMessage::DropPlayer(pid) => {
                CLIENT.get().unwrap().update_tourn(
                    self.id,
                    TournOp::AdminOp(self.admin_id.clone().into(), AdminOp::AdminDropPlayer(pid)),
                );
                false
            }
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
                    <div class="col"> { self.player.as_ref().map(|p| p.view(self.process.clone())).unwrap_or_default() }</div>
                    <div class="col">{ self.subview() }</div>
                </div>
            </div>
        }
    }
}

impl DeckProfile {
    fn view(&self) -> Html {
        html! { <h4>{ "Not implemented yet... sorry" }</h4> }
    }
}

impl SubviewProfile {
    fn matches(&self, info: &SubviewInfo) -> bool {
        match (self, info) {
            (SubviewProfile::Round(rnd), SubviewInfo::Round(id)) => rnd.id == *id,
            // (SubviewProfile::Deck(deck), SubviewInfo::Deck(_, name)) => &deck.name == name,
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
                .map(|rnd| RoundProfile::new(tourn, rnd).into()),
            /*
            SubviewInfo::Deck(p_id, name) => tourn
                .player_reg
                .players
                .get(&p_id)?
                .decks
                .get(&name)
                .map(|deck| DeckProfile::new(deck).into()),
            */
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
