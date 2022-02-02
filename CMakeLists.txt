cmake_minimum_required(VERSION 3.15)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

##
## PROJECT
## name and version
##
project(mhwd
        VERSION 0.6.5
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
include(FetchContent)

FetchContent_Declare(fmt
  GIT_REPOSITORY "https://github.com/fmtlib/fmt.git"
  GIT_TAG "ae1aaaee5f5f36ce5303bb140d51b6db21e1af4e"
)
FetchContent_Declare(hwinfo
  GIT_REPOSITORY "https://github.com/vnepogodin/hwinfo.git"
  GIT_TAG "master"
)
FetchContent_MakeAvailable(fmt hwinfo)
if(CMAKE_CXX_COMPILER_ID MATCHES ".*Clang")
   FetchContent_Declare(range-v3
     GIT_REPOSITORY "https://github.com/ericniebler/range-v3.git"
     GIT_TAG "9aa032ccd0eb4bd77f58e5b181168f1038c222c6"
   )
   FetchContent_MakeAvailable(range-v3)
endif()

##
## CONFIGURATION
##
set(EXECUTABLE_OUTPUT_PATH ${CMAKE_CURRENT_SOURCE_DIR}/bin)
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -flto")

if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
   set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -fwhole-program")
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

add_subdirectory(libmhwd)
add_subdirectory(src)
add_subdirectory(scripts)
