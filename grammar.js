module.exports = grammar({
    name: 'rigz',

    extras: $ => [
        $.comment, $._whitespace
    ],
    rules: {
        // TODO: add the actual grammar rules
        program: $ => seq(choice(
            $.assignment,
            $.expression,
        )),
        statement: $ => {
            $.import,
            $.export,
        },
        expression: $ => choice(
            $.binary_expression,
            $.unary_expression,
            $.function_call,
            $.value,
        ),
        assignment: $ => choice(
            seq("mut", $.identifier, "=", $.value),
            seq($.identifier, "=", $.value),
        ),
        _attribute: $ => choice(
            seq($.identifier, "=", $.value),
            $.function_call,
        ),
        object: $ => seq("{", $._attribute, repeat(seq(',', $._attribute)), optional(','), "}"),
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