+++
title = "Installation"
description = "One line, a prebuilt binary, or from source"
weight = 10
toc = true
+++

sqzass is a single binary with no runtime dependencies. No Node, no Python, no
system libraries.

## One line

```bash
curl -fsSL https://sqzass.sqzer.com/install.sh | sh
```

The script does exactly four things, and you can [read it](/install.sh) first:
detect the platform (Linux on x86_64 or ARM64, or Apple-silicon macOS), download the latest
release tarball with its `.sha256`, verify the checksum, and `install` the
binary into `/usr/local/bin` — asking for sudo only if that directory is not
writable. Set `SQZASS_INSTALL_DIR` to install somewhere else. It touches
nothing else: no shell config, no `PATH` edits.

With Rust installed, one line also builds from source:

```bash
cargo install --git https://github.com/sqzer-x/sqzass
```

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

Attached to each [release](https://github.com/sqzer-x/sqzass/releases): two
statically linked Linux builds, `x86_64-unknown-linux-musl` and
`aarch64-unknown-linux-musl`, and an `aarch64-apple-darwin` one, each with a
`.sha256` beside it.

```bash
curl -LO https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz
tar xzf sqzass-x86_64-unknown-linux-musl.tar.gz
sudo install -m755 sqzass-*/sqzass /usr/local/bin/
```

The Linux builds are static, so they have no glibc requirement and run on
distributions older than the one they were built on — which is also why one
tarball covers Debian, Fedora and Arch alike. They are the builds that use the
pure-Rust regex engine: everywhere else sqzass highlights with Oniguruma, which
is measurably faster on code-heavy sites, but Oniguruma is a C binding and a C
binding is exactly what breaks a musl static build. The engines' grammar sets
are not identical either: the pure-Rust engine cannot run a few grammars'
regexes, so the static artifacts drop seven of them entirely — PowerShell,
JavaScript (Babel), Salt State and ARM Assembly among them. A ` ```powershell `
or ` ```jsx ` fence that a native build highlights in full falls back to plain
text there, without an error. Even ` ```js ` differs in markup structure: the
native build resolves it to the Babel grammar, the static ones to plain
JavaScript. Each binary is still fully deterministic on its own.

## Checking the install

```bash
sqzass --version
```

Next: [Your first site](@/start/first-site.md).
