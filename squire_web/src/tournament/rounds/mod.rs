use chrono::{DateTime, Utc};
use squire_sdk::{
    model::{
        identifiers::RoundIdentifier,
        rounds::{RoundId, RoundStatus},
    },
    tournaments::TournamentId,
};

use yew::prelude::*;

use crate::{utils::TextInput, CLIENT};

pub mod creator;
pub mod input;
pub mod roundresultticker;
pub mod scroll;
pub mod selected;
pub use creator::*;
pub use input::*;
pub use roundresultticker::*;
pub use scroll::*;
pub use selected::*;

#[derive(Debug, PartialEq, Properties)]
pub struct RoundsFilterProps {
    pub id: TournamentId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RoundsFilterMessage {
    RoundSelected(RoundId),
    FilterInput(RoundFilterInputMessage),
    TimerTick,
}

pub struct RoundsView {
    pub id: TournamentId,
    input: RoundFilterInput,
    scroll: RoundScroll,
    selected: Option<SelectedRound>,
}

impl Component for RoundsView {
    type Message = RoundsFilterMessage;
    type Properties = RoundsFilterProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            RoundsFilterMessage::TimerTick
        });
        Self {
            id: ctx.props().id,
            input: RoundFilterInput::new(ctx.link().callback(RoundsFilterMessage::FilterInput)),
            scroll: RoundScroll::new(ctx.link().callback(RoundsFilterMessage::RoundSelected)),
            selected: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RoundsFilterMessage::FilterInput(msg) => {
                if self.input.update(msg) {
                    self.scroll.update(self.input.get_report())
                } else {
                    false
                }
            }
            RoundsFilterMessage::RoundSelected(r_id) => {
                if self.selected.is_none() {
                    self.selected.insert(SelectedRound::new(r_id, self.id));
                }
                self.selected
                    .as_mut()
                    .map(|sr| sr.update(r_id))
                    .unwrap_or_default()
            }
            RoundsFilterMessage::TimerTick => {
                ctx.link().send_future(async move {
                    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
                    RoundsFilterMessage::TimerTick
                });
                self.selected.is_some()
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let report = self.scroll.report.clone();
        let plyrs = self
            .selected
            .as_ref()
            .map(|sr| {
                sr.round_data_buffer
                    .as_ref()
                    .map(|rnd| rnd.players.iter().cloned().collect())
            })
            .flatten();
        match CLIENT
            .get()
            .unwrap()
            .query_tourn(self.id, |tourn| {
                (
                    RoundScrollQuery::new(report, tourn),
                    plyrs.map(|plyrs| SelectedRoundQuery::new(plyrs, tourn)),
                )
            })
            .process()
        {
            Some((scroll_query, selected_query)) => {
                html! {
                    <div>
                        { self.input.view() }
                        <div class="d-flex flex-row my-4">
                            <div>
                                <div class="overflow-auto player-scroll-box">
                                    { self.scroll.view(scroll_query) }
                                </div>
                            </div>
                            <div>
                                {
                                    match (self.selected.as_ref(), selected_query) {
                                        (Some(sr), Some(query)) => sr.view(query),
                                        _ => Html::default(),
                                    }
                                }
                            </div>
                        </div>
                    </div>
                }
            }
            None => Html::default(),
        }
    }
}
