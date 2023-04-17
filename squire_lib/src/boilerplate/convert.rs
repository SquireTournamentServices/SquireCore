//! Boilerplate implementations for enum wrappers. Implements `From` for various types that are
//! wrapped by an enum.

use crate::settings::{
    CommonPairingSetting, CommonScoringSetting, FluidPairingSetting, GeneralSetting,
    PairingSetting, PairingStyleSetting, ScoringSetting, ScoringStyleSetting,
    StandardScoringSetting, SwissPairingSetting, TournamentSetting,
};

/* --------- Convert sub-settings to a `TournamentSetting` --------- */

impl From<GeneralSetting> for TournamentSetting {
    fn from(setting: GeneralSetting) -> Self {
        Self::GeneralSetting(setting)
    }
}

impl From<PairingSetting> for TournamentSetting {
    fn from(value: PairingSetting) -> Self {
        Self::PairingSetting(value)
    }
}

impl From<CommonPairingSetting> for TournamentSetting {
    fn from(setting: CommonPairingSetting) -> Self {
        Self::PairingSetting(setting.into())
    }
}

impl From<PairingStyleSetting> for TournamentSetting {
    fn from(setting: PairingStyleSetting) -> Self {
        Self::PairingSetting(setting.into())
    }
}

impl From<SwissPairingSetting> for TournamentSetting {
    fn from(setting: SwissPairingSetting) -> Self {
        Self::PairingSetting(PairingSetting::Style(setting.into()))
    }
}

impl From<FluidPairingSetting> for TournamentSetting {
    fn from(setting: FluidPairingSetting) -> Self {
        Self::PairingSetting(PairingSetting::Style(setting.into()))
    }
}

impl From<ScoringSetting> for TournamentSetting {
    fn from(setting: ScoringSetting) -> Self {
        Self::ScoringSetting(setting)
    }
}

impl From<CommonScoringSetting> for TournamentSetting {
    fn from(setting: CommonScoringSetting) -> Self {
        Self::ScoringSetting(setting.into())
    }
}

impl From<ScoringStyleSetting> for TournamentSetting {
    fn from(setting: ScoringStyleSetting) -> Self {
        Self::ScoringSetting(setting.into())
    }
}

impl From<StandardScoringSetting> for TournamentSetting {
    fn from(setting: StandardScoringSetting) -> Self {
        Self::ScoringSetting(ScoringSetting::Style(setting.into()))
    }
}

/* --------- Convert sub-settings to a `PairingSetting` --------- */

impl From<CommonPairingSetting> for PairingSetting {
    fn from(setting: CommonPairingSetting) -> Self {
        Self::Common(setting)
    }
}

impl From<PairingStyleSetting> for PairingSetting {
    fn from(setting: PairingStyleSetting) -> Self {
        Self::Style(setting)
    }
}

impl From<SwissPairingSetting> for PairingSetting {
    fn from(setting: SwissPairingSetting) -> Self {
        Self::Style(setting.into())
    }
}

impl From<FluidPairingSetting> for PairingSetting {
    fn from(setting: FluidPairingSetting) -> Self {
        Self::Style(setting.into())
    }
}

/* --------- Convert sub-settings to a `PairingStyleSetting` --------- */

impl From<SwissPairingSetting> for PairingStyleSetting {
    fn from(setting: SwissPairingSetting) -> Self {
        Self::Swiss(setting)
    }
}

impl From<FluidPairingSetting> for PairingStyleSetting {
    fn from(setting: FluidPairingSetting) -> Self {
        Self::Fluid(setting)
    }
}

/* --------- Convert sub-settings to a `ScoringSetting` --------- */

impl From<CommonScoringSetting> for ScoringSetting {
    fn from(setting: CommonScoringSetting) -> Self {
        Self::Common(setting)
    }
}

impl From<ScoringStyleSetting> for ScoringSetting {
    fn from(setting: ScoringStyleSetting) -> Self {
        Self::Style(setting)
    }
}

/* --------- Convert sub-settings to a `ScoringStyleSetting` --------- */

impl From<StandardScoringSetting> for ScoringStyleSetting {
    fn from(other: StandardScoringSetting) -> Self {
        Self::Standard(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::{StandardScoringSetting, TournamentSetting};

    fn subsetting_to_tourn_setting<F, T, S>(f: F) -> TournamentSetting
    where
        T: Default,
        F: Fn(T) -> S,
        TournamentSetting: From<S>,
    {
        f(Default::default()).into()
    }

    #[test]
    fn standard_scoring_setting_to_tourn_setting() {
        use StandardScoringSetting::*;
        let _ = subsetting_to_tourn_setting(MatchWinPoints);
        let _ = subsetting_to_tourn_setting(MatchDrawPoints);
        let _ = subsetting_to_tourn_setting(MatchLossPoints);
        let _ = subsetting_to_tourn_setting(GameWinPoints);
        let _ = subsetting_to_tourn_setting(GameDrawPoints);
        let _ = subsetting_to_tourn_setting(GameLossPoints);
        let _ = subsetting_to_tourn_setting(ByePoints);
        let _ = subsetting_to_tourn_setting(IncludeByes);
        let _ = subsetting_to_tourn_setting(IncludeMatchPoints);
        let _ = subsetting_to_tourn_setting(IncludeGamePoints);
        let _ = subsetting_to_tourn_setting(IncludeMwp);
        let _ = subsetting_to_tourn_setting(IncludeGwp);
        let _ = subsetting_to_tourn_setting(IncludeOppMwp);
        let _ = subsetting_to_tourn_setting(IncludeOppGwp);
    }

    #[test]
    fn fluid_pairing_setting_to_tourn_setting() {}
}
