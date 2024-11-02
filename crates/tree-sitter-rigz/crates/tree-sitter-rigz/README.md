# tree-sitter-rigz

## Setup for Neovim
1. Add the following to init.lua
```lua
vim.filetype.add({
  extension = {
    rigz = "rigz",
  },
})

local parser_config = require "nvim-treesitter.parsers".get_parser_configs()
parser_config.rigz = {
  install_info = {
    url = "~/parsers/tree-sitter-rigz", -- local path or git repo
    files = {"src/parser.c"},
    generate_requires_npm = false, -- if stand-alone parser without npm dependencies
    requires_generate_from_grammar = false, -- if folder contains pre-generated src/parser.c
    branch = "main", -- default branch is not master
    filetype = 'rigz',
  },
}
```

2. curl "https://gitlab.com/magicfoodhand/tree-sitter-rigz/-/raw/main/queries/highlights.scm?ref_type=heads" -o ~/.local/share/nvim/lazy/nvim-treesitter/queries/rigz/highlights.scm

# Rust Derive Macro Highlighting

```scheme
(macro_invocation
  (scoped_identifier
    name: (identifier) @name (#eq? @name derive_module!))

  (token_tree (raw_string_literal) @rigz_string)
)

(macro_invocation
  (scoped_identifier
    path: (identifier) @path (#eq? @path rigz_ast_derive)
    name: (identifier) @name (#eq? @name derive_module!))

  (token_tree (raw_string_literal) @rigz_string)
)
```
