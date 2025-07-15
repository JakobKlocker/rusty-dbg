# `rusty-dbg`: A Minimal x86_64 Linux Debugger in Rust

`rusty-dbg` is a command-line debugger for Linux systems, written entirely in Rust. It provides basic functionality like stepping through instructions, setting breakpoints, inspecting registers and memory, disassembling instructions, and viewing DWARF-based debug info.

---

## Features

| Feature                  | Command            | Description                                          |
|--------------------------|--------------------|------------------------------------------------------|
| **Continue Execution**   | `cont` / `c`       | Resume process execution                            |
| **Step Instruction**     | `step` / `s`       | Single-step the next instruction                    |
| **Step Over**            | `next` / `n`       | Step over function calls                            |
| **Set Breakpoint**       | `bp` / `b`         | Set breakpoint at address or function               |
| **Remove Breakpoint**    | `rm-bp` / `rmb`    | Remove a breakpoint by address                      |
| **List Breakpoints**     | `show-bp`          | Show all breakpoints                                |
| **Inspect Registers**    | `regs`             | View all CPU register values                        |
| **Set Register Value**   | `sr <reg> <val>`   | Set a register’s value                              |
| **Get Register Value**   | `gr <reg>`         | Print value of a register                           |
| **Read Memory**          | `dump <addr> [n]`  | Dump `n` bytes at address `addr`                    |
| **Patch Memory**         | `patch <addr> <v>` | Write a value into memory at given address          |
| **Disassemble Code**     | `disas`            | Disassemble instructions at current RIP             |
| **Backtrace**            | `bt` / `backtrace` | Show the current stack trace                        |
| **Sections Info**        | `sections` / `sec` | Print section headers from ELF                      |
| **Function Offset**      | `offset`           | Show offset from base address                       |
| **Function Listing**     | `functions`        | Print list of known functions (ELF symbols)         |
| **Exit Debugger**        | `exit`             | Quit the debugger                                   |

---

### Requirements

- Linux (x86_64)
- Rust (stable)
- `ptrace` permission (usually root or via `cap_sys_ptrace`)

### Build & Run

```bash
cargo build --release
sudo ./target/release/rusty-dbg <pid|path-to-binary>
```

Or just run from source:

```bash
sudo cargo run -- <pid|binary>
```

---

## Limitations

This is a **work-in-progress** learning project and lacks many advanced features:

- No thread support
- Minimal signal handling
- No shared library breakpoint persistence
- Only works on **x86_64 Linux** ELF binaries

---

## Feedback Welcome

**If you have ideas or critiques, especially around architecture or organization I'd love to hear your feedback.**
