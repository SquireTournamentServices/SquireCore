#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// A struct that contains generic settings info
struct Settings;

struct String;

template<typename T = void>
struct Vec;

/// A struct used to communicate the application of a Settings objects
struct SettingsResult {
  Settings accepted;
  Settings rejected;
  Settings errored;
};

extern "C" {

/// Returns a new settings objects whose settings are a subset of this Settings's settings. The
/// give iterator defines the keys for the subset of settings
Settings collect_c(const Settings *self, Vec<String> iter);

/// Does what collect does, but removes the elements instead of cloning them
Settings divide_c(Settings *self, Vec<String> iter);

/// Checks if there were any "bad" settings
bool was_success_c(const SettingsResult *self);

} // extern "C"
