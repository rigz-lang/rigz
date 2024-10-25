(string) @string

(number) @number

(comment) @comment

(assignment (identifier)) @variable

(function_call
  (function_identifier (identifier))) @function.method


[
  "do"
  "else"
  "end"
  "fn"
  "import"
  "let"
  "mut"
  "unless"
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
