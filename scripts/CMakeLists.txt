install(PROGRAMS mhwd
  DESTINATION /var/lib/mhwd/scripts)

install(PROGRAMS mhwd-gpu
  DESTINATION /usr/bin)

install(PROGRAMS mhwd-kernel
  DESTINATION /usr/bin)

install(PROGRAMS mhwd-fb
  DESTINATION /usr/bin)

file(GLOB INCLUDEFILES "include/*")

install(PROGRAMS ${INCLUDEFILES}
  DESTINATION /var/lib/mhwd/scripts/include)

install(FILES "initcpio/hooks/mhwd-fb"
  DESTINATION /usr/lib/initcpio/hooks)

install(FILES "initcpio/install/mhwd-fb"
  DESTINATION /usr/lib/initcpio/install)
