; Include

"#include" @attribute.include
(include_path) @string

; Preproc

[
  "#pragma"
] @attribute

(pragma_directive
  [
    "version"
    "not-version"
    "test-version-set"
  ] @attribute)

; Keywords

[
  "asm"
  "impure"
  "inline"
  "inline_ref"
  "method_id"
  "type"
] @keyword

[
  "return"
] @keyword.return

; Conditionals

[
  "if"
  "ifnot"
  "else"
  "elseif"
  "elseifnot"
  "until"
] @keyword.conditional

; Exceptions

[
  "try"
  "catch"
] @keyword.exception

; Repeats

[
  "do"
  "forall"
  "repeat"
  "while"
] @keyword.repeat

; Qualifiers
[
  "const"
  "global"
  (var)
] @type.qualifier

; Variables

(identifier) @variable

; Constants

(const_var_declarations
  name: (identifier) @constant)

; Functions/Methods

(function_definition
  name: (function_name) @function)

(function_application
  function: (identifier) @function)

(method_call
  method_name: (identifier) @function)
; @method.call)

; Parameters

(parameter) @parameter

; Types

(type_identifier) @type

(primitive_type) @type.builtin

; Operators

[
  "="
  "+="
  "-="
  "*="
  "/="
  "~/="
  "^/="
  "%="
  "~%="
  "^%="
  "<<="
  ">>="
  "~>>="
  "^>>="
  "&="
  "|="
  "^="
  "=="
  "<"
  ">"
  "<="
  ">="
  "!="
  "<=>"
  "<<"
  ">>"
  "~>>"
  "^>>"
  "-"
  "+"
  "|"
  "^"
  "*"
  "/"
  "%"
  "~/"
  "^/"
  "~%"
  "^%"
  "/%"
  "&"
  "~"
] @operator

; Literals

[
  (string)
  (asm_instruction)
] @string

[
  (string_type)
  (underscore)
] @character.special

(number) @number

; Punctuation

; ["{" "}"] @punctuation.bracket

; ["(" ")" "()"] @punctuation.bracket

; ["[" "]"] @punctuation.bracket

[
  ";"
  ","
  "->"
] @punctuation.delimiter

; Comments

(comment) @comment
