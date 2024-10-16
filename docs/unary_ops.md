All unary operations take a register and an output, the output is where the result of the operation is stored.

## Not
Binary Not for numbers, convert to boolean for all other types

## Neg
Negative of the value, complex types are unchanged. Uint is 2's complement

## Rev
Reverse the value in register and store in output

## Print
Print the value in register to stdout, optional store the value in another register (use 0 otherwise)

## EPrint
Print the value in register to stderr, optional store the value in another register (use 0 otherwise)

## PrintLn
Print the value in register to stdout with a trailing newline, optionally store the value in another register (use 0 otherwise)

## EPrintLn
Print the value in register to stderr with a trailing newline, optionally store the value in another register (use 0 otherwise)
