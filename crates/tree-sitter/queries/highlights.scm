(string) @string

(number) @number

(comment) @comment

(assignment (identifier)) @variable

(function_call
  (function_identifier (identifier))) @function.method


[
  "do"
  "if"
  "else"
  "end"
  "fn"
  "import"
  "let"
  "mut"
  "raise"
  "unless"
  "try"
  "catch"
  "match"
  "enum"
  "object"
] @keyword

[
  "="
  "->"
  "+"
  "-"
  "*"
  "/"
  "%"
  "^"
  "|"
  "||"
  "&"
  "&&"
  ">>"
  "<<"
  "<="
  "<"
  ">"
  ">="
  "?:"
  "|>"
] @operator

[
  ","
  ";"
  "."
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

(self) @variable.builtin

[
  (none)
  (bool)
] @constant.builtin
