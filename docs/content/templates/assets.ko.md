+++
title = "정적 에셋"
description = "복사, 해시, 그리고 원래 이름으로 하는 조회"
weight = 30
toc = true
+++

`static/` 아래의 모든 파일이 경로를 유지한 채 출력으로 복사됩니다.

```
static/
├── css/main.css   →  /css/main.a1b2c3d4.css
├── js/search.js   →  /js/search.e5f6a7b8.js
├── images/x.png   →  /images/x.png
└── CNAME          →  /CNAME
```

CSS와 JavaScript는 파일명에 콘텐츠 해시가 붙습니다. 조회는 원래 쓴 이름으로
합니다.

```html
<link rel="stylesheet" href="{{ asset("css/main.css") }}">
<script src="{{ asset("js/search.js") }}" defer></script>
```

## 쿼리 문자열이 아니라 파일명에

`main.css?v=123`에는 문제가 둘 있습니다. 일부 CDN과 프록시는 캐시 키에서 쿼리를
빼 버려서 무효화가 일어나지 않습니다. 그리고 흔히 쓰이는 형태, 즉 빌드마다 스탬프
하나를 전 파일에 붙이는 방식은 CSS 한 줄만 고쳐도 사이트 전체를 무효화합니다.

파일별 콘텐츠 해시는 바뀐 파일만 정확히 무효화하고, 캐시 설정이 아예 없는 호스트를
포함해 어디서나 똑같이 동작합니다.

## CSS와 JS만 해시하는 이유

이미지는 보통 경로를 그대로 적어 참조합니다. 마크다운에서, CSS에서, 때로는 직접 쓰지
않은 템플릿에서요. 이름을 바꾸면 그 참조들이 깨지는데 고칠 방법이 없습니다.

그리고 어떤 파일명은 그 자체가 계약입니다. `CNAME`은 GitHub Pages에 도메인을
알려주고, `robots.txt`는 이름으로 찾아집니다. `CNAME.9f8e7d6c`는 아무도 읽지 않는
파일입니다.

## 생성된 에셋도 같은 길을 지납니다

하이라이트 스타일시트는 복사되는 게 아니라 테마에서 만들어지지만, 해시와 서빙은
동일합니다. 템플릿에서는 `site.highlight_css`로 가져옵니다.

## 끄기

```toml
[assets]
source_dir  = "static"
fingerprint = false
```

`fingerprint = false`면 파일명이 그대로 유지되고 `asset()`은 계속 동작합니다.
템플릿을 고치지 않고 켜고 끌 수 있습니다.

## asset-manifest.json

빌드마다 출력 루트에 하나 쓰입니다. 논리 이름과 실제 쓰인 URL의 대응표입니다.

```json
{
  "css/main.css": "/css/main.a1b2c3d4.css",
  "CNAME": "/CNAME"
}
```

해시가 붙은 것만이 아니라 모든 정적 파일이 들어가고, `fingerprint = false`여도
쓰입니다. sqzass는 이걸 다시 읽지 않습니다. 빌드 **밖**의 무언가가 `asset()`이
빌드 안에서 답하는 질문에 똑같이 답할 수 있으라고 있는 파일입니다 — 서비스 워커,
배포 스크립트, 캐시 예열기 같은 것들.

무시해도 됩니다. 다만 정리 단계에서 지운 뒤에 이걸 쓰던 도구가 왜 멈췄는지
찾아 헤매는 건 곤란하고, 이 문단이 그걸 막으려고 있습니다.

## 검색 색인은 해시하지 않습니다

`/search-en.json`과 `/search-ko.json`은 고정된 이름을 씁니다. `<head>`에서 링크되는
게 아니라 스크립트가 가져가고, 내용이 렌더된 페이지에서 나오는데 페이지는 에셋이
정해진 **뒤에** 렌더되므로 해시를 붙이면 순환이 됩니다. 색인이 10분쯤 낡는 대가는
검색 결과 몇 줄입니다. [검색](@/features/search.md)을 참고하세요.
