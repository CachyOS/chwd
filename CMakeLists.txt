cmake_minimum_required(VERSION 3.15)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

##
## PROJECT
## name and version
##
project(mhwd
        VERSION 0.7.1
        LANGUAGES C CXX)

##
## INCLUDE
##
include(GNUInstallDirs)
include(StandardProjectSettings)
include(CompilerWarnings)
include(EnableCcache)
include(Linker)
include(StaticAnalyzers)
include(Sanitizers)
include(CPM)

CPMAddPackage(
  NAME fmt
  GITHUB_REPOSITORY fmtlib/fmt
  GIT_TAG 9.1.0
  EXCLUDE_FROM_ALL YES
)

CPMAddPackage(
  NAME Corrosion
  GITHUB_REPOSITORY corrosion-rs/corrosion
  GIT_TAG 14d5ecf8b85e15abc19f24f309df81f5f1d0d7ca
  EXCLUDE_FROM_ALL YES
)

if(CMAKE_CXX_COMPILER_ID MATCHES ".*Clang")
   CPMAddPackage(
     NAME range-v3
     GITHUB_REPOSITORY ericniebler/range-v3
     GIT_TAG 4624c63972c6f2c2871c7b87813c42048ddb80ad
     EXCLUDE_FROM_ALL YES
   )
endif()

##
## CONFIGURATION
##
if(CMAKE_CXX_COMPILER_ID MATCHES ".*Clang")
   set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -flto=thin -fwhole-program-vtables")
   set(CMAKE_EXE_LINKER_FLAGS_RELEASE "${CMAKE_EXE_LINKER_FLAGS_RELEASE} -flto=thin -fwhole-program-vtables")
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
   set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -flto -fwhole-program -fuse-linker-plugin")
endif()

# Link this 'library' to set the c++ standard / compile-time options requested
add_library(project_options INTERFACE)
target_compile_features(project_options INTERFACE cxx_std_20)

# Link this 'library' to use the warnings specified in CompilerWarnings.cmake
add_library(project_warnings INTERFACE)
set_project_warnings(project_warnings)

# Add linker configuration
configure_linker(project_options)

# sanitizer options if supported by compiler
enable_sanitizers(project_options)

option(ENABLE_UNITY "Enable Unity builds of projects" OFF)
if(ENABLE_UNITY)
   # Add for any project you want to apply unity builds for
   set_target_properties(${PROJECT_NAME} PROPERTIES UNITY_BUILD ON)
endif()

include_directories(${CMAKE_SOURCE_DIR}/src)

if(ENABLE_SANITIZER_ADDRESS)
    list(APPEND CARGO_FLAGS "-Z" "build-std")
endif()

corrosion_import_crate(MANIFEST_PATH "chwd-rust/Cargo.toml" FLAGS "${CARGO_FLAGS}")
corrosion_add_cxxbridge(chwd-cxxbridge CRATE chwd-rust MANIFEST_PATH "chwd-rust/Cargo.toml" FILES lib.rs)

add_subdirectory(libmhwd)
add_subdirectory(src)
add_subdirectory(scripts)
