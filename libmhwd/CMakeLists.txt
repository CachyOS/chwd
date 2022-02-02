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

add_library(mhwd SHARED ${LIB_SOURCES} ${LIB_HEADERS})
target_link_libraries(mhwd hd::hd)

add_library(mhwd::mhwd ALIAS mhwd)

install(TARGETS mhwd
  LIBRARY DESTINATION lib
)
