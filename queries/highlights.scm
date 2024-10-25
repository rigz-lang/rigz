(string) @string

(number) @number

(comment) @comment

(assignment (identifier)) @variable

(function_call
  (function_identifier (identifier))) @function.method

(self) @variable.builtin

[
  (none)
  (bool)
] @constant.builtin
