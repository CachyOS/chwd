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

target_link_libraries(mhwd-bin PRIVATE project_warnings project_options mhwd::mhwd fmt::fmt hd::hd x86emu uuid)
target_link_libraries(mhwd-bin PRIVATE hd)
set_target_properties(mhwd-bin PROPERTIES OUTPUT_NAME mhwd)

install(
   TARGETS mhwd-bin
   RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
