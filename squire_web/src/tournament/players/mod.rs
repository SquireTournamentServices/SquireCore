use squire_sdk::model::{
    identifiers::{AdminId, TournamentId},
    operations::OpResult,
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

use super::spawn_update_listener;

#[derive(Debug, PartialEq, Properties)]
pub struct PlayerViewProps {
    pub id: TournamentId,
    pub admin_id: AdminId,
    pub send_op_result: Callback<OpResult>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlayerViewMessage {
    FilterInput(PlayerFilterInputMessage),
    PlayerScroll(PlayerScrollMessage),
    SelectedPlayer(SelectedPlayerMessage),
    ReQuery,
}

pub struct PlayerView {
    pub id: TournamentId,
    pub admin_id: AdminId,
    input: PlayerFilterInput,
    scroll: PlayerScroll,
    selected: SelectedPlayer,
}

impl Component for PlayerView {
    type Message = PlayerViewMessage;
    type Properties = PlayerViewProps;

    fn create(ctx: &Context<Self>) -> Self {
        spawn_update_listener(ctx, PlayerViewMessage::ReQuery);
        let id = ctx.props().id;
        let admin_id = ctx.props().admin_id;
        let send_op_result = ctx.props().send_op_result.clone();
        Self {
            id,
            admin_id,
            input: PlayerFilterInput::new(
                ctx.link().callback(PlayerViewMessage::FilterInput),
                id,
                admin_id,
                send_op_result,
            ),
            scroll: PlayerScroll::new(ctx, id),
            selected: SelectedPlayer::new(
                ctx.link().callback(PlayerViewMessage::SelectedPlayer),
                id,
                admin_id,
            ),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlayerViewMessage::FilterInput(msg) => self.input.update(msg),
            PlayerViewMessage::SelectedPlayer(msg) => {
                ctx.link().send_message(PlayerViewMessage::ReQuery);
                self.selected.update(ctx, msg)
            }
            PlayerViewMessage::PlayerScroll(msg) => self.scroll.update(msg),
            PlayerViewMessage::ReQuery => {
                spawn_update_listener(ctx, PlayerViewMessage::ReQuery);
                fetch_player_summaries(ctx, self.id);
                false
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
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
