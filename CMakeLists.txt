set(TARGET_DIR "debug")

if(ENABLE_LTO)
    set(RUST_FLAGS "-Clinker-plugin-lto" "-Clinker=clang-13" "-Clink-arg=-fuse-ld=lld")
endif()


set(RUST_PART_CXX "${CMAKE_CURRENT_BINARY_DIR}/squire_core.a")
add_library(squire_core STATIC ${RUST_PART_CXX})
add_custom_command(
    OUTPUT ${RUST_PART_CXX}
    COMMAND CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR} RUSTFLAGS="${RUST_FLAGS}" bash ./build.sh
    COMMAND cp ${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/libsquire_core.a ${RUST_PART_CXX}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

target_link_libraries(squire_core pthread dl ${RUST_PART_CXX})

add_test(NAME squire_core_test
    COMMAND cargo test
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

