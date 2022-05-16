#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class TournamentPreset {
  Swiss,
  Fluid,
};

enum class TournamentStatus {
  Planned,
  Started,
  Frozen,
  Ended,
  Cancelled,
};

struct FluidPairings;

struct Player;

struct PlayerIdentifier;

struct PlayerRegistry;

struct Round;

struct RoundIdentifier;

struct RoundRegistry;

struct StandardScore;

struct StandardScoring;

template<typename S = void>
struct Standings;

struct String;

struct SwissPairings;

template<typename T = void>
struct Vec;

struct TournamentId {
  Uuid _0;
};

struct PairingSystem {
  enum class Tag {
    Swiss,
    Fluid,
  };

  struct Swiss_Body {
    SwissPairings _0;
  };

  struct Fluid_Body {
    FluidPairings _0;
  };

  Tag tag;
  union {
    Swiss_Body swiss;
    Fluid_Body fluid;
  };
};

struct ScoringSystem {
  enum class Tag {
    Standard,
  };

  struct Standard_Body {
    StandardScoring _0;
  };

  Tag tag;
  union {
    Standard_Body standard;
  };
};

struct Tournament {
  TournamentId id;
  String name;
  String format;
  uint8_t game_size;
  uint8_t min_deck_count;
  uint8_t max_deck_count;
  PlayerRegistry player_reg;
  RoundRegistry round_reg;
  PairingSystem pairing_sys;
  ScoringSystem scoring_sys;
  bool reg_open;
  bool require_check_in;
  bool require_deck_reg;
  TournamentStatus status;
};

struct PlayerId {
  Uuid _0;
};

struct RoundResult {
  enum class Tag {
    Wins,
    Draw,
  };

  struct Wins_Body {
    PlayerId _0;
    uint8_t _1;
  };

  struct Draw_Body {

  };

  Tag tag;
  union {
    Wins_Body wins;
    Draw_Body draw;
  };
};

struct SwissPairingsSetting {
  enum class Tag {
    MatchSize,
    DoCheckIns,
  };

  struct MatchSize_Body {
    uint8_t _0;
  };

  struct DoCheckIns_Body {
    bool _0;
  };

  Tag tag;
  union {
    MatchSize_Body match_size;
    DoCheckIns_Body do_check_ins;
  };
};

struct FluidPairingsSetting {
  enum class Tag {
    MatchSize,
  };

  struct MatchSize_Body {
    uint8_t _0;
  };

  Tag tag;
  union {
    MatchSize_Body match_size;
  };
};

struct PairingSetting {
  enum class Tag {
    Swiss,
    Fluid,
  };

  struct Swiss_Body {
    SwissPairingsSetting _0;
  };

  struct Fluid_Body {
    FluidPairingsSetting _0;
  };

  Tag tag;
  union {
    Swiss_Body swiss;
    Fluid_Body fluid;
  };
};

struct StandardScoringSetting {
  enum class Tag {
    MatchWinPoints,
    MatchDrawPoints,
    MatchLossPoints,
    GameWinPoints,
    GameDrawPoints,
    GameLossPoints,
    ByePoints,
    IncludeByes,
    IncludeMatchPoints,
    IncludeGamePoints,
    IncludeMwp,
    IncludeGwp,
    IncludeOppMwp,
    IncludeOppGwp,
  };

  struct MatchWinPoints_Body {
    double _0;
  };

  struct MatchDrawPoints_Body {
    double _0;
  };

  struct MatchLossPoints_Body {
    double _0;
  };

  struct GameWinPoints_Body {
    double _0;
  };

  struct GameDrawPoints_Body {
    double _0;
  };

  struct GameLossPoints_Body {
    double _0;
  };

  struct ByePoints_Body {
    double _0;
  };

  struct IncludeByes_Body {
    bool _0;
  };

  struct IncludeMatchPoints_Body {
    bool _0;
  };

  struct IncludeGamePoints_Body {
    bool _0;
  };

  struct IncludeMwp_Body {
    bool _0;
  };

  struct IncludeGwp_Body {
    bool _0;
  };

  struct IncludeOppMwp_Body {
    bool _0;
  };

  struct IncludeOppGwp_Body {
    bool _0;
  };

  Tag tag;
  union {
    MatchWinPoints_Body match_win_points;
    MatchDrawPoints_Body match_draw_points;
    MatchLossPoints_Body match_loss_points;
    GameWinPoints_Body game_win_points;
    GameDrawPoints_Body game_draw_points;
    GameLossPoints_Body game_loss_points;
    ByePoints_Body bye_points;
    IncludeByes_Body include_byes;
    IncludeMatchPoints_Body include_match_points;
    IncludeGamePoints_Body include_game_points;
    IncludeMwp_Body include_mwp;
    IncludeGwp_Body include_gwp;
    IncludeOppMwp_Body include_opp_mwp;
    IncludeOppGwp_Body include_opp_gwp;
  };
};

struct ScoringSetting {
  enum class Tag {
    Standard,
  };

  struct Standard_Body {
    StandardScoringSetting _0;
  };

  Tag tag;
  union {
    Standard_Body standard;
  };
};

struct TournamentSetting {
  enum class Tag {
    Format,
    MinDeckCount,
    MaxDeckCount,
    RequireCheckIn,
    RequireDeckReg,
    PairingSetting,
    ScoringSetting,
  };

  struct Format_Body {
    String _0;
  };

  struct MinDeckCount_Body {
    uint8_t _0;
  };

  struct MaxDeckCount_Body {
    uint8_t _0;
  };

  struct RequireCheckIn_Body {
    bool _0;
  };

