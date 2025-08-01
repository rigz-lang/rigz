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
        program: $ => prec.left(repeat1($.statement)),
        statement: $ => prec.left(seq(choice(
            $.binary_assignment,
            $.assignment,
            $.function_definition,
            $.expression,
            $.instance_set,
            $.import,
            $.mod,
            $.trait,
            $.object,
            $.impl,
            $.enum,
        ), optional($._terminator))),
        type_definition: $ => choice(
            seq($._type, $.type_identifier, "=", $.type_object),
        ),
        type_object: $ => seq("{", seq($.identifier, "=", $.type), repeat(seq(',', seq($.identifier, "=", $.type))), optional(','), "}"),
        mod: $ => seq("mod", $.type, repeat($.program), "end"),
        object: $ => seq("object", $.type, repeat($.program), "end"),
        trait: $ => seq("trait", $.type, repeat(choice($.function_declaration, $.function_definition)), "end"),
        impl: $ => seq("impl", $.type, "for", $.type, repeat($.function_definition), "end"),
        enum: $ => seq("enum", $.type_identifier, "do", $.type, repeat(seq($.type, ",")), optional($.type), "end"),
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
        import: $ => seq("import", choice($.type, $.string)),
        scope: $ => prec.right(choice(seq("=", $.expression), seq($.program, $._end))),
        assignment: $ => choice(prec.right(seq(
            choice(seq(optional($._let), $.identifier), seq($._mut, $.identifier)),
            "=",
            $._expression_or_lambda
        )), $.tuple_assign),
        tuple_assign: $ => seq(
            optional(choice($._let, $._mut)),
            seq("(", seq(optional(choice($._let, $._mut)), $.identifier), repeat1(seq(",", seq(optional(choice($._let, $._mut)), $.identifier))), ")"),
            "=",
            $.expression
        ),
        binary_assignment: $ => seq(
            $.identifier,
            seq(choice("+", "-", "*", "/", "%", "^", "|", "||", "&", "&&", ">>", "<<"), "="),
            $.expression
        ),
        instance_set: $ => seq($.expression, repeat1(choice(seq(".", choice($.identifier, $.int)), $.index)), "=", $.expression),
        function_declaration: $ => seq(
            optional($.lifecycle),
            $._fn, $.function_identifier, seq(
                optional($._function_args), optional(seq("->", optional($._mut), $.type))
            ),
        ),
        function_definition: $ => prec(2, seq(
            optional($.lifecycle),
            $._fn, $.function_identifier, seq(
                optional($._function_args), optional(seq("->", optional($._mut), $.type))
            ),
            $.scope
        )),
        _function_args: $ => seq("(", $.function_arg,
            repeat(seq(',', $.function_arg)),
            ")"),
        function_arg: $ => prec(2, seq(
            $.identifier, optional(seq(":", optional($._mut), $.type))
        )),
        expression: $ => prec.right(1, seq(choice(
            $.value,
            $.function_call,
            $.unary,
            $.binary,
            $.self,
            $.do_scope,
            $.if_else,
            $.unless,
            $.for_list,
            $.for_map,
            $.tuple,
            $.match,
            seq("try", $.expression),
            seq($.expression, "catch", seq(optional(seq("|", $.identifier, "|")), choice(seq(repeat($.statement), $.expression, "end"), seq("=", $.expression)))),
            seq("return", optional($.expression)),
            // todo support string interpolation
            seq("(", $.expression, ")")
            // todo into isn't quite right here foo 1, 2, 4 |> bar, as-is it will pass 4 into bar instead of foo 1, 2, 4
        ), optional(choice($.cast, $.unless_guard, $.if_guard, $.into, repeat1($.index))))),
        match: $ => seq("match", $.expression, "do",
            choice(repeat1($.match_variant),
                choice(
                    seq(
                        choice($.identifier, "else"),
                        choice(
                            seq("=>", $.expression, optional(",")),
                            $.match_variant_scope
                        )
                    ),
                    seq($.match_variant_lhs, "=>", $.expression, optional(",")),
                )
            ),
            "end"),
        match_variant: $ => seq($.match_variant_lhs, $.match_variant_body),
        match_variant_lhs: $ => prec.left(seq(choice(
            seq(".", choice($.identifier, $.type_identifier), optional($.expression)),
            seq($.type_identifier, ".", choice($.identifier, $.type_identifier), optional($.expression)),
        ), optional(choice($.unless_guard, $.if_guard)))),
        match_variant_body: $ => prec.left(choice(seq("=>", $.expression, ","), $.match_variant_scope)),
        match_variant_scope: $ => prec.left(1, choice(seq("=>", "{", repeat($.statement), $.expression, "}"), seq("do", repeat($.statement), $.expression, "end"))),
        do_scope: $ => seq(optional($.lifecycle), "do", $.scope),
        index: $=> seq("[", $.expression, "]"),
        function_call: $ => choice(prec.right(seq(
            $.function_identifier,
            optional($._args)
        )), prec.left(2, seq($.expression, ".", $.function_call))),
        into: $ => prec.left(repeat1(prec(3, seq("|>", $.function_call)))),
        _args: $ => prec.right(seq($._expression_or_lambda, repeat(seq(",", $._expression_or_lambda)))),
        unary: $ => prec.left(seq(choice("-", "!"), $.expression)),
        binary: $ => prec.right(2, seq(
            $.expression,
            choice("+", "-", "*", "/", "%", "^", "|", "||", "&", "&&", ">>", "<<", "<=", "<", ">", ">=", "?:"),
            $.expression
        )),
        _expression_or_lambda: $ => choice($.expression, $.lambda),
        lambda: $ => choice(
            seq("|", optional(seq($.function_arg, repeat(seq(',', $.function_arg)))), "|", $.expression),
            seq("{", "|", optional(seq($.function_arg, repeat(seq(',', $.function_arg)))), "|", $.expression, "}"),
            seq("do", "|", optional(seq($.function_arg, repeat(seq(',', $.function_arg)))), "|", $.scope),
        ),
        tuple: $ => seq("(", $.expression, repeat1(seq(",", $.expression)), ")"),
        for_list: $ =>
            seq("[", "for", $.identifier, "in", $.expression, ":", $.expression, "]"),
        for_map: $ =>
            seq("{", "for", $.identifier, optional(seq(",", $.identifier)), "in", $.expression, ":", choice(seq($.expression, ",", $.expression), $.expression), "}"),
        if_guard: $ => prec.right(seq($._if, $.expression)),
        unless_guard: $ => prec.right(seq($._unless, $.expression)),
        if_else: $ => prec.right(1, seq("if", $.expression, choice($.scope, seq("else", $.scope)))),
        unless: $ => prec.right(1, seq("unless", $.expression, $.scope)),
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
        number: _ => token(/-?\d[\d_]*(\.[\d_]*)?/),
        int: _ => token(/-?\d[\d_]*/),
        // todo support infinite ranges (requires VM changes) & full ascii character range
        range: $ => choice(
            seq(/[0-9]+/, "..", /[0-9]+/),
            seq($.char, "..", $.char)
        ),
        char: $ => seq("'", /./, "'"),
        string: $ => choice(
            $._single_quoted_string,
            $._double_quoted_string,
            $._backtick_string
        ),
        list: $ => prec(1, seq("[", $.expression, repeat(seq(',', $.expression)), optional(','), "]")),
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
            "List",
            "Map",
            "VM",
            seq("(", $.type, repeat(seq(",", $.type)), ")"),
            seq("[", $.type, "]"),
            seq("{", $.type, "}"),
            seq("{", $.type, $.type, "}"),
            "Error",
            $.type_identifier,
        ), optional(choice(repeat1(seq("::", $.type)), "!", "?")))),
        _single_quoted_string: $ => /'([^'\\]|\\[\s\S])*'/,
        _double_quoted_string: $ => /"([^"\\]|\\[\s\S])*"/,
        _backtick_string: $ => /`([^`\\]|\\[\s\S])*`/,
        // todo support args for raise, right now tuples are required
        error: $ => prec.right(seq("raise", $.expression)),
        cast: $ => seq("as", $.type),
        lifecycle: $ => seq("@", $.identifier),
        function_identifier: $ => prec.left(1, choice(seq($.type, ".", $.identifier), $.identifier)),
        comment: $ => token(choice(
            seq('#', /[^\n]*/),
            seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
        )),
        type_identifier: _ => /[A-Z]\w+/,
        identifier: _ => /(\$[A-Za-z0-9_]*)|[a-z_][A-Za-z0-9_]*/,
        _whitespace: _ => /\s/,
    }
});
