+++
title = "언어"
description = "하나의 콘텐츠 트리로 두 언어를, 미번역 페이지는 감춘 채"
weight = 30
toc = true
+++

언어는 `sqzass.toml`에 선언합니다. 기본 언어가 루트를 쓰고, 나머지는 접두사를 갖습니다.

```toml
default_language = "en"

[languages.en]
name   = "English"
weight = 1

[languages.ko]
name   = "한국어"
weight = 2
```

`/start/`가 영어, `/ko/start/`가 한국어입니다.

## 파일명 접미사

```
content/start/
├── installation.md        →  /start/installation/
└── installation.ko.md     →  /ko/start/installation/
```

두 파일이 나란히 놓이므로 `ls` 한 번이면 무엇이 아직 번역되지 않았는지 보입니다.
`content.ko/` 같은 병렬 트리를 두면 그게 diff 뒤로 숨습니다.

## 미번역 페이지는 복제하지 않고 감춥니다

한국어판이 없는 페이지는 한국어 내비게이션에 아예 나타나지 않습니다. 나머지 두
선택지는 둘 다 더 나쁩니다. 한국어 URL에 영어 본문을 렌더하면 검색 엔진 입장에서
중복 콘텐츠가 되고, 404를 내면 사이트가 스스로 그린 링크로 독자를 막다른 길에
보내는 셈입니다.

템플릿에는 `page.translations`가 넘어가는데, 여기엔 이 페이지가 **실제로 존재하는**
언어만 담깁니다. 그래서 언어 전환 UI는 동작하는 선택지만 그릴 수 있습니다.

```html
{% for t in page.translations %}
<a href="{{ t.url }}" hreflang="{{ t.code }}">{{ t.name }}</a>
{% endfor %}
```

목록이 비면 전환 버튼도 없습니다. 버튼이 거짓말을 하는 상태가 생기지 않습니다.

## 번역을 짝짓는 방법

파일명 stem으로 짝짓습니다. `installation.md`와 `installation.ko.md`는 한 페이지의
두 언어판입니다. 파일명이 달라야 할 때 — 슬러그를 현지화하는 경우 등 — 는 양쪽
파일에 같은 `translation_key`를 주면 됩니다.

```toml
# content/start/installation.md
+++
title = "Installation"
translation_key = "install"
+++
```

```toml
# content/start/설치.ko.md
+++
title = "설치"
translation_key = "install"
+++
```

## UI 문자열

페이지의 글은 `content/`에 있습니다. **템플릿이** 넣는 말 — "본문으로 건너뛰기",
"이 페이지", "이전" — 은 `i18n/<코드>.toml`에 둡니다.

```toml
# i18n/en.toml
home         = "Home"
on_this_page = "On this page"
```

```toml
# i18n/ko.toml
home         = "홈"
on_this_page = "이 페이지"
```

```html
<a href="{{ home }}">{{ t("home") }}</a>
```

`t`는 지금 렌더 중인 페이지에서 언어를 읽습니다. 그래서 템플릿은 자기가 어느
언어인지 묻지 않아도 되고, 알려 주는 걸 깜빡할 수 있는 자리도 생기지 않습니다.

**어느 한 언어에 키가 없으면 에러**이고, 어느 언어에는 있는지도 알려 줍니다.

```
번역 키 'next' 가 i18n/ko.toml 에 없습니다 (en 에는 있습니다) (in page.html:22)
```

기본 언어로 떨어뜨리면 한국어 페이지 안에 영어 라벨이 들어갑니다. 한국어 독자에게는
전부 보이고 사이트를 관리하는 사람에게는 보이지 않는데, 미번역 페이지를 복제하지
않고 감추는 것과 정확히 같은 이유입니다.

`i18n/` 디렉터리가 없는 사이트도 잘 돕니다. `t`는 그걸 부르는 템플릿에만 필요합니다.

## 한국어에서 알아 둘 것

알아서 처리되지만 알고는 있어야 하는 게 둘 있습니다. 둘 다 어긋났을 때 조용하기
때문입니다.

**`**강조**한다`가 강조로 파싱됩니다.** CommonMark의 flanking 규칙은 단어 사이에
공백을 두는 언어를 전제로 쓰였고, 그 규칙대로면 `**굵게**` 바로 뒤에 조사가 붙은
형태는 강조가 아예 아닙니다. sqzass는 comrak의 `cjk_friendly_emphasis`를 켜 두기
때문에 한국어로 자연스럽게 쓴 마크다운이 그대로 동작합니다. `[markdown]`의 키이며,
이걸 끄면 한국어 본문이 "내 마크다운이 틀렸나" 싶은 모양으로 깨집니다.

**한국어 제목은 한글 id를 유지합니다.** `## 설치`는 `id="설치"`가 되므로 로마자
변환 없이 앵커와 목차가 동작합니다.
