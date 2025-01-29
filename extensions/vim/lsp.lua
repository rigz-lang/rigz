vim.lsp.start({
  cmd = { "rigz_lsp" },
  root_dir = vim.fn.getcwd(), -- Use PWD as project root dir.
})