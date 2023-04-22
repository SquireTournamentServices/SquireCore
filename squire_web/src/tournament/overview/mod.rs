use squire_sdk::model::tournament::TournamentStatus;
use yew::prelude::*;

use squire_sdk::model::{players::PlayerStatus, rounds::RoundStatus};
use squire_sdk::tournaments::{TournamentId, TournamentManager};

use crate::CLIENT;

#[derive(Debug, Properties, PartialEq, Eq)]
pub struct OverviewProps {
    pub id: TournamentId,
}

pub struct TournOverview {
    pub id: TournamentId,
}

impl Component for TournOverview {
    type Message = ();
    type Properties = OverviewProps;

    fn create(ctx: &Context<Self>) -> Self {
        TournOverview { id: ctx.props().id }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let report = CLIENT
            .get()
            .unwrap()
            .query_tourn(self.id, OverviewReport::new)
            .process()
            .unwrap_or_default();
        report.view()
    }
}

#[derive(Debug, Default)]
struct OverviewReport {
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

impl OverviewReport {
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
            tourn.player_reg.players.values().fold((0, 0), |mut acc, p| {
                match p.status {
                    PlayerStatus::Registered => acc.0 += 1,
                    PlayerStatus::Dropped => acc.1 += 1,
                    _ => {}
                }
                acc
            });
        let name = tourn.name.clone();
        let format = tourn.settings.format.clone();
        let status = tourn.status;
        let judge_count = tourn.judges.len();
        let admin_count = tourn.admins.len();
        Self { name, format, status, reg_plyrs, dropped_plyrs, active_rnds, cert_rnds, judge_count, admin_count }
    }

    fn view(self) -> Html {
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

                        if dropped_plyrs > 0 {
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
