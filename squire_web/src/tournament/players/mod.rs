use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
};
use yew::prelude::*;

pub mod creator;
pub mod input;
pub mod scroll;
pub mod selected;
pub use creator::*;
pub use input::*;
pub use scroll::*;
pub use selected::*;

use super::{viewer_component::{TournViewerComponent, WrapperState, TournViewerComponentWrapper, TournQuery, InteractionResponse, WrapperMessage}};

#[derive(Debug, PartialEq, Properties)]
pub struct PlayerViewProps {
//    pub id: TournamentId,
//    pub admin_id: AdminId,
//    pub send_op_result: Callback<OpResult>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerViewMessage {
    FilterInput(PlayerFilterInputMessage),
    PlayerScroll(PlayerScrollMessage),
    SelectedPlayer(SelectedPlayerMessage),
}
pub enum PlayerViewQueryMessage {
}

pub struct PlayerView {
    pub id: TournamentId,
    pub admin_id: AdminId,
    input: PlayerFilterInput,
    scroll: PlayerScroll,
    selected: SelectedPlayer,
}

impl TournViewerComponent for PlayerView {
    type Properties = PlayerViewProps;
    type InteractionMessage = PlayerViewMessage;
    type QueryMessage = PlayerViewQueryMessage;

    fn v_create(ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        // spawn_update_listener(ctx, PlayerViewMessage::ReQuery);
        let id = state.t_id.clone();
        let admin_id = state.a_id.clone();
        Self {
            id,
            admin_id,
            input: PlayerFilterInput::new(
                ctx.link().callback(|input| WrapperMessage::Interaction(PlayerViewMessage::FilterInput(input))),
                id,
                admin_id,
            ),
            scroll: PlayerScroll::new(ctx, id),
            selected: SelectedPlayer::new(
                ctx.link().callback(|input| WrapperMessage::Interaction(PlayerViewMessage::SelectedPlayer(input))),
                id,
                admin_id,
            ),
        }
    }

    fn query(
        &mut self,
        ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
    ) -> TournQuery<Self::QueryMessage> {
        todo!()
    }

    fn load_queried_data(&mut self, _msg: Self::QueryMessage, _state: &WrapperState) -> bool {
        false
    }

    fn interaction(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _msg: Self::InteractionMessage,
        _state: &WrapperState,
    ) -> InteractionResponse<Self> {
        match _msg {
            PlayerViewMessage::FilterInput(msg) => self.input.update(msg, _state).into(),
            PlayerViewMessage::SelectedPlayer(msg) => {
                //_ctx.link().send_message(PlayerViewMessage::ReQuery);
                self.selected.update(_ctx, msg).into()
            }
            PlayerViewMessage::PlayerScroll(msg) => self.scroll.update(msg).into(),
        }
    }

    fn v_view(
        &self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        state: &WrapperState,
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
