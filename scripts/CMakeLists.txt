install(PROGRAMS mhwd
  DESTINATION /var/lib/mhwd/scripts)

install(PROGRAMS mhwd-fb
  DESTINATION /usr/bin)

install(FILES "initcpio/hooks/mhwd-fb"
  DESTINATION /usr/lib/initcpio/hooks)

install(FILES "initcpio/install/mhwd-fb"
  DESTINATION /usr/lib/initcpio/install)
