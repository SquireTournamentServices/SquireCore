use yew::prelude::*;

use squire_sdk::{tournaments::Tournament, model::pairings::{PairingStyle, SwissPairings, FluidPairings}};

#[derive(Debug, Default)]
pub struct PairingsSettings {}

impl PairingsSettings {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self, tourn: &Tournament) -> Html {
        let sys = &tourn.pairing_sys;
        html! {
            <div>
                <h2>{ "Pairings Settings:" }</h2>
                <p>{ format!("Match size: {}", sys.match_size) }</p>
                <p>{ format!("Repair tolerance: {}", sys.repair_tolerance) }</p>
                <p>{ format!("Algorithm: {}", sys.alg) }</p>
                { pairings_style_view(&sys.style) }
            </div>
        }
    }
}

fn pairings_style_view(style: &PairingStyle) -> Html {
    match &style {
        PairingStyle::Swiss(style) => swiss_style_view(style),
        PairingStyle::Fluid(style) => fluid_style_view(style),
    }
}

fn swiss_style_view(style: &SwissPairings) -> Html {
    html! {
        <div>
            <h3>{ "Swiss Settings" }</h3>
            <p>{ format!("Do check ins: {}", style.do_check_ins()) }</p>
        </div>
    }
}

fn fluid_style_view(style: &FluidPairings) -> Html {
    html! {}
}
