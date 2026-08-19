# falsec, a compiler and interpreter for the False language, written in Rust.

Read about the language: https://strlen.com/false-language/ and https://esolangs.org/wiki/FALSE

## Installation

Install falsec from the AUR:

```sh
paru -S falsec
```

Or install the pre-compiled binary from the AUR:

```sh
paru -S falsec-bin
```

Or just run it from source:

```sh
cargo run -- --version
```

## Configuration

You can write a configuration file (read the [schema](./config.schema.json) and [Config struct](./falsec-types/src/lib.rs) for more),
then pass it to falsec using the `-c` or `--config` flag.

```json
{
    "$schema": "config.schema.json",
    "type_safety": "Lambda"
}
```

For the CLI, run the help command for general help or help on a given subcommand:

```sh
# subcommands: falsec run, falsec compile, falsec help

# general help:
falsec --help
falsec help

# help on <command> (where <command> is one of "run", "compile"):
falsec <command> --help
falsec help <command>
```

## Language Reference

From https://esolangs.org/wiki/FALSE#Commands:

```false
123'c$%\@Ø+-*/_&|~>=[]!?#a:;^,"asd".ß{dsa}
```

The `` ` `` command is not implemented, as that was used for 68000 machine instructions. If I implemented it, it could work in the compiler, but I'm not sure how to make it work in the interpreter. It's just a bandaid fix for missing language features anyway (syscalls, etc), which I'd rather implement with more language features instead, or just internal lambdas (for example `1_!` could call a builtin function that does something specific, like a syscall).

## Notes

The compiler does not use a standard library, so no `printf`. The `.` command calls a `print_decimal` function that I [wrote myself](./falsec-compiler/src/linux_x86_64_elf/boilerplate.rs#L244-L328) in assembly.

The `print_<x>` functions all use buffering to reduce the amount of syscalls, configurable with `stdout_buffer_size`.

