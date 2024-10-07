# rigz_runtime

Handles parsing and converting rigz to its VM instructions, no AST available (use [tree-sitter-rigz](https://crates.io/crates/tree-sitter-rigz) instead)

## Parser
This is an AST parser, but as soon as it has an element that would be in the AST that element is converted into VM 
instructions. Tokens are read from left to right, however expressions are built from right to left and there is no 
operator precedence. This means that `1 + 2 * 3` is not equal to `3 * 2 + 1`; the first is 7, (2 * 3) + 1, while the 
second is 9, (2 + 1) * 3.

## TODO
- update parser to support repl, may require VM updates as well
- Better error messages


### lifecycles

```
@on("event")
fn new_event(e) 
end

dispatch('event', {
    something: 32
})

@plan
fn foo
end

@apply
fn foo
end

[parse, run]
@plan = @after(@run)
@apply = @after(@plan, @confirm)
[parse, run, @plan, @confirm, @apply]

@plan
fn s3_bucket(name: string) 
end

@apply
fn s3_bucket(name: string) 
end

s3_bucket foo
```

### polc

```rigz
allow data.external {
    bin = "foo"
}
```


### database migration

```rigz
create_table foo, do |t|
    t.string bar 
    t.column baz, :number
    t.timestamps 
end
```


let a = $('cat file')

$```ruby

```