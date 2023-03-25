find_package(PkgConfig REQUIRED)
pkg_check_modules(
  LIBALPM
  REQUIRED
  IMPORTED_TARGET
  libalpm>=13.0.0)

add_executable(mhwd-kernel-bin
    utils.cpp
    kernel.cpp
    mhwd-kernel.cpp
)

target_link_libraries(mhwd-kernel-bin PRIVATE project_warnings project_options fmt::fmt PkgConfig::LIBALPM)
if(CMAKE_CXX_COMPILER_ID MATCHES ".*Clang")
   target_link_libraries(mhwd-kernel-bin PRIVATE range-v3::range-v3)
endif()
set_target_properties(mhwd-kernel-bin PROPERTIES OUTPUT_NAME mhwd-kernel)

install(PROGRAMS mhwd
  DESTINATION /var/lib/mhwd/scripts)

install(
   TARGETS mhwd-kernel-bin
   RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

install(PROGRAMS mhwd-fb
  DESTINATION /usr/bin)

install(FILES "initcpio/hooks/mhwd-fb"
  DESTINATION /usr/lib/initcpio/hooks)

install(FILES "initcpio/install/mhwd-fb"
  DESTINATION /usr/lib/initcpio/install)
