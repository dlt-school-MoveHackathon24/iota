---
id: ir-to-bytecode
title: Move IR Compiler
custom_edit_url: https://github.com/move-language/move/edit/main/language/move-ir-compiler/README.md
---

## Summary

The Move IR compiler compiles the Move IR down to its bytecode representation.

## Overview

The Move IR compiler compiles modules and scripts written in Move down to
their respective bytecode representations. The two data types used to
represent these outputs are `CompiledModule` and `CompiledScript`. These
data types are defined in [file_format.rs](https://github.com/move-language/move/blob/main/language/move-binary-format/src/file_format.rs).

Beyond translating Move IR to Move bytecode, the compiler's purpose is as a
testing tool for the bytecode verifier. Because of this, its job is to
output bytecode programs that correspond as closely as possible to the
input IR; optimizations and advanced semantic checks are specifically not
performed during the compilation process. In fact, the compiler goes out of
its way to push these semantic checks into the bytecode, and compile
semantically invalid code in the Move IR to equivalent---semantically
invalid---bytecode programs. The semantics of the compiled bytecode is
then verified by the [bytecode verifier](https://github.com/move-language/move/blob/main/language/move-bytecode-verifier/README.md). The compiler command line
automatically calls the bytecode verifier at the end of compilation.

## Command-line options

```text
USAGE:
    compiler [FLAGS] [OPTIONS] <source-path>

FLAGS:
    -h, --help                 Prints help information
    -l, --list-dependencies    Instead of compiling the source, emit a dependency list of the compiled source
    -m, --module               Treat input file as a module (default is to treat file as a script)
        --no-verify            Do not automatically run the bytecode verifier
        --src-map
    -V, --version              Prints version information

OPTIONS:
    -d, --deps <deps-path>    Path to the list of modules that we want to link with

ARGS:
    <source-path>    Path to the Move IR source to compile
```

### Example Usage

> cargo build --bin compiler

- This will build the compiler + verifier binary.
- The binary can be found at `diem/target/debug/compiler`.
- Alternatively, the binary can be run directly with `cargo run -p compiler`.

To compile and verify `foo.mvir`, which contains a Move IR module:

> `compiler --address 0x42 --no-stdlib -m foo.mvir`

To compile and verify `bar.mvir`, which contains a transaction script:

> `compiler --address 0xca --no-stdlib bar.mvir`

## Folder Structure

```text
compiler                        # Main compiler crate. This depends on stdlib.
├── ir-to-bytecode              # Core backend compiler logic, independent of stdlib.
│   ├── src
│   │   ├── compiler.rs         # Main compiler logic - converts an AST generated by `syntax.rs` to a `CompiledModule` or `CompiledScript`.
│   │   └── parser.rs           # Wrapper around Move IR syntax crate.
│   └── syntax                  # Crate containing Move IR syntax.
│       └── src
│           ├── ast.rs          # Contains all the data structures used to build the AST representing the parsed Move IR input.
│           ├── lexer.rs        # Lexer for the Move IR language.
|           └── syntax.rs       # Parser for the Move IR language.
└── src
    ├── main.rs                 # Compiler driver - parses command line options and calls the parser, compiler, and bytecode verifier.
    └── util.rs                 # Misc compiler utilities.
```
