if exists("b:current_syntax")
  finish
endif

syn case match
syn sync minlines=80

syn keyword snifBoolean true false
syn keyword snifNull null nil
syn keyword snifKeyword package build dependencies dev_dependencies app features profile opt strip entry version
syn keyword snifKeyword library resources functions constants resource c_type constructor destructor surface_type depends_on
syn keyword snifKeyword c_function surface input output value type

syn match snifNumber "\v<[-+]?\d+%(\.\d+)?>"
syn region snifString start=+"+ skip=+\\\\\|\\"+ end=+"+ contains=snifEscape
syn match snifEscape "\\[nrt0\"\\]" contained
syn match snifKey "\v^\s*\zs[-_A-Za-z0-9.]+\ze\s*[:=]"
syn match snifComment "#.*$" contains=snifTodo
syn match snifComment "//.*$" contains=snifTodo
syn keyword snifTodo TODO FIXME NOTE contained

hi def link snifBoolean Boolean
hi def link snifNull Constant
hi def link snifKeyword Keyword
hi def link snifNumber Number
hi def link snifString String
hi def link snifEscape SpecialChar
hi def link snifKey Identifier
hi def link snifComment Comment
hi def link snifTodo Todo

let b:current_syntax = "snif"

