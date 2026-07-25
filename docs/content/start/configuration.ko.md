+++
title = "설정"
description = "sqzass.toml의 모든 키와, 오타를 냈을 때 벌어지는 일"
weight = 30
toc = true
+++

`sqzass.toml`은 사이트 루트에 둡니다. 반드시 있어야 하는 키는 둘이고, 나머지는
전부 기본값이 있어 적지 않으면 그 기본값으로 동작합니다.

```toml
title    = "내 사이트"
base_url = "https://example.com"
```

## 오타는 에러입니다

```
error: sqzass.toml 파싱 실패: TOML parse error at line 8, column 1
  |
8 | theme_ligth = "InspiredGitHub"
  | ^^^^^^^^^^^
unknown field `theme_ligth`, expected one of `enabled`, `theme_light`, `theme_dark`
```

sqzass가 읽지 않는 키는 아무 일도 하지 않는 키이고, 조용히 아무 일도 하지 않는
설정은 깨진 링크와 같은 종류의 실패입니다. 무언가를 요청했고, 도구는 알겠다고 했고,
아무 일도 일어나지 않았습니다. 테마가 왜 안 바뀌는지 찾느라 오후를 쓰게 됩니다.

front matter도 마찬가지이고, 에러가 가리키는 줄 번호는 front matter 블록 기준이
아니라 **파일 기준**입니다.

## 사이트

| 키 | 기본값 | |
|---|---|---|
| `title` | — | **필수.** |
| `base_url` | — | **필수.** canonical URL, sitemap, `robots.txt`가 씁니다. |
| `description` | `""` | |
| `default_language` | `"en"` | 이 언어가 루트를 쓰고, 나머지는 URL 접두사를 갖습니다. |

## `[languages.<코드>]`

```toml
[languages.en]
name   = "English"
weight = 1

[languages.ko]
name   = "한국어"
weight = 2
```

`name`은 언어 전환 UI에 보이는 이름이고, `weight`는 순서입니다. 언어를 아예
선언하지 않아도 됩니다. 그러면 `default_language` 하나로 도는 단일 언어 사이트가
됩니다. [언어](@/content/languages.md)를 참고하세요.

## `[build]`

| 키 | 기본값 | |
|---|---|---|
| `output_dir` | `"public"` | 사이트 루트 기준. |
| `drafts` | `false` | 명령줄의 `--drafts`가 이걸 켭니다. |

## `[markdown]`

| 키 | 기본값 | |
|---|---|---|
| `footnotes` | `true` | |
| `tables` | `true` | |
| `tasklist` | `true` | |
| `strikethrough` | `true` | |
| `autolink` | `true` | |
| `alerts` | `true` | GitHub의 `> [!NOTE]` 콜아웃. |
| `cjk_friendly_emphasis` | `true` | **한국어라면 켜 두세요.** |
| `heading_anchors` | `"right"` | `none`, `left`, `right`. |

`cjk_friendly_emphasis`가 `**강조**한다`를 강조로 파싱되게 하는 키입니다.
CommonMark의 flanking 규칙은 단어 사이에 공백을 두는 언어를 전제로 하고, 이게
없으면 굵게 표시한 부분 바로 뒤에 조사가 붙은 형태는 강조가 아예 아닙니다. 끄면
한국어 본문이 "내 마크다운이 틀렸나" 싶은 모양으로 깨집니다.
[마크다운](@/content/markdown.md)을 참고하세요.

## `[highlight]`

| 키 | 기본값 | |
|---|---|---|
| `enabled` | `true` | |
| `theme_light` | `"InspiredGitHub"` | |
| `theme_dark` | `"base16-ocean.dark"` | |

없는 테마 이름을 쓰면 에러이고, 메시지에 쓸 수 있는 이름들이 같이 나옵니다.
[구문 강조](@/features/highlighting.md)를 참고하세요.

## `[assets]`

| 키 | 기본값 | |
|---|---|---|
| `source_dir` | `"static"` | |
| `fingerprint` | `true` | CSS와 JS의 파일명에 콘텐츠 해시를 붙입니다. |

[정적 에셋](@/templates/assets.md)을 참고하세요.

## `[nav]`

| 키 | 기본값 | |
|---|---|---|
| `sort_by` | `"weight"` | `weight` 또는 `title`. 섹션이 각자 덮어쓸 수 있습니다. |

## 명령줄에서 덮어쓰기

`--base-url`과 `--drafts`는 파일보다 우선합니다. 설정 하나로 프리뷰 배포와
프로덕션을 함께 다룰 수 있는 이유입니다.

```bash
sqzass build -i mysite --base-url https://preview.example.com --drafts
```
