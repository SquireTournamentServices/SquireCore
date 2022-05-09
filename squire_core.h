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

struct RoundId;

struct RoundIdentifier;

struct RoundRegistry;

struct StandardScore;

struct StandardScoring;

template<typename S = void>
struct Standings;

struct String;

struct SwissPairings;

/// This enum captures all ways in which a tournament can mutate.
struct TournOp;

struct TournamentError;

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

extern "C" {

void from_preset_c(Tournament *expected, char *name_buf, TournamentPreset preset, char *format_buf);

void apply_op_c(Tournament *self, const TournamentError *error, TournOp op);

bool is_planned_c(const Tournament *self);

bool is_frozen_c(const Tournament *self);

bool is_active_c(const Tournament *self);

bool is_dead_c(const Tournament *self);

void get_player_c(const Tournament *self, const Player *expected, const PlayerIdentifier *ident);

void get_round_c(const Tournament *self, const Round *expected, const RoundIdentifier *ident);

void get_player_round_c(const Tournament *self,
                        const RoundId *expected,
                        const PlayerIdentifier *ident);

Standings<StandardScore> get_standings_c(const Tournament *self);

} // extern "C"
