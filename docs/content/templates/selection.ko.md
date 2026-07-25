+++
title = "템플릿 선택"
description = "네 단계, 순서대로, 캐스케이드 없이"
weight = 10
toc = true
+++

페이지는 아래 중 **가장 먼저 존재하는** 것으로 렌더됩니다.

1. 페이지 front matter의 `template`
2. 페이지가 섹션 인덱스면 `section.html`
3. 부모 섹션 `_index.md`의 `page_template`
4. `page.html`

규칙은 이게 전부입니다. 존재하지 않는 `template`을 지정하면 에러이고, 메시지에
지금 갖고 있는 템플릿 목록이 같이 나옵니다.

## 왜 스무 단계가 아니라 네 단계인가

Hugo는 kind × section × type × layout × language × output format을 곱해 만든
lookup order로 템플릿을 찾습니다. 더 강력하고, 동시에 Hugo 사용자가 가장 많이
헤매는 바로 그 지점입니다. "어느 템플릿이 선택됐는지 출력이라도 해 달라"는 요청이
10년째 열려 있습니다.

머릿속에 담기는 네 단계에는 그런 명령이 필요 없습니다. 어느 템플릿이 이 페이지를
렌더했는지 알 수 없다면, 그 규칙은 너무 복잡한 것입니다.

## 섹션 기본값 지정하기

```toml
# content/blog/_index.md
+++
title = "블로그"
page_template = "post.html"
+++
```

이제 `content/blog/`의 페이지들은 자기 것을 지정하지 않는 한 `post.html`로
렌더됩니다. 직속 자식에게만 적용되며, 하위 섹션은 자기 것을 따로 정합니다.

## base 상속하기

보통은 스켈레톤 하나에 얇은 템플릿을 얹는 구성을 씁니다.

```html
{# templates/base.html #}
<!doctype html>
<html lang="{{ page.language }}">
<head><title>{{ page.title }}</title></head>
<body>{% block content %}{% endblock %}</body>
</html>
```

```html
{# templates/page.html #}
{% extends "base.html" %}
{% block content %}<article class="prose">{{ page.content | safe }}</article>{% endblock %}
```
