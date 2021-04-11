# Command overview

The Calculator provides a minimalistic environment to some math.
It supports calculation of expressions, definition of function, ploting function's and
solving linar expressions for a given variable.

## Calculation

The calculation supports binary operators normal `+`, `-`, `*`, and `/`.
The binary `%` calculates the remainder, e.g. `17 % 5` returns `2`.
The other binary operator is for `^` power, e.g. `3 ^ 2` returns `9` and `3 ^ 4` returns `81`.

Functions can be called with the usual syntax, e.g. `abs(-1)` returns `1`.

Variables are definied with the `:=` operator, e.g. `a := 12`.
Such a variable can be used in expressions, e.g. `a * 3` returns `36`.
Variables can be redefined.
The new value is used for the next commands.
Calculator contains build-in constants (see below).
These constants *cannot* be redefined.

Custom functions are also defined with the `:=` operator, e.g. `add1(x) := x + 1`.
These custom functions are called with the syntax above, e.g. `add1(12)` returns `13`.
Like variables custom function can be redefined.
Functions can have more than one argument, e.g. `sum3(x, y, z) := x + y + z`.

Note: the `*` operator is not optional.

### Build-in functions

The Calculator contains the following build-in functions:

- `abs`
- `sin`
- `cos`
- `tan`
- `sinh`
- `cosh`
- `tanh`
- `asin`
- `acos`
- `atan`
- `asinh`
- `acosh`
- `atanh`
- `sqrt`
- `exp`
- `ln`
- `log2`
- `log10`

### Build-in constants

The build-in constants of Calculator are:

- `e`: Euler's constant
- `pi`

## Ploting functions

The Calculator can plot functions with one argument.
The command starts with the `plot` keyword followed by the function name, e.g. `plot sin`.

The plot appears below the command.
It can be moved and zoomed by touch and/or mouse.
To move by the mouse point into the graph hold the left mouse button and move the mouse.
To zoom scroll the mouse.
To move by touch move one finger in the graph plot and to zoom use the pinch-to-zoom gesture.

## Solving linear equations

The Calculator can solve linear equations with one variable with the `solve ... for ...` syntax.
Example: `solve x = 4 for x` returns `4`.
The equation may contain function calls, but the dependent variable must not appear in the arguments of the called functions.
Other variables in the equation must be defined.

A more complex example: `solve 12 * x = 33 + x for x` returns `3`.
