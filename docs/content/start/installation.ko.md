+++
title = "설치"
description = "한 줄로, 미리 빌드된 바이너리로, 소스에서"
weight = 10
toc = true
+++

sqzass는 런타임 의존성이 없는 단일 바이너리입니다. Node도, Python도, 시스템
라이브러리도 필요 없습니다.

## 한 줄

```bash
curl -fsSL https://sqzass.sqzer.com/install.sh | sh
```

스크립트가 하는 일은 정확히 넷이고, 실행 전에 [직접 읽어볼](/install.sh) 수
있습니다: 플랫폼 판별(x86_64 또는 ARM64 리눅스, Apple Silicon macOS), 최신 릴리스
타르볼과 `.sha256` 다운로드, 체크섬 검증, `/usr/local/bin`에 `install` — sudo는
그 디렉터리에 쓸 수 없을 때만 묻습니다. 다른 곳에 설치하려면
`SQZASS_INSTALL_DIR`을 주세요. 그 밖의 일은 하지 않습니다 — 셸 설정도, `PATH`도
건드리지 않습니다.

## Cargo로

Rust가 이미 있다면 한 줄로 소스에서 빌드합니다.

```bash
cargo install --git https://github.com/sqzer-x/sqzass
```

## Arch Linux

AUR에 [`sqzass`](https://aur.archlinux.org/packages/sqzass)가 있습니다. 릴리스
태그에서 빌드합니다.

```bash
yay -S sqzass          # paru도, 클론해서 makepkg -si도 됩니다
```

`sqzass-bin`은 만들지 않았습니다. Arch에서 Rust 도구를 소스에서 빌드하는 건
정상이고, 패키지가 둘이면 어느 쪽이 최신인지 묻는 일만 생깁니다.

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
| C 컴파일러 | rustc가 링커 구동에 씁니다. 기본 빌드는 구문 강조의 정규식 엔진인 Oniguruma도 이걸로 컴파일합니다. |
| 그 외 | — |

정적 릴리스가 싣는 구성 그대로, 즉 순수 Rust 정규식 엔진에 C 소스 컴파일이
없는 상태도 플래그 하나면 됩니다.

```bash
cargo build --release --no-default-features --features pure-rust
```

## 미리 빌드된 바이너리

[릴리스](https://github.com/sqzer-x/sqzass/releases)마다 정적 링크된 리눅스 빌드 둘
(`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`)과
`aarch64-apple-darwin` 빌드가 붙습니다. 각각 `.sha256`이 옆에 있습니다.

```bash
curl -LO https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz
tar xzf sqzass-x86_64-unknown-linux-musl.tar.gz
sudo install -m755 sqzass-*/sqzass /usr/local/bin/
```

리눅스 빌드는 정적이라 glibc를 요구하지 않고, 빌드한 배포판보다 오래된 배포판에서도
돕니다. 타르볼 하나가 Debian에서도 Fedora에서도 Arch에서도 그대로 도는 이유가
이것입니다. 순수 Rust 정규식 엔진을 쓰는 빌드이기도 합니다 — 다른 모든 곳에서
sqzass는 코드가 많은 사이트에서 눈에 띄게 빠른 Oniguruma로 강조하는데, Oniguruma는
C 바인딩이고, musl 정적 빌드를 깨뜨리는 게 바로 C 바인딩입니다. 두 엔진의 문법 집합도
같지 않습니다. 순수 Rust 엔진이 일부 문법의 정규식을 못 돌리기 때문에 정적
아티팩트에는 문법 일곱 개가 통째로 빠집니다 — PowerShell, JavaScript (Babel),
Salt State, ARM Assembly 등이요. 네이티브 빌드가 온전히 칠하는 ` ```powershell `
이나 ` ```jsx ` 펜스가 거기서는 에러 없이 평문이 됩니다. ` ```js `
조차 마크업 구조가 다릅니다. 네이티브 빌드는 Babel 문법으로, 정적 빌드는 일반
JavaScript로 해석합니다. 각 바이너리 자체는 여전히 완전히 결정적입니다.

## 설치 확인

```bash
sqzass --version
```

다음은 [첫 사이트 만들기](@/start/first-site.md)입니다.
