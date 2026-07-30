+++
title = "Installation"
description = "Build sqzass from source"
weight = 10
toc = true
+++

sqzass is a single binary with no runtime dependencies. No Node, no Python, no
system libraries.

## From source

```bash
git clone https://github.com/sqzer-x/sqzass
cd sqzass
cargo build --release
```

The binary lands at `target/release/sqzass`.

## Requirements

| Requirement | Version |
|---|---|
| Rust | 1.97 or newer |
| A C compiler | rustc drives the linker with it; the default build also compiles Oniguruma, the highlighter's regex engine, with it. |
| Everything else | — |

The exact configuration the static release ships — the pure-Rust regex
engine, no C source compiled — is a feature flag away:

```bash
cargo build --release --no-default-features --features pure-rust
```

## Prebuilt binaries

Attached to each [release](https://github.com/sqzer-x/sqzass/releases): a
statically linked `x86_64-unknown-linux-musl` build and an
`aarch64-apple-darwin` one, each with a `.sha256` beside it.

```bash
curl -LO https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-v0.1.0-x86_64-unknown-linux-musl.tar.gz
tar xzf sqzass-*.tar.gz
sudo install -m755 sqzass-*/sqzass /usr/local/bin/
```

The Linux build is static, so it has no glibc requirement and runs on
distributions older than the one it was built on. It is also the one build
that uses the pure-Rust regex engine: everywhere else sqzass highlights with
Oniguruma, which is measurably faster on code-heavy sites, but Oniguruma is a
C binding and a C binding is exactly what breaks a musl static build. The
engines' grammar sets are not identical either: the pure-Rust engine cannot
run a few grammars' regexes, so the static artifact drops seven of them
entirely — PowerShell, JavaScript (Babel), Salt State and ARM Assembly among
them. A ` ```powershell ` or ` ```jsx ` fence that a native build highlights
in full falls back to plain text in the static artifact, without an error.
Even ` ```js ` differs in markup structure: the native build resolves it to
the Babel grammar, the static one to plain JavaScript. Each binary is still
fully deterministic on its own.

## Checking the install

```bash
sqzass --version
```
