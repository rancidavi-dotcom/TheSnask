if exists("current_compiler")
  finish
endif
let current_compiler = "snask"

CompilerSet makeprg=snask\ build\ %
CompilerSet errorformat=%f:%l:%c:\ %t%*[^:]:\ %m,%f:%l:%c:\ %m,%Eerror[%m],%Eerror:\ %m,%Wwarning:\ %m,%-G%.%#
