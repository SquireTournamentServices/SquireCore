use crate::settings::{
    FluidPairingsSetting, PairingSetting, ScoringSetting, StandardScoringSetting,
    SwissPairingsSetting, TournamentSetting,
};

impl Into<TournamentSetting> for PairingSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::PairingSetting(self)
    }
}

impl Into<TournamentSetting> for ScoringSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::ScoringSetting(self)
    }
}

impl Into<PairingSetting> for SwissPairingsSetting {
    fn into(self) -> PairingSetting {
        PairingSetting::Swiss(self)
    }
}

impl Into<PairingSetting> for FluidPairingsSetting {
    fn into(self) -> PairingSetting {
        PairingSetting::Fluid(self)
    }
}

impl Into<ScoringSetting> for StandardScoringSetting {
    fn into(self) -> ScoringSetting {
        ScoringSetting::Standard(self)
    }
}

impl Into<TournamentSetting> for SwissPairingsSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::PairingSetting(self.into())
    }
}

impl Into<TournamentSetting> for FluidPairingsSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::PairingSetting(self.into())
    }
}

impl Into<TournamentSetting> for StandardScoringSetting {
    fn into(self) -> TournamentSetting {
        TournamentSetting::ScoringSetting(self.into())
    }
}
