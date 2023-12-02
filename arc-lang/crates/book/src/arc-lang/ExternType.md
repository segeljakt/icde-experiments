# Extern types

An **extern type** is an abstract data type with methods whose implementations are defined in Rust.

```grammar
ExternType ::= "extern" "type" [Name] [Generics]? [Where]? ";"
```

## Example

The following code shows how to define an extern type `String` in arc-lang.

```arc-lang
{{#include ../../../arc-lang/stdlib/std.arc:array}}
```
