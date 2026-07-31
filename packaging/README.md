# 배포 채널

셋만 지원한다. **curl**, **cargo**, **AUR**이다.

| 채널 | 무엇 | 어디 |
|---|---|---|
| curl | `install.sh` 한 줄, 그리고 릴리스 타르볼을 직접 받는 길 | `docs/static/install.sh`, `.github/workflows/release.yml` |
| cargo | `cargo install --git`, 소스 빌드 | 별도 파일 없음 |
| AUR | 소스에서 짓는 `sqzass` 패키지 | `aur/` |

셋을 고른 기준은 하나다. **승인받을 사람이 없고, 우리가 고치면 그날 반영된다.**
셋 다 대상이 겹치지 않는다 — curl은 CI와 서버, cargo는 Rust를 이미 가진 사람,
AUR은 Arch 사용자다.

## 안 하기로 한 것

셋 다 "안 된다"가 아니라 "지금 할 값이 아니다"이다. 근거를 적어 두는 건
같은 조사를 두 번 하지 않기 위해서다.

**Homebrew.** homebrew-core는 notability 기준이 있고 이 레포는 스타·포크·워처가
전부 0이라 지금 내면 거절된다. 자기 탭은 승인이 필요 없어 오늘도 되지만, 릴리스가
내는 macOS 타깃이 `aarch64-apple-darwin` 하나뿐이라 바이너리 formula로는 인텔 맥이
깨지고 소스 빌드 formula여야 한다. 그 formula가 주는 건 `cargo install`과 거의
같다.

**Snap.** strict confinement + `home` + `network-bind`로 기술적으로는 된다 —
이 도구는 홈 디렉터리 조회도, `Command::new`도, 런타임 HTTP 클라이언트도 없는
얌전한 CLI다. 안 하는 이유는 관객이다. snap의 관객은 Ubuntu 데스크톱이고 이
도구의 사용자는 이미 curl·cargo·AUR을 가진 개발자다. 배포 문서 1순위(weight 10)가
`ubuntu-latest`에서 `cargo run`으로 도는 GitHub Pages라 snapd가 낄 자리가 없다.
비용은 반대로 확실하다: `snapcraft.yaml`은 `./`·`snap/`·`build-aux/snap/`에만
놓을 수 있어 이 디렉터리 관례를 깨야 하고, 이름 등록이 수동 심사이며, 스토어
토큰이 1년마다 만료된다. 무엇보다 **개발 머신이 Arch라 snapd도 LXD도 없어서
검증할 수 없다** — 이 레포는 눈으로 확인하기 전에 완료라고 하지 않는다.

**Nix.** 기술적으로는 오히려 잘 맞는다. `Cargo.lock`이 커밋돼 있고 락에 git
소스가 0건이라 `cargoLock.lockFile` 한 줄로 벤더링이 끝나고, 네트워크 없는
네임스페이스에서 `cargo test --release --offline --locked`가 147개 전부
통과한다(Nix 샌드박스가 요구하는 조건을 이미 만족한다는 뜻이다). `ldd`가
libc·libm·libgcc_s만 말해서 `buildInputs`도 빈다. 그런데도 안 하는 이유는 둘이다.
**개발 머신에 nix가 없어 `nix build`를 돌려 볼 수 없고**, Nix 사용자가 실제로
있다는 신호가 아직 없다. nixpkgs는 별개로 더 무겁다 — 머지되면 버전 범프와
MSRV 파손 대응을 영구히 떠안는다.

## 다시 꺼낼 조건

"있으면 좋겠다"는 근거가 아니다. 이런 게 근거다.

- 그 생태계 사용자가 **요청한** 이슈나 PR
- 검증할 환경이 생겼을 때 — Nix는 nix 있는 머신, Snap은 Ubuntu VM
- Homebrew core는 notability를 넘겼을 때

늘릴 때 드는 진짜 비용은 파일 하나가 아니라 릴리스마다 반복되는 절차와, 그
채널에서만 나오는 질문에 답하는 시간이다.
