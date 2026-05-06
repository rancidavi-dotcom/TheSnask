; Placeholder queries for future tree-sitter-snask parsers.
; The plugin works today through Vim syntax + LSP semantic tokens.

[
  "let"
  "mut"
  "const"
  "fun"
  "class"
  "return"
  "new"
  "import"
  "import_c_om"
  "from"
] @keyword

[
  "if"
  "elif"
  "else"
  "while"
  "for"
  "in"
] @keyword.conditional

[
  "scope"
  "zone"
  "entangle"
  "with"
  "promote"
  "to"
  "unsafe"
] @keyword.storage

[
  "true"
  "false"
] @boolean

"nil" @constant.builtin

(comment) @comment
(string) @string
(number) @number
(function_declaration name: (identifier) @function)
(class_declaration name: (identifier) @type)
(call_expression function: (identifier) @function.call)

