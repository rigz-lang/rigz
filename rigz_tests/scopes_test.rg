mut a = 1
bar = do
    a += 1
    print :a, '=', a
    21 * a
end

fn foo = bar

@test
fn test_foo
  assert_eq foo, 42
  # scopes are only processed once
  assert_eq foo, 42
end

foo