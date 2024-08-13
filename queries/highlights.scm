(string) @string
(number) @number

(comment) @comment

(identifier) @variable

(function_call
  (function_identifier) @function.method)

[
  (none)
  (bool)
] @constant.builtin
