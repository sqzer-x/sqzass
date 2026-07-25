+++
title = "함수와 필터"
description = "템플릿이 부를 수 있는 전부, 그리고 그 밖엔 없다는 마지막 줄"
weight = 40
toc = true
+++

## sqzass가 제공하는 것

함수 둘. 이게 전부입니다.

### asset(path)

정적 파일이 실제로 쓰인 URL을 돌려줍니다. 해시와 서브경로가 포함됩니다.

```html
<link rel="stylesheet" href="{{ asset("css/main.css") }}">
<script src="{{ asset("js/search.js") }}" defer></script>
```

인자는 `static/` 기준 논리 경로이고, 앞의 슬래시는 있어도 없어도 됩니다. 수집되지
않은 이름을 요청하면 수집된 이름 전부를 나열하는 빌드 에러가 납니다. 스타일시트
이름을 바꿨을 때 모든 방문자에게 404를 내는 대신 빌드가 실패합니다.

### t(key)

지금 렌더 중인 페이지의 언어로 `i18n/<언어>.toml`에서 UI 문자열을 찾습니다.

```html
<a class="skip" href="#content">{{ t("skip_to_content") }}</a>
```

인자는 하나입니다. 언어는 절대 넘기지 않는데, 그게 왜 의도된 것인지는
[언어](@/content/languages.md)에 적어 두었습니다. 현재 언어에 키가 없으면 빌드
에러입니다.

> [!WARNING]
> 루프 변수 이름을 `t`로 두지 마세요. `{% for t in page.translations %}`는 그 블록
> 안에서 함수를 가리고, 거기에 라벨을 처음 넣는 사람이 원인에서 멀리 떨어진 에러를
> 받게 됩니다.

## minijinja가 주는 것

표준 Jinja2 문법이 동작합니다. `{% if %}`, `{% for %}`, `{% extends %}`,
`{% block %}`, `{% include %}`, `{% macro %}`, `{% from … import … %}`,
`{% set %}`, 그리고 익숙한 필터들 — `safe`, `escape`, `length`, `join`,
`default`, `upper`, `lower`, `replace`, `trim`, `first`, `last`, `reverse`,
`sort`, `map`, `select`, `selectattr`, `batch`, `slice`, `int`, `float`, `abs`,
`round`, `urlencode`, `striptags`, `indent`.

이 사이트의 템플릿이 `{% macro %}`와
`{% from "partials/sidebar.html" import nav %}`를 쓰므로, 그 둘은 빌드마다 실제로
돌고 있습니다.

`tojson`은 **없습니다.** minijinja의 `json` 피처가 꺼져 있고, `{% for %}` 루프로
이미 되는 일을 위해 의존성을 켜지는 않습니다.

## 그 밖엔 없습니다

`url_for`도, `markdownify`도, `date`도, `now()`도, `env()`도, 커스텀 테스트도
없습니다. 그중 일부는 빈자리가 아니라 근거가 있는 부재입니다.

`now()`와 `env()`는 "같은 입력이면 같은 바이트"라는 보장을 깹니다. CI가 push마다
확인하는 그 보장입니다. 시계를 읽을 수 있는 템플릿은 재현 가능할 수 없습니다.

`url_for`는 이미 한 줄인 비교를 감싸는 것입니다. 페이지와 섹션 URL은 바로 쓸 수
있는 형태로 오고, 서브경로도 이미 붙어 있습니다.

없는 이름을 읽으면 빈 문자열이 아니라 빌드 에러입니다. 다른 생성기의 습관으로
무언가를 불렀다면 구멍 난 페이지가 아니라 빌드 시점에 알게 됩니다.

## 현재 페이지 표시하기

이걸 위한 헬퍼는 없고, 필요하지도 않습니다. 페이지는 정확히 일치로:

```html
<a href="{{ p.url }}"{% if p.url == page.url %} aria-current="page"{% endif %}>{{ p.title }}</a>
```

섹션은 `page.section`으로 조상 일치를:

```html
<a href="{{ s.url }}"{% if page.section and page.section.url == s.url %} aria-current="true"{% endif %}>{{ s.title }}</a>
```

접두사 검사에는 `startingwith`를 쓰되 루트를 조심하세요. 모든 URL이 `/`로
시작하므로, 홈 링크는 반드시 정확히 일치로 비교해야 하고 조상으로 봐서는 안 됩니다.
