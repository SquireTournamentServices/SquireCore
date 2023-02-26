use crate::{
    model::{
        identifiers::{PlayerIdentifier, RoundIdentifier},
        players::Player,
        rounds::Round,
        tournament::TournamentId,
    },
    tournaments::TournamentManager,
};

pub trait ClientState {
    fn query_tournament<Q, R>(&self, id: &TournamentId, query: Q) -> Option<R>
    where
        Q: FnOnce(&TournamentManager) -> R;

    fn import_tournament(&mut self, tourn: TournamentManager);

    fn query_player<Q, R>(
        &self,
        t_id: &TournamentId,
        p_ident: &PlayerIdentifier,
        query: Q,
    ) -> Option<Option<R>>
    where
        Q: FnOnce(&Player) -> R,
    {
        self.query_tournament(t_id, |t| t.get_player(p_ident).ok().map(query))
    }

    fn query_round<Q, R>(
        &self,
        t_id: &TournamentId,
        r_ident: &RoundIdentifier,
        query: Q,
    ) -> Option<Option<R>>
    where
        Q: FnOnce(&Round) -> R,
    {
        self.query_tournament(t_id, |t| t.get_round(r_ident).ok().map(query))
    }
}
