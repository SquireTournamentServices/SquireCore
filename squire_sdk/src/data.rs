use std::{collections::HashMap, time::Duration};

use hashbag::HashBag;
use squire_lib::{
    pairings::PairingSystem,
    players::{Deck, Player, PlayerId, PlayerRegistry, PlayerStatus},
    rounds::{Round, RoundContext, RoundId, RoundRegistry, RoundStatus},
    scoring::ScoringSystem,
    tournament::{Tournament, TournamentId, TournamentStatus},
};

pub struct CompressedTournament {
    pub id: TournamentId,
    pub name: String,
    pub format: String,
    pub min_deck_count: u8,
    pub max_deck_count: u8,
    pub player_reg: CompressedPlayerReg,
    pub round_reg: CompressedRoundReg,
    pub pairing_sys: PairingSystem,
    pub scoring_sys: ScoringSystem,
    pub require_check_in: bool,
    pub require_deck_reg: bool,
    pub status: TournamentStatus,
}

impl From<Tournament> for CompressedTournament {
    fn from(value: Tournament) -> Self {
        let Tournament {
            id,
            name,
            format,
            min_deck_count,
            max_deck_count,
            player_reg,
            round_reg,
            pairing_sys,
            scoring_sys,
            require_check_in,
            require_deck_reg,
            status,
            ..
        } = value;
        Self {
            id,
            name,
            format,
            min_deck_count,
            max_deck_count,
            pairing_sys,
            scoring_sys,
            require_check_in,
            require_deck_reg,
            status,
            player_reg: player_reg.into(),
            round_reg: round_reg.into(),
        }
    }
}

pub struct CompressedPlayerReg {
    pub players: HashMap<PlayerId, CompressedPlayer>,
}

impl From<PlayerRegistry> for CompressedPlayerReg {
    fn from(value: PlayerRegistry) -> Self {
        let PlayerRegistry { players, .. } = value;
        Self {
            players: players.into_iter().map(|(id, p)| (id, p.into())).collect(),
        }
    }
}

pub struct CompressedDeck {
    pub mainboard: HashBag<String>,
    pub sideboard: HashBag<String>,
    pub commanders: HashBag<String>,
}

impl From<Deck> for CompressedDeck {
    fn from(value: Deck) -> Self {
        let Deck {
            mainboard,
            sideboard,
            commanders,
            ..
        } = value;
        Self {
            mainboard: mainboard
                .set_iter()
                .map(|(card, n)| (card.get_name(), n))
                .collect(),
            sideboard: sideboard
                .set_iter()
                .map(|(card, n)| (card.get_name(), n))
                .collect(),
            commanders: commanders
                .set_iter()
                .map(|(card, n)| (card.get_name(), n))
                .collect(),
        }
    }
}

pub struct CompressedPlayer {
    pub id: PlayerId,
    pub name: String,
    pub decks: HashMap<String, CompressedDeck>,
    pub status: PlayerStatus,
}

impl From<Player> for CompressedPlayer {
    fn from(value: Player) -> Self {
        let Player {
            id,
            name,
            decks,
            status,
            ..
        } = value;
        Self {
            id,
            name,
            status,
            decks: decks
                .into_iter()
                .map(|(name, deck)| (name, deck.into()))
                .collect(),
        }
    }
}

pub struct CompressedRoundReg {
    pub rounds: HashMap<RoundId, CompressedRound>,
    pub starting_table: u64,
}

impl From<RoundRegistry> for CompressedRoundReg {
    fn from(value: RoundRegistry) -> Self {
        let RoundRegistry {
            rounds,
            starting_table,
            ..
        } = value;
        Self {
            rounds: rounds.into_iter().map(|(id, r)| (id, r.into())).collect(),
            starting_table,
        }
    }
}

pub struct CompressedRound {
    pub id: RoundId,
    pub match_number: u64,
    pub table_number: u64,
    pub players: Vec<PlayerId>,
    pub status: RoundStatus,
    pub winner: Option<PlayerId>,
    pub results: HashMap<PlayerId, u32>,
    pub draws: u32,
    pub context: RoundContext,
    pub length: Duration,
    pub extension: Duration,
    pub is_bye: bool,
}

impl From<Round> for CompressedRound {
    fn from(value: Round) -> Self {
        let Round {
            id,
            match_number,
            table_number,
            players,
            status,
            winner,
            results,
            draws,
            context,
            length,
            extension,
            is_bye,
            ..
        } = value;
        Self {
            id,
            match_number,
            table_number,
            players,
            status,
            winner,
            results,
            draws,
            context,
            length,
            extension,
            is_bye,
        }
    }
}
