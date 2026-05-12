# Semantics

Syntax is the grammar of the language, checking that sentences (or lines of code) are written correctly.

On the other hand, semantics checks that these sentences makes sense.

In the code, this means that we do not perform any illegal operations.

Implemented in snake lang is:
- Scoping and declaration
    - Check that variables are declared before use.
    - Not allowing redeclaration of variables within the same scope.
        - Redefining parent scope is fine.

- Type checking
    - Functions
        - Return type and return statements must agree.
        - Callers must respect argument count and argument types.
    - Variables
        - Must conform to use in expressions.
        - Assignment of variable must conform in type.
    - If statement
        - Checks that condition evaluates to boolean.
