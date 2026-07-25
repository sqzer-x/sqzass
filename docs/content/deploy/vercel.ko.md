+++
title = "Vercel"
description = "vercel.json, 그리고 프레임워크 기능들이 여기서 무의미한 이유"
weight = 30
toc = true
+++

```json
{
  "buildCommand": "curl -sSL https://github.com/sqzer-x/sqzass/releases/latest/download/sqzass-x86_64-unknown-linux-musl.tar.gz | tar xz --strip-components=1 && ./sqzass build",
  "outputDirectory": "public",
  "framework": null
}
```

`"framework": null`이 중요합니다. Vercel의 감지는 `package.json`을 찾는데, 없으면
추측을 합니다. 잘못 추측하면 빌드가 실패하고, 그 실패가 sqzass 탓처럼 읽힙니다.

## 프리뷰 배포

Vercel은 프리뷰 호스트를 스킴 없이 `$VERCEL_URL`로 줍니다.

```json
{
  "buildCommand": "… && ./sqzass build --base-url \"https://$VERCEL_URL\""
}
```

이유는 다른 호스트와 같습니다. 내부 링크는 루트 절대 경로라 어디서든 동작하지만,
canonical·sitemap·소셜 태그는 절대 URL이라 프리뷰에서 프로덕션을 가리키게 됩니다.

## 끝 슬래시

sqzass는 `/start/index.html`을 쓰므로 `/start/`가 정본이고, 생성하는 모든 링크가
그 형태입니다. Vercel의 기본 `trailingSlash` 동작은 둘 사이를 리다이렉트하므로
손으로 `/start`라고 쓴 링크에는 한 번의 왕복이 더 듭니다. `"trailingSlash": true`로
두면 그 왕복이 없어집니다.

필수가 아니라 취향입니다. 어느 쪽이든 사이트는 옳게 동작합니다.

## 필요 없는 것

서버리스 함수도, ISR도, 엣지 설정도, 이미지 최적화도 없습니다. sqzass는 요청 시점에
실행할 게 없으므로, Vercel을 파일 서버와 구별되게 하는 부분이 전부 여기서는 놀고
있습니다.

이건 분명히 말해 둘 만합니다. 팀이 이미 Vercel을 쓰고 있다면 잘 됩니다. sqzass
사이트를 위해 호스트를 **고르는** 중이라면, 프레임워크 지원이 선택 기준이 되지는
않을 겁니다.
