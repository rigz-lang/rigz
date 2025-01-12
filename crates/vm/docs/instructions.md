# Instructions

=======
## Special Registers
The following registers are special and cannot be written to, they will always return the following values.
R0 - Value::None
R1 - Value::Number(Number::Int(1))

## Halt

Halt the VM and return the value from Register

## Puts

Print a list of values, separated by a comma with a trailing newline

### Arguments:
args - The values to print

## Log

Log a template string followed by a list of values, requires `enable_logging` to be set. Disabled by default.

### Arguments:
level - log level (uses log built-ins)
args - The values to log


### Arguments:
Register - The register to return a value from

# Unary

Apply a unary operation to the value in `from` and save the result in `to`.

### Arguments: 
Unary {
    op: UnaryOperation - The unary operation
    from: Register - The register with the source value
    output: Register - the output register
}

## Binary
Apply a binary operation to the values in lhs and rhs, save the result in output 

### Arguments:
Binary {
    op: BinaryOperation - The binary operation to apply
    lhs: Register - The left hand side
    rhs: Register - The right hand side of the binary operation
    output: Register - The output register
}


## Load
Load value into register

### Arguments 
Register - The register to fill
Value - The value to use

## Copy
Copy the value `from` register to `to` register

### Arguments 
Register - from
Register  - to register

## Call
Create a call frame from this scope

### Arguments
usize - scope index

## CallEq

### Arguments
Register - lhs
Register - rhs
usize - scope index


## CallNeq

Call scope if registers not equal

### Arguments
Register - lhs
Register - rhs
usize - scope index

## IfElse

If `truthy` call if_scope, otherwise call the else scope.

### Arguments
truthy: Register - The value register to check
if_scope: usize - Scope to call if true
else_scope: usize - Scope to call if false

## Cast
Cast value in `from` to `rigz_type`, save output in `to`

### Arguments:
from: Register - Original value register 
to: Register - Result Register
rigz_type: RigzType - Type to convert to

# Ret
Return the current call frame and set call frame to parent to parent

## LoadLet
Load an immutable variable into the current frame

### Arguments 
String - Name of the variable
Value - The value to use

## LoadMut

Load a mutable variable into the current frame

### Arguments 
String - Name of the variable
Value - The value to use

## GetVariable

Get the named variable and store in register

### Arguments 
String - The name of the variable
Register  - The register to store the output

## LoadLetRegister

Load let value from register

### Arguments 
String - The name of the variable
Register  - The register to store the value

## LoadMutRegister
Load let value from register

### Arguments 
String - The name of the variable
Register  - The register to store the value