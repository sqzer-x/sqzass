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
git clone https://github.com/devastator-x/sqzass
cd sqzass
cargo build --release
```

The binary lands at `target/release/sqzass`.

## Requirements

| | |
|---|---|
| Rust | 1.97 or newer |
| Everything else | nothing |

> [!NOTE]
> Prebuilt binaries and an AUR package are planned but not published yet.

## Checking the install

```bash
sqzass --version
```
