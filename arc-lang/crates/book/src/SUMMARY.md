# Summary

- [Introduction](introduction.md)
- [Getting Started](getting-started.md)
- [Rationale](rationale.md)
- [Tour of Arc-Lang](tour/mod.md)
  - [Basics](tour/basics.md)
  <!-- - [Tasks and Channels](tour/tasks.md) -->
  - [Queries](tour/queries.md)
- [Examples](examples/mod.md)
  - [Word Count](examples/word-count.md)
  - [TF-IDF](examples/tf-idf.md)
  - [Sensor Data Aggregation](examples/sensor-data-aggregation.md)
  - [Deduplication](examples/deduplication.md)
  - [Fraud Detection](examples/fraud-detection.md)
- [Command Line Interface](command-line-interface.md)
- [Arc-Lang Reference](arc-lang/mod.md)
  - [Programs](arc-lang/Program.md)
  - [Names](arc-lang/Name.md)
  - [Paths](arc-lang/Path.md)
  - [Values](arc-lang/Value.md)
  - [Types](arc-lang/Type.md)
  - [Effects](dev/arc-mlir/Effects.md)
  - [Blocks](arc-lang/Block.md)
  - [Statements](arc-lang/Stmt.md)
  - [Expressions](arc-lang/Expr.md)
  - [Patterns](arc-lang/Pattern.md)
  - [Queries](arc-lang/Query.md)
  - [Items](arc-lang/Item.md)
    - [Globals](arc-lang/Global.md)
    - [Functions](arc-lang/Def.md)
    <!-- - [Tasks](arc-lang/Task.md) -->
    - [Type Aliases](arc-lang/TypeAlias.md)
    - [Type Classes](arc-lang/TypeClass.md)
    - [Type Class Instances](arc-lang/Instance.md)
    - [Extern Functions](arc-lang/ExternDef.md)
    - [Extern Types](arc-lang/ExternType.md)
    - [Uses](arc-lang/Use.md)
    - [Modules](arc-lang/Module.md)
  - [Function bodies](arc-lang/Body.md)
  - [Arms](arc-lang/Arms.md)
  - [Sinks](arc-lang/Sinks.md)
  - [Parameters](arc-lang/Params.md)
  - [Generics](arc-lang/Generics.md)
  - [Type Arguments](arc-lang/TypeArgs.md)
  - [Assignments](arc-lang/Assign.md)
  - [Annotations](arc-lang/Annots.md)
- [Standard library](stdlib/mod.md)
  - [Strings](stdlib/String.md)
  - [Arrays](stdlib/Array.md)
  - [DataFrames](stdlib/DataFrame.md)
  - [Channels](stdlib/Channel.md)
- [Development](dev/mod.md)
  - [Continuous Integration](dev/ci.md)
  - [Arc-Lang Reference](dev/arc-lang/mod.md)
  - [Arc-MLIR Reference](dev/arc-mlir/mod.md)
    - [Programs](dev/arc-mlir/Program.md)
    - [Names](dev/arc-mlir/Name.md)
    - [Operations](dev/arc-mlir/Operation.md)
    - [Types](dev/arc-mlir/Type.md)
    - [Values](dev/arc-mlir/Value.md)
    - [Blocks](dev/arc-mlir/Block.md)
    - [Items](dev/arc-mlir/Item.md)
      - [Functions](dev/arc-mlir/Func.md)
    - [Parameters](dev/arc-mlir/Params.md)
  - [Arc-Sys Reference](dev/arc-sys/mod.md)
    - [Execution Model](dev/arc-sys/execution-model.md)
