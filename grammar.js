module.exports = grammar({
    name: 'rigz',

    extras: $ => [
        $.comment, $._whitespace
    ],
    rules: {
        program: $ => repeat1($.statement),
        statement: $ => choice(
            $._import,
            $._export,
            $.expression,
            $.assignment,
            $.type_definition,
        ),
        _import: $ => seq("import"),
        _export: $ => seq("export"),
        expression: $ => choice(
            $.function_call,
            $.binary_expression,
            $.unary_expression,
            $.value,
        ),
        binary_expression: $ => prec.left(2, choice(
            seq($.expression, choice("+", "-", "*", "/", ">>", "<<", "|", "||", "&", "&&", "^", "%", "==", "!=", "<=", "<", ">", ">="), $.expression))
        ),
        unary_expression: $ => choice(
            seq(choice("!", "-"), $.expression)
        ),
        function_call: $ => prec.left(2, seq(
            $.identifier,
            optional(seq($.expression, repeat(seq(",", $.expression))))
        )),
        closure: $ => seq("{",
            optional(choice(
                seq($.identifier, repeat(seq(",", $.identifier))),
                seq($.identifier, ":", $.type, repeat(seq(",", $.identifier, $.type)))
            )),
            "->",
            $.program,
            "}"
        ),
        assignment: $ => choice(
            seq("mut", $.identifier, "=", $.expression),
            seq($.identifier, "=", $.expression),
        ),
        identifier: $ => $._valid_chars,
        type_identifier: $ => token(/[A-Z]\w+/),
        _valid_chars: $ => token(/(\w|\.|\$)+/),
        type_definition: $ => choice(
            seq("type", $.type_identifier, "=", $.type_object),
        ),
        _attribute: $ => choice(
            seq($.identifier, "=", $.expression),
            $.expression
        ),
        type: $ => choice(
            "None",
            "Float",
            "Int",
            "Uint",
            "String",
            seq("[", $.type, "]"),
            seq("{", $.type, "}"),
            seq("{", $.type, $.type, "}"),
            seq("error", $.string)
        ),
        value: $ => choice($.none, $.bool, $.number, $.string, $.object, $.list, $.closure),
        none: $ => 'none',
        bool: $ => choice('true', 'false'),
        number: $ => /\d+(\.\d+)?/,
        string: $ => choice(
            $._single_quoted_string,
            $._double_quoted_string,
            $._backtick_string
        ),
        _single_quoted_string: $ => /'([^'\\]|\\[\s\S])*'/,
        _double_quoted_string: $ => /"([^"\\]|\\[\s\S])*"/,
        _backtick_string: $ => /`([^`\\]|\\[\s\S])*`/,
        object: $ => seq("{", $._attribute, repeat(seq(',', $._attribute)), optional(','), "}"),
        type_object: $ => seq("{", seq($.identifier, "=", $.type), repeat(seq(',', seq($.identifier, "=", $.type))), optional(','), "}"),
        list: $ => seq("[", $.value, repeat(seq(',', $.value)), optional(','), "]"),
        comment: $ => token(
            choice(
                seq("#", /.*/),
                seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
            )
        ),
        _whitespace: $ => /[\s\uFEFF\u2060\u200B\u00A0]+/,
    }
});