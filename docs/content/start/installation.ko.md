+++
title = "설치"
description = "소스에서 sqzass를 빌드합니다"
weight = 10
toc = true
+++

sqzass는 런타임 의존성이 없는 단일 바이너리입니다. Node도, Python도, 시스템
라이브러리도 필요 없습니다.

## 소스에서 빌드

```bash
git clone https://github.com/sqzer-x/sqzass
cd sqzass
cargo build --release
```

바이너리는 `target/release/sqzass`에 생깁니다.

## 요구사항

| 요구사항 | 버전 |
|---|---|
| Rust | 1.97 이상 |
| 그 외 | — |

## 미리 빌드된 바이너리

[릴리스](https://github.com/sqzer-x/sqzass/releases)마다 정적 링크된
`x86_64-unknown-linux-musl` 빌드와 `aarch64-apple-darwin` 빌드가 붙습니다. 각각
`.sha256`이 옆에 있습니다.

```bash
curl -LO https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-v0.1.0-x86_64-unknown-linux-musl.tar.gz
tar xzf sqzass-*.tar.gz
sudo install -m755 sqzass-*/sqzass /usr/local/bin/
```

리눅스 빌드는 정적이라 glibc를 요구하지 않고, 빌드한 배포판보다 오래된 배포판에서도
돕니다. 이 프로젝트가 syntect를 순수 Rust 정규식 엔진으로 고정한 이유가 이것입니다.
기본값은 C 바인딩이고, C 바인딩은 정확히 musl 정적 빌드를 깨뜨립니다.

> [!NOTE]
> 아직 릴리스가 없습니다 — 워크플로는 있고 첫 태그를 아직 붙이지 않았습니다.
> 그때까지는 소스에서 빌드하세요.

## 설치 확인

```bash
sqzass --version
```

다음은 [첫 사이트 만들기](@/start/first-site.md)입니다.
