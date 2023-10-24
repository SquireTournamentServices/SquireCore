use squire_sdk::{
    model::{
        identifiers::TournamentId, players::PlayerStatus, rounds::RoundStatus,
        tournament::TournamentStatus,
    },
    sync::TournamentManager,
};
use yew::prelude::*;

use super::viewer_component::{
    TournQuery, TournViewerComponent, TournViewerComponentWrapper, WrapperState,
};

pub enum TournOverviewMessage {}
pub enum TournOverviewQueryMessage {
    OverviewQueryReady(Option<TournamentProfile>), // Optional because the lookup "may" fail
}

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct OverviewProps {}

pub struct TournOverview {
    pub id: TournamentId,
    profile: Option<TournamentProfile>,
}
impl TournViewerComponent for TournOverview {
    type Properties = OverviewProps;
    type InteractionMessage = TournOverviewMessage;
    type QueryMessage = TournOverviewQueryMessage;

    fn v_create(_ctx: &Context<TournViewerComponentWrapper<Self>>, state: &WrapperState) -> Self {
        let id = state.t_id;
        TournOverview { id, profile: None }
    }

    fn load_queried_data(&mut self, msg: Self::QueryMessage, _state: &WrapperState) -> bool {
        match msg {
            TournOverviewQueryMessage::OverviewQueryReady(data) => {
                let digest = self.profile != data || data.is_none();
                self.profile = data;
                digest
            }
        }
    }

    fn v_view(
        &self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> yew::Html {
        match self.profile.as_ref() {
            Some(p) => p.view(),
            None => Html::default(),
        }
    }

    fn query(
        &mut self,
        _ctx: &Context<TournViewerComponentWrapper<Self>>,
        _state: &WrapperState,
    ) -> TournQuery<Self::QueryMessage> {
        let q_func = |tourn: &TournamentManager| {
            let data = TournamentProfile::new(tourn);
            Self::QueryMessage::OverviewQueryReady(Some(data))
        };
        Box::new(q_func)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct TournamentProfile {
    name: String,
    format: String,
    status: TournamentStatus,
    reg_plyrs: usize,
    dropped_plyrs: usize,
    active_rnds: usize,
    cert_rnds: usize,
    judge_count: usize,
    admin_count: usize,
}

impl TournamentProfile {
    fn new(tourn: &TournamentManager) -> Self {
        let (active_rnds, cert_rnds) =
            tourn.round_reg.rounds.values().fold((0, 0), |mut acc, r| {
                match r.status {
                    RoundStatus::Open => acc.0 += 1,
                    RoundStatus::Certified => acc.1 += 1,
                    _ => {}
                }
                acc
            });

        let (reg_plyrs, dropped_plyrs) =
            tourn
                .player_reg
                .players
                .values()
                .fold((0, 0), |mut acc, p| {
                    match p.status {
                        PlayerStatus::Registered => acc.0 += 1,
                        PlayerStatus::Dropped => acc.1 += 1,
                    }
                    acc
                });
        let name = tourn.name.clone();
        let format = tourn.settings.format.clone();
        let status = tourn.status;
        let judge_count = tourn.judges.len();
        let admin_count = tourn.admins.len();
        Self {
            name,
            format,
            status,
            reg_plyrs,
            dropped_plyrs,
            active_rnds,
            cert_rnds,
            judge_count,
            admin_count,
        }
    }

    fn view(&self) -> Html {
        let Self {
            name,
            format,
            status,
            reg_plyrs,
            dropped_plyrs,
            active_rnds,
            cert_rnds,
            judge_count,
            admin_count,
        } = self;
        html! {
            <div class="m-lg-0 m-md-4 my-3">
                <div class="p-5 bg-light rounded-3">
                    <div class="container-fluid p-md-5">
                        <h1 class="display-5 fw-bold">{ format!("Welcome to {name}") }</h1>
                        <hr class="my-4"/>
                        <p>{ format!("Format : {format}") }</p>
                        <p>{ format!("Status : {status}") }</p>
                        <p>{ format!("Registered players : {reg_plyrs}") }</p>

                        if *dropped_plyrs > 0 {
                            <p>{ format!("Dropped players : {dropped_plyrs}") }</p>
                        }

                        <p>{ format!("Active rounds : {active_rnds}") }</p>
                        <p>{ format!("Certified rounds : {cert_rnds}") }</p>
                        <p>{ format!("Number of judges : {judge_count}") }</p>
                        <p>{ format!("Number of admins : {admin_count}") }</p>
                    </div>
                </div>
            </div>
        }
    }
}
