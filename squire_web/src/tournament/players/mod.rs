use squire_sdk::{
    model::{error::TournamentError, identifiers::TournamentId},
    sync::TournamentManager,
};
use yew::prelude::*;

pub mod input;
pub mod scroll;
pub mod selected;
pub use input::*;
pub use scroll::*;
pub use selected::*;

use super::{
    model::PlayerProfile, InteractionResponse, TournQuery, TournViewerComponent,
    TournViewerComponentWrapper, WrapperMessage, WrapperState,
};

#[derive(Debug, PartialEq, Properties)]
pub struct PlayerViewProps {}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerViewMessage {
    FilterInput(PlayerFilterInputMessage),
    SelectedPlayer(SelectedPlayerMessage),
}
pub enum PlayerViewQueryMessage {
    AllData(PlayerViewQueryData),
    SelectedPlayer(Result<PlayerProfile, TournamentError>),
    SelectedSubview(Option<SubviewProfile>),
}
pub struct PlayerViewQueryData {
    players: Vec<PlayerSummary>,
}

pub struct PlayerView {
    pub id: TournamentId,
    input: PlayerFilterInput,
    scroll: PlayerScroll,
    selected: SelectedPlayer,
}

impl TournViewerComponent for PlayerView {
    type Properties = PlayerViewProps;
    type InteractionMessage = PlayerViewMessage;
    type QueryMessage = PlayerViewQueryMessage;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        let id = state.t_id;
        Self {
            id,
            input: PlayerFilterInput::new(
                ctx.link().callback(|input| {
                    WrapperMessage::Interaction(PlayerViewMessage::FilterInput(input))
                }),
                id,
            ),
            scroll: PlayerScroll::new(
                ctx.link().callback(|input| {
                    WrapperMessage::Interaction(PlayerViewMessage::SelectedPlayer(
                        SelectedPlayerMessage::PlayerSelected(input),
                    ))
                }),
                id,
            ),
            selected: SelectedPlayer::new(
                ctx.link().callback(|input| {
                    WrapperMessage::Interaction(PlayerViewMessage::SelectedPlayer(input))
                }),
                id,
            ),
        }
    }

    fn query(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> TournQuery<Self::QueryMessage> {
        let q_func = |tourn: &TournamentManager| {
            let mut players: Vec<PlayerSummary> = tourn
                .player_reg
                .players
                .values()
                .map(PlayerSummary::new)
                .collect();
            players.sort_by_cached_key(|p| p.name.clone());
            players.sort_by_cached_key(|p| p.status);
            Self::QueryMessage::AllData(PlayerViewQueryData { players })
        };
        Box::new(q_func)
    }

    fn load_queried_data(&mut self, msg: Self::QueryMessage, state: &WrapperState) -> bool {
        match msg {
            PlayerViewQueryMessage::AllData(data) => self
                .scroll
                .update(PlayerScrollMessage::ScrollQueryReady(data.players)),
            PlayerViewQueryMessage::SelectedPlayer(result) => {
                self.selected
                    .update(SelectedPlayerMessage::PlayerQueryReady(result.ok()), state);
                true
            }
            PlayerViewQueryMessage::SelectedSubview(profile) => {
                self.selected
                    .update(SelectedPlayerMessage::SubviewQueryReady(profile), state);
                true
            }
        }
    }

    fn interaction(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _msg: Self::InteractionMessage,
        _state: &WrapperState,
    ) -> InteractionResponse<Self> {
        match _msg {
            PlayerViewMessage::FilterInput(msg) => self.input.update(msg, _state),
            PlayerViewMessage::SelectedPlayer(msg) => self.selected.update(msg, _state),
        }
    }

    fn v_view(
        &self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> yew::Html {
        let report = self.input.get_report();
        html! {
            <div>
                <div>{ self.input.view() }</div>
                <div class="d-flex flex-row my-4">
                    <div>
                        <div class="overflow-auto player-scroll-box px-4">
                            { self.scroll.view(report) }
                        </div>
                    </div>
                    <div>
                        { self.selected.view() }
                    </div>
                </div>
            </div>
        }
    }
}

impl From<PlayerFilterInputMessage> for PlayerViewMessage {
    fn from(msg: PlayerFilterInputMessage) -> Self {
        Self::FilterInput(msg)
    }
}

impl From<SelectedPlayerMessage> for PlayerViewMessage {
    fn from(msg: SelectedPlayerMessage) -> Self {
        Self::SelectedPlayer(msg)
    }
}
