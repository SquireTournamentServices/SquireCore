#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

template<typename K = void, typename V = void, typename Hasher = void>
struct HashMap;

struct String;

/// A struct that contains generic settings info
struct Settings {
  HashMap<String, String> settings;
};

/// A struct used to communicate the application of a Settings objects
struct SettingsResult {
  Settings accepted;
  Settings rejected;
  Settings errored;
};

extern "C" {

/// Checks if there were any "bad" settings
bool was_success(const SettingsResult *self);

} // extern "C"
