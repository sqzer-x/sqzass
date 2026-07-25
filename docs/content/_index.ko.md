+++
title = "sqzass"
description = "Rust로 만든 정적 사이트 생성기"
weight = 0
+++

대부분의 생성기는 예쁜 URL·리다이렉트·캐시 헤더를 호스트에 맡긴다. sqzass는 그걸
직접 한다. 그래서 같은 출력물이 GitHub Pages든 Vercel이든 Cloudflare Pages든,
그냥 디렉터리를 HTTP로 서빙하든 똑같이 동작한다.

```bash
sqzass build -i docs
sqzass serve -i docs
```

## 그래서 얻는 것

**마크다운을 트리에서 변환한다.** 링크 재작성, heading anchor, 목차 모두
[comrak]의 AST 연산이다. 흔한 지름길인 "완성된 HTML에 정규식 돌리기"는 속성을
홑따옴표로 쓰거나 따옴표를 빼면 조용히 처리에서 빠지고, 그 사실을 프로덕션에서
알게 된다.

**템플릿이 반쯤 쓰인 파일을 읽을 수 없다.** `templates/`를 빌드마다 한 번
스냅샷으로 뜨고 [minijinja]의 로더가 `include`와 `extends`를 그 스냅샷에서
해석한다. 저장하는 도중에 리빌드가 걸려도 항상 일관된 상태를 본다.

**깨진 참조는 빌드를 멈춘다.** 해석 안 되는 `@/` 링크, 없는 템플릿, 같은 URL을
주장하는 두 페이지, 없는 에셋 이름 — 전부 경고가 아니라 에러다.

**같은 입력은 같은 바이트를 낸다.** 두 번 빌드하면 바이트까지 동일하고, CI가
push마다 확인한다.

**처음부터 이중 언어.** 번역은 자동으로 연결되고, 각 언어의 내비게이션에는 그
언어에 실제로 있는 페이지만 담긴다. 미번역 페이지가 죽은 링크를 남길 수 없다.

## 상태

초기 단계이고, 그 점을 숨기지 않는다. 구문 강조·내비게이션·목차·에셋
파이프라인·개발 서버는 동작한다. 검색과 피드, 테마는 아직 없다.
[시작하기](@/start/_index.md)부터 보라.

[comrak]: https://github.com/kivikakk/comrak
[minijinja]: https://github.com/mitsuhiko/minijinja
