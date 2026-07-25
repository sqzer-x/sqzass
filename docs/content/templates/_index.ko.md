+++
title = "템플릿"
description = "Jinja2 호환 템플릿, 엄격한 데이터 모델, 명시적인 선택 규칙"
weight = 30
sort_by = "weight"
+++

템플릿은 `templates/`에 두며 [minijinja]입니다. Jinja2 문법이라 `{% extends %}`,
`{% block %}`, `{% include %}`, `{% macro %}`와 필터가 아는 그대로 동작합니다.

```
templates/
├── base.html
├── page.html
├── section.html
└── partials/
    ├── sidebar.html
    └── toc.html
```

## 익숙한 것보다 엄격한 지점이 둘 있습니다

**정의되지 않은 값을 읽으면 에러입니다.** front matter 키 이름을 바꾸면, 구멍이
뚫린 페이지를 렌더하는 대신 템플릿 이름과 키 이름을 들고 빌드가 멈춥니다. 가장
엄격한 설정은 아닙니다. 정의된 적 없는 값에 대한 `{% if optional %}`은 그대로
동작하는데, "이게 있나?"를 물을 수 없는 템플릿으로는 페이지마다 생김새가 다른
사이트를 만들 수 없기 때문입니다.

**모든 값이 기본적으로 이스케이프됩니다.** `page.content`는 이미 HTML이므로
`| safe`가 필요합니다. `| safe`를 의도적으로 한 번 쓰는 대신, 나머지 모든 값이
아무 생각 없이도 안전해집니다.

## 빌드마다 스냅샷

`templates/`는 빌드 시작 때 한 번 읽고, `include`/`extends`는 그 스냅샷에서
해석됩니다. 개발 서버가 지켜보는 중에 파셜을 저장해도, 그때 일어난 리빌드는 일관된
한 벌의 파일만 봅니다. 반쯤 쓰인 파일을 보는 일이 없습니다.

[minijinja]: https://github.com/mitsuhiko/minijinja
