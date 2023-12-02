# Patterns

Patterns can be used to deconstruct values and bind their constituents with variables.

```grammar
Pattern ::=
  | [Name]                       # Variable binding
  | [Value]                      # Value comparison
  | [Pattern] ":" [Type]         # Type annotated
  | "case" [Path] [Pattern]?         # Variant deconstruction
  | "{" ([Name] (":" [Pattern])?)","* "}"  # Record deconstruction
  | "(" [Pattern] "," [Pattern]","* ")"    # Tuple deconstruction
  | [Pattern]? ".." ("="? [Pattern])?  # Range deconstruction
  | [Pattern] "or" [Pattern]         # Alternation
```

## Examples

### Tuples

```arc-lang
{{#include ../../../arc-lang/examples/tuples.arc:patterns}}
```

### Records

```arc-lang
{{#include ../../../arc-lang/examples/records.arc:patterns}}
```

### Enums

```arc-lang
{{#include ../../../arc-lang/examples/enums.arc:patterns}}
```

### Arrays

```arc-lang
{{#include ../../../arc-lang/examples/arrays.arc:patterns}}
```
