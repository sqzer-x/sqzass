# AUR

소스에서 빌드하는 `sqzass` 패키지다. 미리 빌드된 바이너리를 받는 `sqzass-bin`은
만들지 않았다 — Rust 도구를 소스에서 빌드하는 건 Arch에서 정상이고, 패키지가 둘이면
어느 쪽이 최신인지 묻는 이슈가 생긴다.

`sha256sums`가 `SKIP`으로 되어 있다. **이대로 올리지 말 것.** 태그가 존재해야
체크섬을 계산할 수 있으므로 순서가 이렇다:

```bash
# 1. 릴리스 태그를 먼저 만든다
git tag v0.1.0 && git push origin v0.1.0

# 2. 체크섬을 채운다 (pacman-contrib)
cd packaging/aur && updpkgsums

# 3. 로컬에서 실제로 빌드되는지 본다
makepkg -si

# 4. .SRCINFO를 만든다. AUR은 이 파일로 패키지를 읽는다
makepkg --printsrcinfo > .SRCINFO
```

## 처음 올릴 때

AUR 계정과 SSH 키가 필요하다. 웹에서 해야 하는 일이라 여기 자동화하지 않았다.

```bash
ssh-keygen -t ed25519 -f ~/.ssh/aur          # 키를 만들고
# aur.archlinux.org 계정 설정에 공개키를 등록한 뒤
git clone ssh://aur@aur.archlinux.org/sqzass.git aur-sqzass
cp PKGBUILD .SRCINFO aur-sqzass/
cd aur-sqzass && git add -A && git commit -m 'Initial import' && git push
```

## 버전을 올릴 때

`pkgver`를 고치고 `pkgrel=1`로 되돌린 뒤 2~4를 반복한다. `pkgrel`은 같은 버전의
패키징만 고쳤을 때 올린다.
