# cel — a self-hosting compiler

**cel** is a statically-typed systems language that compiles to C. It is also a compiler written in itself, the Rust reference implementation bootstraps the cel compiler, which can then compile itself without Rust.

```
cel         ->  out.c  ->  cc  ->  native binary
(compiler)      (C99)             (your program)
```

Right now, the compiler is partially self-hosted but this is expected to be completed soon as only a couple of features are left for the compiler to be able to fully compile itself. Once that happens the rust version of the compiler will be deprecated and may even be removed fom the codebase since it will no longer be needed, all work will continue in Cel.

## Features 

- **Self-hosting.** The cel compiler is written in cel. A Rust reference compiler (`src/`) compiles `cel/main.ce` into C, which `cc` turns into a binary. That binary can recompile `cel/main.ce` with no Rust involved.
- **Minimal dependencies.** The compiler was built purely using the Rust standard library and `cc`, the generated C code targets C99 with `-fno-builtin`.
- **Full pipeline.** Hand-written lexer, recursive-descent parser with precedence climbing, static analysis / type inference, and C code generation — all built from scratch.
- **Real type system.** Primitives, pointers, fixed/dynamic arrays, structs, and **choice types** (tagged unions with exhaustive `match`). Short variable declarations with type inference (`x := expr`).
- **Codegen to C.** Compiles to C99 instead of targeting assembly or an IR.

## Examples

Fibonacci:

```c
include "core.ce";

fib: proc(n: i32) -> i32 {
    if n == 0 {
        return 0;
    } else if n == 1 {
        return 1;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

main: proc() {
    printf("fib(5) = %d\n".ptr, fib(5));
}
```

Structs, pointers, and type inference:

```c
Vec2: type struct {
    x: i32,
    y: i32,
}

main: proc() {
    pos: Vec2 = Vec2{34, 35};

    a: i32 = 69;
    d := &a;           // *i32, inferred
    e: i32 = d.*;      // pointer dereference

    msg := "hello\n\0";
    printf(msg.ptr);   // strings are {length, ptr}
}
```

Choice types (tagged unions) with `match`:

```c
Type_Builtin: type struct {
    kind: u8,
}
Type_Pointer: type struct {
    inner: *Type,
}
Type: type choice {
    Type_Builtin,
    Type_Pointer,
}

print_type: proc(t: Type) {
    // assigning the underlying t to t_
    match t_ := t {
        Type_Builtin => {
            printf("builtin\n".ptr);
        },
        Type_Pointer => {
            printf("pointer\n".ptr);
        }
    }
}
```

Extern FFI:
- Support for linking with libc
```c
write:  @extern proc(fd: i32, buf: *u8, count: i32) -> i32;
exit:   @extern proc(code: i32);
calloc: @extern proc(n: u64, size: u64) -> void_ptr;
```

## Project layout

```
src/       Rust reference compiler (lexer -> parser -> codegen)
cel/       cel compiler written in cel (self-hosting target)
samples/   example cel programs (fib, arrays, structs)
```

## Build

```sh
# First build the Rust compiler
cargo build --release

# compile the cel compiler with itself (the bootstrap)
./target/release/cel cel/main.ce    # produces `out` (the cel compiler)
./out samples/hello.cel             # Compile cel program 
```

## Status

The cel compiler is **partially self-hosting** — it can parse and compile many programs but is still missing `match`, choice types, and dynamic array codegen (the features it needs to build itself).

## language features

| feature | status |
|---------|--------|
| primitives (`u8`–`u64`, `i8`–`i64`, `bool`, `string`) | done |
| pointers (`*T`, `&`, `.*`) | done |
| fixed arrays (`[N]T`), dynamic arrays (`[+]T`) | done |
| structs | done |
| choice types (tagged unions) | Rust done, cel wip |
| exhaustive `match` | Rust done, cel wip |
| `if`/`else`, `for`, `defer` | done |
| type inference (`x := expr`) | done |
| `@extern` FFI, `@static` | done |
| `include`, `#FILE`, `#LINE` | done |
| slices (`[]T`) | planned |
