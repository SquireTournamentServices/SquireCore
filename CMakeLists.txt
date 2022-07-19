set(TARGET_DIR "debug")
if(ENABLE_LTO)
  set(RUST_FLAGS "-Clinker-plugin-lto" "-Clinker=clang-13"
                 "-Clink-arg=-fuse-ld=lld")
  message(STATUS "LTO ENABLED")
endif()

if(WIN32)
  set(EXT "lib")
else()
  set(EXT "a")
endif()

set(RUST_PART_CXX "${CMAKE_CURRENT_BINARY_DIR}/squire_core.${EXT}")
add_library(squire_core STATIC ${RUST_PART_CXX})
message(
  STATUS
    "Building rust with options: ${TARGET_DIR}, ${RUST_FLAGS}, linking .${EXT}")
if(UNIX)
  add_custom_command(
    OUTPUT ${RUST_PART_CXX}
    COMMAND TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} RUSTFLAGS="${RUST_FLAGS}"
            bash "ffi_build.sh"
    COMMAND cp "${CMAKE_CURRENT_SOURCE_DIR}/target/debug/libsquire_lib.${EXT}"
            "${RUST_PART_CXX}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
elseif()
  add_custom_command(
    OUTPUT ${RUST_PART_CXX}
    COMMAND cmd TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} RUSTFLAGS="${RUST_FLAGS}"
            bash "ffi_build.sh"
    COMMAND cp "${CMAKE_CURRENT_SOURCE_DIR}/target/debug/libsquire_lib.${EXT}"
            "${RUST_PART_CXX}"
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})
endif()

target_link_libraries(squire_core pthread dl ${RUST_PART_CXX})

add_test(
  NAME squire_core_test
  COMMAND cargo test
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

message(STATUS "Squire Core FFI is being used")
