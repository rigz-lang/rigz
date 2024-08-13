module.exports = grammar({
    name: 'rigz',

    extras: $ => [
        $.comment, $._whitespace
    ],
    conflicts: $ => [
        [$.function_call],
        [$.statement, $.binary],
        [$.unary, $.binary],
        [$.binary],
        [$.expression],
        [$.binary, $.assignment],
        [$.binary, $.function_call],
        [$.trailing_if, $.binary],
        [$.trailing_unless, $.binary],
    ],
    rules: {
        program: $ => repeat1($.statement),
        statement: $ => seq(choice(
            $.expression,
            $.function_definition,
            $.assignment,
            // $.import,
            // $.export,
        ), optional(";")),
        function_definition: $ => seq(
            optional($.lifecycle),
            "fn", $.function_identifier, "(", ")", optional($.type),
            $.program,
            "end"
        ),
        expression: $ => seq(choice(
            $.function_call,
            $.value,
            $.binary,
            $.unary,
            $.if_else,
            $.unless,
            seq("(", $.expression, ")"),
        ), optional(
            choice(
                $.cast,
                $.trailing_if,
                $.trailing_unless,
            )
        )),
        trailing_if: $ => seq("if", $.expression),
        trailing_unless: $ => seq("unless", $.expression),
        if_else: $ => seq("if", $.expression, $.program, optional(seq("else", $.program)), "end"),
        unless: $ => seq("unless", $.expression, $.program, "end"),
        cast: $ => seq("as", $.type),
        unary: $ => seq(choice("-", "!"), $.expression),
        binary: $ => seq(
            $.expression,
            choice("+", "-", "*", "/", "%", "^", "|", "||", "&", "&&", ">>", "<<", "<=", "<", ">", ">="),
            $.expression
        ),
        symbol: $ => seq(":", /\w+/),
        assignment: $ => seq(
            choice("let", "mut"),
            $.identifier,
            "=",
            $.expression,
        ),
        function_call: $ => prec(3, seq(
            $.function_identifier,
            optional(seq($.expression, repeat(seq(",", $.expression))))
        )),
        none: $ => token('none'),
        value: $ => choice(
          $.none,
            $.bool,
            $.number,
            $.string,
            $.list,
            $.map,
            $.error,
            $.symbol
        ),
        scope: $ => seq(optional($.lifecycle), "do", $.program, "end"),
        lifecycle: $ => seq("@", $.identifier),
        error: $ => seq("error", $.string),
        bool: $ => choice('true', 'false'),
        number: $ => /\d+(.\d+)?/,
        string: $ => choice(
            $._single_quoted_string,
            $._double_quoted_string,
            $._backtick_string
        ),
        _single_quoted_string: $ => /'([^'\\]|\\[\s\S])*'/,
        _double_quoted_string: $ => /"([^"\\]|\\[\s\S])*"/,
        _backtick_string: $ => /`([^`\\]|\\[\s\S])*`/,
        map: $ => seq("{", $._attribute, repeat(seq(',', $._attribute)), optional(','), "}"),
        type_object: $ => seq("{", seq($.identifier, "=", $.type), repeat(seq(',', seq($.identifier, "=", $.type))), optional(','), "}"),
        type_identifier: $ => /[A-Z]\w+/,
        type_definition: $ => choice(
            seq("type", $.type_identifier, "=", $.type_object),
        ),
        _attribute: $ => choice(
            prec.left(2, seq($.identifier, "=", $.expression)),
            $.expression
        ),
        type: $ => choice(
            "None",
            "Any",
            "Float",
            "Int",
            "Uint",
            "String",
            seq("[", $.type, "]"),
            seq("{", $.type, "}"),
            seq("{", $.type, $.type, "}"),
            "Error" // TODO switch to zig style ! for errors, add zig try catch
        ),
        list: $ => seq("[", $.value, repeat(seq(',', $.value)), optional(','), "]"),
        identifier: $ => /\w+/,
        function_identifier: $ => /[$\w][\w.]*/,
        comment: $ => token(
            choice(
                seq("#", /.*/),
                seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
            )
        ),
        _whitespace: $ => /[\s\uFEFF\u2060\u200B\u00A0]+/,
    }
})