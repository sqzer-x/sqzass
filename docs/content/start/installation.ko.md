+++
title = "설치"
description = "소스에서 sqzass를 빌드한다"
weight = 10
toc = true
+++

sqzass는 런타임 의존성이 없는 단일 바이너리다. Node도, Python도, 시스템
라이브러리도 필요 없다.

## 소스에서 빌드

```bash
git clone https://github.com/sqzer-x/sqzass
cd sqzass
cargo build --release
```

바이너리는 `target/release/sqzass`에 생긴다.

## 요구사항

| 요구사항 | 버전 |
|---|---|
| Rust | 1.97 이상 |
| 그 외 | — |

> [!NOTE]
> 미리 빌드된 바이너리와 AUR 패키지는 계획에 있지만 아직 배포하지 않았다.

## 설치 확인

```bash
sqzass --version
```
