cmake_minimum_required(VERSION 3.16)
project(detersl_ffi CXX)

# Options
option(ENABLE_LTO "Enable LTO flags for Rust (RUSTFLAGS)" OFF)
option(ENABLE_NATIVE_CPU "Enable native CPU optimizations for Rust (-C target-cpu=native)" OFF)
# Optional: pass Cargo feature(s) as a space-separated string (e.g. "foo bar")
set(DETERSL_CARGO_FEATURES "" CACHE STRING "Cargo --features to enable for the Rust build")

# Resolve build mode for Cargo
if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CARGO_ARGS build --lib --verbose)
    set(TARGET_SUBDIR "debug")
else ()
    set(CARGO_ARGS build --lib --release --verbose)
    set(TARGET_SUBDIR "release")
endif ()

# If cross-compiling with a Cargo target triple, outputs are nested accordingly
if (DEFINED ENV{CARGO_BUILD_TARGET})
    set(TARGET_DIR "$ENV{CARGO_BUILD_TARGET}/${TARGET_SUBDIR}")
else()
    set(TARGET_DIR "${TARGET_SUBDIR}")
endif()

# Optional Rust flags (tune for your toolchain if you really need to override linker)
set(RUST_FLAGS "")
if (ENABLE_LTO)
    set(RUST_FLAGS "-Clinker-plugin-lto -Clinker=clang -Clink-arg=-fuse-ld=gold")
endif()

if (ENABLE_NATIVE_CPU)
    set(RUST_FLAGS "${RUST_FLAGS} -C target-cpu=native")
endif()

# Names from Cargo.toml
set(CARGO_PACKAGE deterSL_runtime)  # [package].name
set(RUST_LIB_BASENAME detersl)      # [lib].name

# Cargo command and optional --features
set(CARGO_CMD cargo ${CARGO_ARGS} -p ${CARGO_PACKAGE})
if (DETERSL_CARGO_FEATURES)
    list(APPEND CARGO_CMD --features ${DETERSL_CARGO_FEATURES})
endif()

# Where Cargo will place artifacts when we force CARGO_TARGET_DIR to the CMake build dir
set(RUST_STATIC_LIB "${CMAKE_CURRENT_BINARY_DIR}/${TARGET_DIR}/lib${RUST_LIB_BASENAME}.a")

# cxxbridge emits headers under:
#   ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/include/<crate-name>/src/ffi.rs.h
# and the runtime header at:
#   ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/include/rust/cxx.h
set(CXXBRIDGE_INCLUDE_ROOT "${CMAKE_CURRENT_BINARY_DIR}/cxxbridge/include")
set(GENERATED_HEADER "${CXXBRIDGE_INCLUDE_ROOT}/${CARGO_PACKAGE}/src/ffi.rs.h")

# Build the Rust static library and generate bridge headers
add_custom_command(
    OUTPUT ${RUST_STATIC_LIB} ${GENERATED_HEADER}
    COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${CMAKE_CURRENT_BINARY_DIR}
            RUSTFLAGS=${RUST_FLAGS}
            ${CARGO_CMD}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
    DEPENDS
        ${CMAKE_CURRENT_SOURCE_DIR}/build.rs
        ${CMAKE_CURRENT_SOURCE_DIR}/src/lib.rs
        ${CMAKE_CURRENT_SOURCE_DIR}/src/ffi.rs
    COMMENT "Building Rust crate (${CARGO_PACKAGE}) and generating cxxbridge headers"
    VERBATIM
)

# Phony target representing the Rust build products
add_custom_target(detersl_rust ALL
    DEPENDS ${RUST_STATIC_LIB} ${GENERATED_HEADER}
)

# Provide an INTERFACE library your C++ code can link against
# - Carries include dirs (so consumers can #include "<deterSL_runtime/src/ffi.rs.h>" and "rust/cxx.h")
# - Carries link to the Rust static lib and pthread/dl on Linux
add_library(detersl_ffi INTERFACE)
add_dependencies(detersl_ffi detersl_rust)

target_include_directories(detersl_ffi INTERFACE
    ${CXXBRIDGE_INCLUDE_ROOT}
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Linux-only linking
target_link_libraries(detersl_ffi INTERFACE ${RUST_STATIC_LIB} pthread dl)

# Optional nice alias for consumers
add_library(detersl::ffi ALIAS detersl_ffi)
