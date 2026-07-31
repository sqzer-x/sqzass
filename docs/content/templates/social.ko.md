+++
title = "소셜 카드와 구조화 데이터"
description = "이미 있는 값으로 직접 넣는 마크업"
weight = 50
toc = true
+++

sqzass는 `<head>`에 아무것도 주입하지 않습니다. 주입 지점이 없고 앞으로도 만들지
않을 것입니다. 조용히 태그를 더하는 생성기는 출력물을 끝까지 읽을 수 없게 만드는
생성기니까요. 아래는 전부 `base.html`에 직접 넣는 마크업이고, 재료는 이미
템플릿 컨텍스트에 있는 값들입니다.

## OpenGraph와 Twitter

이게 없으면 사이트의 모든 페이지가 Slack·Discord·카카오톡에서 맨 URL로 나옵니다.

```html
<meta property="og:type" content="{{ "website" if page.is_section else "article" }}">
<meta property="og:site_name" content="{{ site.title }}">
<meta property="og:title" content="{{ page.title }}">
<meta property="og:url" content="{{ page.permalink }}">
<meta property="og:locale" content="{{ "ko_KR" if page.language == "ko" else "en_US" }}">
{%- if page.description %}
<meta property="og:description" content="{{ page.description }}">
{%- endif %}
<meta name="twitter:card" content="summary">
```

`summary_large_image`가 아니라 `summary`입니다. large 쪽은 이미지가 있어야 뜻이
있고, 이미지 없이 선언하면 더 예쁜 카드가 아니라 빈 상자가 나옵니다. 페이지마다
이미지가 있다면 front matter에 넣고 바꾸면 됩니다.

```toml
+++
title = "설치"
[extra]
image = "/images/install.png"
+++
```

```html
{%- if page.extra.image %}
<meta property="og:image" content="{{ site.origin }}{{ page.extra.image }}">
<meta name="twitter:card" content="summary_large_image">
{%- else %}
<meta name="twitter:card" content="summary">
{%- endif %}
```

`og:image`는 절대 URL이어야 합니다. `site.origin`이 그 자리에 있는 이유입니다.

## JSON-LD 브레드크럼

문서 사이트의 구글 검색 결과를 눈에 띄게 바꾸는 구조화 데이터는 이것 하나입니다.
결과에 맨 URL 대신 `홈 › 콘텐츠 작성 › front matter`가 뜹니다.

```html
{%- if page.section %}
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  "itemListElement": [
    {"@type": "ListItem", "position": 1, "name": "{{ site.title }}", "item": "{{ site.origin }}{{ site.base_path }}/"},
    {"@type": "ListItem", "position": 2, "name": "{{ page.section.title }}", "item": "{{ site.origin }}{{ page.section.url }}"},
    {"@type": "ListItem", "position": 3, "name": "{{ page.title }}", "item": "{{ page.permalink }}"}
  ]
}
</script>
{%- endif %}
```

`{%- if page.section %}`를 눈여겨보세요. 최상위 페이지에는 섹션이 없고, 단이 하나
단이 하나 빠진 브레드크럼은 없느니만 못합니다.

> [!WARNING]
> 이 블록은 `<script>` 안이라 HTML 이스케이프가 맞지 않습니다. 제목에 `"`가 들어가면
> 깨진 JSON이 나옵니다. 제목에 따옴표를 쓰지 않거나, 조용히 파싱에 실패하는 JSON을
> 내보내느니 구조화 데이터를 빼세요. 템플릿에서 우리 이스케이프가 지켜 주지 못하는
> 유일한 자리입니다.

## 사이트 단위

```html
<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "WebSite",
  "name": "{{ site.title }}",
  "url": "{{ site.origin }}{{ site.base_path }}/"
}
</script>
```

모든 페이지에 이 블록 하나면 충분합니다. 검색 엔진은 한 번만 읽습니다.
