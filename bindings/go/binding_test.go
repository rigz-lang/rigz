package tree_sitter_rigz_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-rigz"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_rigz.Language())
	if language == nil {
		t.Errorf("Error loading Rigz grammar")
	}
}
