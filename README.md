# 🧠⚡ Brainstorm

Brainstorm is a brainfuck interpreter and debugger written in rust 🦀.

![Demo Gif](examples/demo.gif)

# Interpreter

To run the interpreter, a program has to be passed through the `--program` flag.
Optionally, the behaviour of the interpreter can be tuned through the `--tape-size`, `--eof-behaviour` and `--print-debug` flags.
See `--help` for more information.

# Debugger

To run the debugger, add the `--debugger` flag. To see the available commands, enter `help`.

# Debugging Units

The debugger supports splitting code through units.
To start a new unit, begin a line with a `;` character, and write the unit's name.

See the [examples](examples) directory for more examples.

# Performance

Internally, the interpreter parses the input program to coalesce consecutive increment/decrement or shift-right/shift-left commands.
The jump targets of bracket commands are also precalculated. These optimizations make the interpreter... not slow.

The interpreter wasn't built with performance in mind, but to facilitate the debugger, therefore it's not the fastest possible implementation. That said, it's fast enough for most cases.