  struct RequireDeckReg_Body {
    bool _0;
  };

  struct PairingSetting_Body {
    PairingSetting _0;
  };

  struct ScoringSetting_Body {
    ScoringSetting _0;
  };

  Tag tag;
  union {
    Format_Body format;
    MinDeckCount_Body min_deck_count;
    MaxDeckCount_Body max_deck_count;
    RequireCheckIn_Body require_check_in;
    RequireDeckReg_Body require_deck_reg;
    PairingSetting_Body pairing_setting;
    ScoringSetting_Body scoring_setting;
  };
};

/// This enum captures all ways in which a tournament can mutate.
struct TournOp {
  enum class Tag {
    UpdateReg,
    Start,
    Freeze,
    Thaw,
    End,
    Cancel,
    CheckIn,
    RegisterPlayer,
    RecordResult,
    ConfirmResult,
    DropPlayer,
    AdminDropPlayer,
    AddDeck,
    RemoveDeck,
    SetGamerTag,
    ReadyPlayer,
    UnReadyPlayer,
    UpdateTournSetting,
    GiveBye,
    CreateRound,
    PairRound,
  };

  struct UpdateReg_Body {
    bool _0;
  };

  struct Start_Body {

  };

  struct Freeze_Body {

  };

  struct Thaw_Body {

  };

  struct End_Body {

  };

  struct Cancel_Body {

  };

  struct CheckIn_Body {
    PlayerIdentifier _0;
  };

  struct RegisterPlayer_Body {
    String _0;
  };

  struct RecordResult_Body {
    RoundIdentifier _0;
    RoundResult _1;
  };

  struct ConfirmResult_Body {
    PlayerIdentifier _0;
  };

  struct DropPlayer_Body {
    PlayerIdentifier _0;
  };

  struct AdminDropPlayer_Body {
    PlayerIdentifier _0;
  };

  struct AddDeck_Body {
    PlayerIdentifier _0;
    String _1;
    Deck _2;
  };

  struct RemoveDeck_Body {
    PlayerIdentifier _0;
    String _1;
  };

  struct SetGamerTag_Body {
    PlayerIdentifier _0;
    String _1;
  };

  struct ReadyPlayer_Body {
    PlayerIdentifier _0;
  };

  struct UnReadyPlayer_Body {
    PlayerIdentifier _0;
  };

  struct UpdateTournSetting_Body {
    TournamentSetting _0;
  };

  struct GiveBye_Body {
    PlayerIdentifier _0;
  };

  struct CreateRound_Body {
    Vec<PlayerIdentifier> _0;
  };

  struct PairRound_Body {

  };

  Tag tag;
  union {
    UpdateReg_Body update_reg;
    Start_Body start;
    Freeze_Body freeze;
    Thaw_Body thaw;
    End_Body end;
    Cancel_Body cancel;
    CheckIn_Body check_in;
    RegisterPlayer_Body register_player;
    RecordResult_Body record_result;
    ConfirmResult_Body confirm_result;
    DropPlayer_Body drop_player;
    AdminDropPlayer_Body admin_drop_player;
    AddDeck_Body add_deck;
    RemoveDeck_Body remove_deck;
    SetGamerTag_Body set_gamer_tag;
    ReadyPlayer_Body ready_player;
    UnReadyPlayer_Body un_ready_player;
    UpdateTournSetting_Body update_tourn_setting;
    GiveBye_Body give_bye;
    CreateRound_Body create_round;
    PairRound_Body pair_round;
  };
};

struct RoundId {
  Uuid _0;
};

extern "C" {

/// Returns 0 if everything is ok.
/// Returns 1 if there is an error with the name conversion
/// Returns 2 if there is an error with the format conversion
uintptr_t from_preset_c(Tournament *expected,
                        char *name_buf,
                        TournamentPreset preset,
                        char *format_buf);

/// Returns 0 if everything is ok.
/// Returns 1 if there is a TournamentError::IncorrectStatus,
/// Returns 2 if there is a TournamentError::PlayerLookup,
/// Returns 3 if there is a TournamentError::RoundLookup,
/// Returns 4 if there is a TournamentError::DeckLookup,
/// Returns 5 if there is a TournamentError::RegClosed,
/// Returns 6 if there is a TournamentError::PlayerNotInRound,
/// Returns 7 if there is a TournamentError::NoActiveRound,
/// Returns 8 if there is a TournamentError::InvalidBye,
/// Returns 9 if there is a TournamentError::ActiveMatches,
/// Returns 10 if there is a TournamentError::PlayerNotCheckedIn,
/// Returns 11 if there is a TournamentError::IncompatiblePairingSystem,
/// Returns 12 if there is a TournamentError::IncompatibleScoringSystem,
uintptr_t apply_op_c(Tournament *self, TournOp op);

/// Returns 0 if everything is ok.
/// Returns 1 if the player could not be found
uintptr_t get_player_c(const Tournament *self,
                       const Player *expected,
                       const PlayerIdentifier *ident);

/// Returns 0 if everything is ok.
/// Returns 1 if the round could not be found
uintptr_t get_round_c(const Tournament *self, const Round *expected, const RoundIdentifier *ident);

/// Returns `0` if the player could be found and `1` if they could be not be found.
uintptr_t get_player_round_c(const Tournament *self,
                             const RoundId *expected,
                             const PlayerIdentifier *ident);

Standings<StandardScore> get_standings_c(const Tournament *self);

bool is_planned_c(const Tournament *self);

bool is_frozen_c(const Tournament *self);

bool is_active_c(const Tournament *self);

bool is_dead_c(const Tournament *self);

} // extern "C"
