use chrono::Utc;
use itertools::Itertools;
use squire_lib::{
    error::TournamentError,
    operations::TournOp,
    players::PlayerId,
    tournament::{Tournament, TournamentStatus},
};
use squire_tests::spoof_account;
use uuid::Uuid;

#[test]
fn create_round_test() {
    let mut tourn: Tournament = squire_tests::get_seed().into();
    assert_eq!(tourn.pairing_sys.common.match_size, 2);

    fn make_player(tourn: &mut Tournament) -> PlayerId {
        tourn
            .apply_op(Utc::now(), TournOp::RegisterPlayer(spoof_account()))
            .unwrap()
            .assume_register_player()
    }

    let players_too_large = std::iter::repeat_with(|| make_player(&mut tourn))
        .take(4)
        .collect_vec();
    let players_too_small = vec![make_player(&mut tourn)];
    let players_none = vec![];
    let players_correct = std::iter::repeat_with(|| make_player(&mut tourn))
        .take(2)
        .collect_vec();
    let repeated_players = std::iter::repeat(make_player(&mut tourn))
        .take(2)
        .collect_vec();

    tourn.reg_open = false;
    tourn.status = TournamentStatus::Started;

    assert_eq!(
        tourn.create_round(Utc::now(), players_too_large),
        Err(TournamentError::IncorrectMatchSize)
    );
    assert_eq!(
        tourn.create_round(Utc::now(), players_too_small),
        Err(TournamentError::IncorrectMatchSize)
    );
    assert_eq!(
        tourn.create_round(Utc::now(), players_none),
        Err(TournamentError::IncorrectMatchSize)
    );
    assert!(tourn.create_round(Utc::now(), players_correct).is_ok());

    let unregistered_players = std::iter::repeat_with(|| PlayerId::new(Uuid::new_v4()))
        .take(2)
        .collect_vec();
    assert_eq!(
        tourn.create_round(Utc::now(), unregistered_players),
        Err(TournamentError::PlayerNotFound)
    );

    assert_eq!(
        tourn.create_round(Utc::now(), repeated_players),
        Err(TournamentError::RepeatedPlayerInMatch)
    );
}
