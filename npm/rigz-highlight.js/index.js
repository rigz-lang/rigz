/*
Language: Rigz
Author: Mitch <email@domain.com>
Description: Rigz is a dynamic scripting language; inspired by Ruby, Kotlin, and Rust.
Website: https://rigz-lang.org
*/

export function rigz(hljs) {
    const keywords = {
        'variable.language': [
            "self"
        ],
        keyword: [
            "fn",
            "let",
            "mut",
            "for",
            "next",
            "break",
            "loop",
            "do",
            "end",
            "match",
            "in",
            "var",
            "try",
            "catch",
            "return",
            "if",
            "else",
            "unless",
            "impl",
            "object",
            "enum",
            "as",
            "import",
            "trait",
            "on",
            "mod"
        ],
        built_in: [
            "raise",
            "send",
            "spawn",
            "receive",
            "log",
            "format",
            "not",
            "print",
            "printf",
            "puts",
            "attr",
            "assert",
            "assert_eq",
            "assert_neq",
            "int_from_bits",
            "float_from_bits",
        ],
        literal: ["true", "false", "none"],
        type: [
            "Any",
            "Assertions",
            "Collections",
            "Date",
            "Error",
            "File",
            "Float",
            "Html",
            "Http",
            "Int",
            "JSON",
            "List",
            "Log",
            "Map",
            "Math",
            "None",
            "Number",
            "Random",
            "Range",
            "Self",
            "String",
            "UUID",
            "VM",
        ]
    };

    const comments = {
        className: 'comment',
        variants: [
            hljs.HASH_COMMENT_MODE,
            hljs.C_BLOCK_COMMENT_MODE
        ]
    }

    const number = {
        className: 'number',
        variants: [
            hljs.NUMBER_MODE
        ]
    }

    const string = {
        className: 'string',
        variants: [
            hljs.APOS_STRING_MODE,
            hljs.QUOTE_STRING_MODE,
            {
                begin: '`',
                end: '`'
            }
        ]
    }

    const symbol = {
        className: 'symbol',
        begin: ':(?!\\s+)',
        contains: [
            {
                begin: '\\w+'
            }
        ],
        relevance: 0
    }

    // todo handle args, (231)
    const lifecycle = {
        className: 'meta',
        begin: '@(?!\\s+)',
        contains: [
            {
                begin: '\\w+'
            },
        ],
        relevance: 0
    }

    const operators = {
        scope: 'operator',
        match: /(\?:|\|>|\+\+|--|\.\.)|(([!+\-*/=<>%^]|&&?|\|\|?|>>|<<)=?)/
    }

    const contains = [
        number,
        string,
        comments,
        lifecycle,
        symbol,
        operators
    ]


    return {
        name: 'Rigz',
        aliases: [
            "rigz",
            "rg"
        ],
        keywords,
        contains
    }
}