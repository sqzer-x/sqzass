+++
title = "템플릿 데이터"
description = "템플릿이 읽을 수 있는 모든 것"
weight = 20
toc = true
+++

모든 페이지에서 `site`와 `page` 두 객체를 쓸 수 있습니다.

## site

| | |
|---|---|
| `site.title` | `sqzass.toml`에서. |
| `site.description` | `sqzass.toml`에서. |
| `site.base_url` | 끝의 슬래시를 뗀 값이라 `{{ site.base_url }}{{ page.url }}`이 항상 맞습니다. |
| `site.language` | 지금 렌더 중인 페이지의 언어. |
| `site.sections` | **이 언어의** 최상위 섹션들. |
| `site.highlight_css` | 생성된 하이라이트 스타일시트 URL. 강조가 꺼져 있으면 없습니다. |

`site.sections`에는 현재 언어의 트리만 담깁니다. 내비게이션이 안전한 이유가
이것입니다. 번역되지 않은 페이지는 여기 없으므로 그 링크를 그릴 수가 없습니다.
각 섹션은 `title`, `description`, `url`, `weight`, `pages`, `subsections`를 갖고,
`pages`의 각 항목은 `title`, `description`, `url`, `weight`를 갖습니다.

## page

| | |
|---|---|
| `page.title` | |
| `page.description` | |
| `page.url` | `/ko/start/installation/` |
| `page.permalink` | `base_url` + `url`. |
| `page.content` | 렌더된 HTML. **`\| safe`가 필요합니다.** |
| `page.weight`, `page.draft`, `page.language` | front matter 그대로. |
| `page.toc` | 저자가 목차를 원했는지. |
| `page.toc_entries` | 목차 자체 — `{level, id, title, children}`, 중첩된 형태. |
| `page.translations` | 이 페이지가 존재하는 언어만. 비어 있으면 전환 UI를 그리지 않으면 됩니다. |
| `page.section` | 이 페이지가 속한 섹션. 최상위 페이지에는 없습니다. |
| `page.children` | 섹션의 자식 페이지들. 일반 페이지에서는 비어 있습니다. |
| `page.is_section` | |
| `page.extra` | 당신의 `[extra]` 테이블. |

## asset()

`asset("css/main.css")`은 그 파일이 실제로 쓰인 해시 붙은 URL을 돌려줍니다.

```html
<link rel="stylesheet" href="{{ asset("css/main.css") }}">
```

수집되지 않은 파일을 요청하면 에러입니다. 스타일시트 이름을 바꿨을 때 모든
방문자에게 조용히 404를 내는 대신 빌드가 실패합니다.

## 슬래시는 이스케이프하지 않습니다

Jinja2는 다섯 글자를 이스케이프합니다. 일부 포팅 구현은 `/`까지 이스케이프하는데,
그러면 모든 페이지의 모든 URL이 `href="https:&#x2f;&#x2f;…"`가 됩니다. sqzass는
Jinja2 본래의 동작을 되돌려 두었으므로 URL이 URL로 나옵니다.

## 없는 키는 빌드를 멈춥니다

```
undefined value: page.descriptoin
```

설명이 있어야 할 자리에 빈 문자열이 들어가는 대신입니다.
[템플릿](@/templates/_index.md)을 참고하세요.
