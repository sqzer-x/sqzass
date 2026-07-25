+++
title = "첫 사이트 만들기"
description = "sqzass가 빌드하는 가장 작은 사이트를 한 파일씩"
weight = 20
toc = true
+++

아직 `sqzass init`이 없어서, 사이트는 직접 만드는 파일 세 개로 시작합니다.
그게 스캐폴드의 전부입니다. sqzass는 숨은 상태도, 자기가 관리한다고 가정하는
생성 디렉터리도 두지 않습니다.

## 파일 세 개

```
mysite/
├── sqzass.toml
├── content/
│   └── _index.md
└── templates/
    └── page.html
```

`sqzass.toml`에 반드시 있어야 하는 키는 둘입니다. 나머지는 전부 기본값이 있습니다.

```toml
title    = "내 사이트"
base_url = "https://example.com"
```

`content/_index.md`가 첫 페이지입니다. front matter는 `+++` 펜스 사이의 TOML이고,
반드시 적어야 하는 필드는 `title` 하나뿐입니다.

```markdown
+++
title = "내 사이트"
+++

안녕하세요.
```

`templates/page.html`이 이걸 렌더합니다. `page.content`는 이미 HTML이므로
`| safe`가 필요합니다. 템플릿은 기본적으로 이스케이프하며, 그 기본값 덕분에
본문에 섞인 `<` 하나가 마크업으로 둔갑하지 않습니다.

```html
<!doctype html>
<html lang="{{ page.language }}">
<head><meta charset="utf-8"><title>{{ page.title }}</title></head>
<body>{{ page.content | safe }}</body>
</html>
```

## 빌드하기

```bash
sqzass build -i mysite
```

결과물은 `mysite/public`에 생깁니다. 모든 페이지는 `<경로>.html`이 아니라
`<경로>/index.html`로 쓰이므로 URL이 `/about.html`이 아니라 `/about/`이 됩니다.
rewrite 규칙이 없는 호스트에서 왜 이게 중요한지는
[페이지와 섹션](@/content/_index.md)에 적어 두었습니다.

## 작업하기

```bash
sqzass serve -i mysite
```

<http://127.0.0.1:3000>에 메모리에서 사이트를 띄우고, 파일이 바뀌면 다시
빌드합니다. 도는 동안 `public/`에는 아무것도 쓰지 않으므로, 빌드 중인 사이트가
반쯤 쓰인 파일을 내보내는 일이 없습니다. 페이지는 스스로 새로고침하고,
CSS만 바뀐 변경은 스타일시트만 갈아 끼워 스크롤 위치를 유지합니다.

> [!NOTE]
> 개발 서버는 개발용 도구입니다. 캐싱도, 압축도, 접근 제어도 하지 않습니다.
> 프로덕션에서는 빌드된 디렉터리를 제대로 된 서버로 서빙하세요.

## 다음에 더할 것

페이지를 하나 더 만드는 건 파일을 하나 더 만드는 일입니다. `content/about.md`는
`/about/`이 되고, `content/guide/_index.md`는 그 옆의 파일들을 묶는 섹션을
시작합니다. 페이지가 가질 수 있는 필드는
[front matter](@/content/front-matter.md)에 정리해 두었습니다.
