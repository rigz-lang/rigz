@test
fn test_import
    import "utils.rg"

    assert_eq foo, 42
end