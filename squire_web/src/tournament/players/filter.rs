use squire_sdk::{
    client::state::ClientState,
    model::{identifiers::RoundIdentifier, rounds::RoundStatus},
    players::PlayerId,
    tournaments::TournamentId,
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

use super::{
    input::{PlayerFilterInput, PlayerFilterInputMessage, PlayerFilterReport},
    scroll::PlayerScroll,
    selected::SelectedPlayer,
};

#[derive(Debug, PartialEq, Properties)]
pub struct PlayerFilterProps {
    pub id: TournamentId,
}

pub enum PlayerFilterMessage {
    PlayerSelected(PlayerId),
    FilterInput(PlayerFilterInputMessage),
}

pub struct PlayerFilter {
    pub id: TournamentId,
    input: PlayerFilterInput,
    scroll: PlayerScroll,
    selected: SelectedPlayer,
}

impl Component for PlayerFilter {
    type Message = PlayerFilterMessage;
    type Properties = PlayerFilterProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            id: ctx.props().id,
            input: PlayerFilterInput::new(ctx.link().callback(PlayerFilterMessage::FilterInput)),
            scroll: PlayerScroll::new(ctx.link().callback(PlayerFilterMessage::PlayerSelected)),
            selected: SelectedPlayer::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlayerFilterMessage::FilterInput(msg) => {
                if self.input.update(msg) {
                    self.scroll.update(self.input.get_report())
                } else {
                    false
                }
            }
            PlayerFilterMessage::PlayerSelected(p_id) => {
                self.selected.update(Some(p_id))
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        CLIENT
            .get()
            .unwrap()
            .state
            .query_tournament(&self.id, |t| {
                let process = ctx.link().callback(PlayerFilterMessage::FilterInput);
                html! {
                    <div>
                        { self.input.view() }
                        { self.scroll.view(t) }
                        { self.selected.view(t) }
                    </div>
                }
            })
            .unwrap_or_default()
    }
}
