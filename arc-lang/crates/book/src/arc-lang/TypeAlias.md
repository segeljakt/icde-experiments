# Type Aliases

A **type alias** is a purely cosmetic alias of a type.

```grammar
TypeAlias ::= "type" [Name] [Generics]? "=" [Type] [Where]? ";"
```

## Example

The following code defines type aliases for representing lines on a two-dimensional plane, and a function for calculating the length of a line.

```arc-lang
{{#include ../../../arc-lang/examples/type-alias.arc:example}}
```
