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
| Everything else | — |

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
distributions older than the one it was built on. That is the reason this
project pins syntect to its pure-Rust regex engine: the default one is a C
binding, and a C binding is exactly what breaks a musl static build.

> [!NOTE]
> There is no release yet — the workflow exists and the first tag has not been
> cut. Build from source until then.

## Checking the install

```bash
sqzass --version
```
