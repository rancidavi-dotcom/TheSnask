if exists("b:current_syntax")
  finish
endif

syn case match
syn sync minlines=80

syn keyword snaskKeyword let mut const fun class self return new import import_c_om from as
syn keyword snaskControl if elif else while for in
syn keyword snaskOm promote to scope zone entangle with unsafe
syn keyword snaskBoolean true false
syn keyword snaskNil nil
syn keyword snaskOperatorWord and or not
syn keyword snaskBuiltin print input len upper lower trim split join chars format range sort reverse unique flatten
syn keyword snaskBuiltin is_nil is_str is_obj read_file write_file append_file exists delete read_dir is_file is_dir create_dir
syn keyword snaskBuiltin http_get http_post time sleep exit args env cwd platform arch str_to_num num_to_str calc_eval
syn keyword snaskBuiltin abs floor ceil round pow sqrt min max sin cos wrapping_add wrapping_sub wrapping_mul wrapping_div
syn keyword snaskLowLevel u8 u16 u32 u64 i8 i16 i32 i64 ptr bool str num any void
syn keyword snaskLowLevel to_u8 to_u16 to_u32 to_u64 to_i8 to_i16 to_i32 to_i64
syn keyword snaskLowLevel bit_mask bit_not bit_set bit_clear bit_toggle bit_write bit_test flag_has flag_set flag_clear flag_write
syn keyword snaskLowLevel wrapping_inc wrapping_dec mem_alloc mem_alloc_zero mem_free ptr_add
syn keyword snaskLowLevel mem_read_u8 mem_read_u16 mem_read_u32 mem_write_u8 mem_write_u16 mem_write_u32 mem_fill_u8 mem_copy
syn keyword snaskModule gui os sfs path json sjson snif sqlite zlib blaze blaze_auth string snaskgui requests

syn match snaskDecorator "@\h\w*"
syn match snaskAttribute "\v<%(unsafe)>"
syn match snaskNamespace "\v<\h\w*::" contains=snaskModule
syn match snaskFunction "\v<\h\w*\ze\s*\("
syn match snaskClassName "\v<class\s+\zs\h\w*"
syn match snaskFunctionDecl "\v<fun\s+\zs\h\w*"
syn match snaskVariableDecl "\v<%(let|mut|const)\s+\zs\h\w*"

syn match snaskNumber "\v<0x[0-9A-Fa-f_]+>"
syn match snaskNumber "\v<0b[01_]+>"
syn match snaskNumber "\v<\d+%(\.\d+)?>"
syn match snaskOperator "==="
syn match snaskOperator "=="
syn match snaskOperator "!="
syn match snaskOperator "<="
syn match snaskOperator ">="
syn match snaskOperator "<<"
syn match snaskOperator ">>"
syn match snaskOperator "+="
syn match snaskOperator "-="
syn match snaskOperator "*="
syn match snaskOperator "/="
syn match snaskOperator "//"
syn match snaskOperator "[+\-*\/%&|^~<>=]"

syn region snaskString start=+"+ skip=+\\\\\|\\"+ end=+"+ contains=snaskEscape
syn match snaskEscape "\\[nrt0\"\\]" contained
syn match snaskEscape "\\x[0-9A-Fa-f]\{2}" contained

syn region snaskComment start="/\*" end="\*/" contains=snaskTodo
syn match snaskComment "//.*$" contains=snaskTodo
syn keyword snaskTodo TODO FIXME NOTE SAFETY HACK contained

syn match snaskImportPath "\v(import|from|import_c_om)\s+\"[^\"]+\"" contains=snaskKeyword,snaskString

hi def link snaskKeyword Keyword
hi def link snaskControl Conditional
hi def link snaskOm StorageClass
hi def link snaskBoolean Boolean
hi def link snaskNil Constant
hi def link snaskOperatorWord Operator
hi def link snaskBuiltin Function
hi def link snaskLowLevel Type
hi def link snaskModule Include
hi def link snaskDecorator PreProc
hi def link snaskAttribute PreProc
hi def link snaskNamespace Include
hi def link snaskFunction Function
hi def link snaskClassName Type
hi def link snaskFunctionDecl Function
hi def link snaskVariableDecl Identifier
hi def link snaskNumber Number
hi def link snaskOperator Operator
hi def link snaskString String
hi def link snaskEscape SpecialChar
hi def link snaskComment Comment
hi def link snaskTodo Todo
hi def link snaskImportPath Include

let b:current_syntax = "snask"
