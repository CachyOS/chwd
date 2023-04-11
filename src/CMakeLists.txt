##
## Target
##
add_executable(mhwd-bin
   enums.hpp
   config.cpp config.hpp
   data.cpp data.hpp
   console_writer.cpp console_writer.hpp
   transaction.hpp
   device.hpp
   mhwd.cpp mhwd.hpp
   main.cpp
   )

include_directories(${CMAKE_SOURCE_DIR}/libmhwd)

target_link_libraries(mhwd-bin PRIVATE project_warnings project_options mhwd::mhwd chwd-cxxbridge fmt::fmt hd::hd x86emu::x86emu uuid)
if(CMAKE_CXX_COMPILER_ID MATCHES ".*Clang")
   target_link_libraries(mhwd-bin PRIVATE range-v3::range-v3)
endif()
set_target_properties(mhwd-bin PROPERTIES OUTPUT_NAME mhwd)

option(ENABLE_UNITY "Enable Unity builds of projects" OFF)
if(ENABLE_UNITY)
   # Add for any project you want to apply unity builds for
   set_target_properties(mhwd-bin PROPERTIES UNITY_BUILD ON)
endif()

install(
   TARGETS mhwd-bin
   RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
