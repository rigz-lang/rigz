/**
 * @file Rigz grammar for tree-sitter
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "rigz",
  extras: $ => [
    $._whitespace, $.comment,
  ],
  word: $ => $.identifier,
  rules: {
    // TODO: add the actual grammar rules
    program: $ => repeat1($.statement),
    statement: $ => prec.left(seq(choice(
        $.binary_assignment,
        $.assignment,
        $.function_definition,
        $.expression
    ), optional($._terminator))),
    type_definition: $ => choice(
        seq($._type, $.type_identifier, "=", $.type_object),
    ),
    type_object: $ => seq("{", seq($.identifier, "=", $.type), repeat(seq(',', seq($.identifier, "=", $.type))), optional(','), "}"),
    _terminator: _ => choice(";", "\n"),
    _type: _ => "type",
    _let: _ => "let",
    _mut: _ => "mut",
    _unless: _ => "unless",
    _if: _ => "if",
    _else: _ => "else",
    _fn: _ => "fn",
    _end: _ => "end",
    self: _ => "self",
    scope: $ => prec.right(choice(seq("=", $.expression), seq($.program, $._end))),
    assignment: $ => prec.right(seq(
        choice(seq(optional($._let), $.identifier), seq($._mut, $.identifier)),
        "=",
        $.expression
    )),
    binary_assignment: $ => seq(
        $.identifier,
        seq(choice("+", "-", "*", "/", "%", "^", "|", "||", "&", "&&", ">>", "<<"), "="),
        $.expression
    ),
    function_definition: $ => seq(
        optional($.lifecycle),
        $._fn, $.function_identifier, seq(
            optional($._function_args), optional(seq("->", optional($._mut), $.type))
        ),
        $.scope
    ),
    _function_args: $ => seq("(", $.function_arg,
        repeat(seq(',', $.function_arg)),
        ")"),
    function_arg: $ => prec(2, seq(
        $.identifier, optional(seq(":", optional($._mut), $.type))
    )),
    expression: $ => prec.right(seq(choice(
        $.value,
        $.function_call,
        $.unary,
        $.binary,
        $.self,
        $.do_scope,
        $.if_else,
        $.unless,
        seq("(", $.expression, ")")
    ), optional(choice($.cast, $.unless_guard, $.if_guard)))),
    do_scope: $ => seq(optional($.lifecycle), "do", $.scope),
    function_call: $ => choice(prec.right(seq(
        $.function_identifier,
        optional($._args)
    )), prec.left(2, seq($.expression, ".", $.function_call))),
    _args: $ => prec.right(seq($.expression, repeat(seq(",", $.expression)))),
    unary: $ => prec.left(seq(choice("-", "!"), $.expression)),
    binary: $ => prec.right(2, seq(
        $.expression,
        choice("+", "-", "*", "/", "%", "^", "|", "||", "&", "&&", ">>", "<<", "<=", "<", ">", ">=", "?:"),
        $.expression
    )),
    if_guard: $ => prec.right(seq($._if, $.expression)),
    unless_guard: $ => prec.right(seq($._unless, $.expression)),
    if_else: $ => prec.right(seq("if", $.expression, choice($.scope, seq("else", $.scope)))),
    unless: $ => prec.right(seq("unless", $.expression, $.scope)),
    value: $ => choice(
        $.none,
        $.bool,
        $.number,
        $.string,
        $.error,
        $.symbol,
        $.list,
        $.map,
        $.range,
    ),
    none: _ => "none",
    bool: _ => choice("false", "true"),
    number: _ => token(/\d[\d_]*(\.[\d_]*)?/),
    // todo support infinite ranges (requires VM changes) & full ascii character range
    range: $ => choice(
        seq(/[0-9]+/, "..", /[0-9]+/),
        seq($.char, "..", $.char)
    ),
    char: $ => seq("'", /\w/, "'"),
    string: $ => choice(
        $._single_quoted_string,
        $._double_quoted_string,
        $._backtick_string
    ),
    list: $ => seq("[", $.expression, repeat(seq(',', $.expression)), optional(','), "]"),
    map: $ => seq("{", $._attribute, repeat(seq(',', $._attribute)), optional(','), "}"),
    _attribute: $ => choice(
        seq($.identifier, "=", $.expression),
        $.expression
    ),
    symbol: _ => seq(":", /\w+/),
    type: $ => prec.left(seq(choice(
        "None",
        "Any",
        "Float",
        "Int",
        "Number",
        "String",
        "Range",
        seq("[", $.type, "]"),
        seq("{", $.type, "}"),
        seq("{", $.type, $.type, "}"),
        "Error",
        $.type_identifier,
    ), optional("!"), optional("?"))),
    _single_quoted_string: $ => /'([^'\\]|\\[\s\S])*'/,
    _double_quoted_string: $ => /"([^"\\]|\\[\s\S])*"/,
    _backtick_string: $ => /`([^`\\]|\\[\s\S])*`/,
    error: $ => seq("error", $._args),
    cast: $ => seq("as", $.type),
    lifecycle: $ => seq("@", $.identifier),
    function_identifier: $ => prec.left(choice(seq($.type, ".", $.identifier), $.identifier)),
    comment: $ => token(choice(
        seq('#', /[^\n]*/),
        seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
    )),
    type_identifier: _=> /[A-Z]\w+/,
    identifier: _ => /(\$[A-Za-z0-9_]*)|[a-z_][A-Za-z0-9_]*/,
    _whitespace: _ => /\s/,
  }
});
