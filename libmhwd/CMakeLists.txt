include_directories(. vita ${mhwd_SOURCE_DIR}/src)

###
### mhwd library
###

set(LIB_HEADERS
    const.hpp
    vita/string.hpp
)

set(LIB_SOURCES
    vita/string.cpp
)

add_library(mhwd STATIC ${LIB_SOURCES} ${LIB_HEADERS})
target_link_libraries(mhwd PRIVATE project_warnings project_options hd::hd x86emu uuid)
if(CMAKE_CXX_COMPILER_ID MATCHES ".*Clang")
   target_link_libraries(mhwd PRIVATE range-v3::range-v3)
endif()

add_library(mhwd::mhwd ALIAS mhwd)

install(TARGETS mhwd
  LIBRARY DESTINATION lib
)
