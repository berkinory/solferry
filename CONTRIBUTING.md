# Contributing

## Setup

You need stable Rust. The core and CLI support Rust 1.91. The desktop app follows GPUI and needs 1.98 or newer. Install the tools once:

```sh
cargo install just taplo-cli typos-cli cargo-nextest cargo-deny
```

## Everyday commands

| Command | Does |
|---|---|
| `just fmt` | Format Rust and TOML |
| `just lint` | Formatting, spelling and clippy with warnings denied |
| `just test` | Run the test suite |
| `just check` | Everything CI runs |

## Conventions

- Keep files under about 300 lines and split by responsibility.
- Comment only what the code cannot say.
- No catch-all arms over `Error` in user-facing code.
- Add a test only for critical behavior that nothing else protects.
- Use Conventional Commits.

## Network scenarios

`tests/netlab` runs real binaries through NAT, packet loss, outages and restarts in Linux network namespaces. It needs passwordless sudo and an `iroh-relay` binary.

```sh
cargo build --release -p beam-cli
cargo install iroh-relay --features server
python3 tests/netlab/scenarios.py iroh-relay
```

Pass scenario names after the relay path to run only some of them.
