#!/bin/sh
# sqzass 설치 스크립트 — 이 머신에 맞는 최신 릴리스 바이너리를 받아 설치한다.
#   curl -fsSL https://sqzass.sqzer.com/install.sh | sh
#
# 하는 일 전부: 플랫폼 판별 → 최신 태그 조회 → 타르볼과 .sha256 다운로드 →
# 체크섬 검증 → /usr/local/bin에 설치(SQZASS_INSTALL_DIR로 변경 가능).
# 그 밖의 일은 하지 않는다 — 셸 설정도, PATH도 건드리지 않는다.
set -eu

repo="sqzer-x/sqzass"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target="x86_64-unknown-linux-musl" ;;
  Darwin-arm64) target="aarch64-apple-darwin" ;;
  *)
    echo "지원하지 않는 플랫폼입니다: $(uname -s) $(uname -m)" >&2
    echo "소스에서 빌드하세요: https://sqzass.sqzer.com/start/installation/" >&2
    exit 1
    ;;
esac

# API가 아니라 releases/latest의 리다이렉트에서 태그를 읽는다 — API는
# 미인증 요청에 레이트리밋이 있어 공유 IP(프록시·CI)에서 403이 난다.
tag="$(curl -fsSLI -o /dev/null -w '%{url_effective}' \
  "https://github.com/$repo/releases/latest")"
tag="${tag##*/}"
case "$tag" in
  v*) ;;
  *) echo "최신 릴리스를 찾지 못했습니다." >&2; exit 1 ;;
esac

name="sqzass-$tag-$target"
url="https://github.com/$repo/releases/download/$tag/$name.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "받는 중: $url"
curl -fsSL -o "$tmp/$name.tar.gz" "$url"
curl -fsSL -o "$tmp/$name.tar.gz.sha256" "$url.sha256"

# 체크섬은 검증 도구가 있을 때만 건너뛰지 않는다 — 둘 다 없으면 중단이 정직하다.
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$tmp" && sha256sum -c "$name.tar.gz.sha256" >/dev/null)
elif command -v shasum >/dev/null 2>&1; then
  (cd "$tmp" && shasum -a 256 -c "$name.tar.gz.sha256" >/dev/null)
else
  echo "sha256sum도 shasum도 없어 체크섬을 검증할 수 없습니다." >&2
  exit 1
fi

tar -xzf "$tmp/$name.tar.gz" -C "$tmp"

dir="${SQZASS_INSTALL_DIR:-/usr/local/bin}"
if [ -d "$dir" ] && [ -w "$dir" ]; then
  install -m 755 "$tmp/$name/sqzass" "$dir/sqzass"
else
  echo "$dir 에 쓰려면 관리자 권한이 필요합니다."
  sudo install -m 755 "$tmp/$name/sqzass" "$dir/sqzass"
fi

echo "설치했습니다: $dir/sqzass ($("$dir/sqzass" --version))"
