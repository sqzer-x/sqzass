+++
title = "명령줄"
description = "명령과 플래그, exit code, 기계가 읽는 출력"
weight = 40
toc = true
+++

```
sqzass init  [DIR]
sqzass build [-i DIR] [-o DIR] [--drafts] [--base-url URL]
sqzass serve [-i DIR] [-b ADDR] [-p PORT] [--drafts] [--base-url URL]
sqzass doctor [-i DIR] [--fail-on note|warn] [--drafts]
```

`--json`은 셋 다에 붙습니다.

## init

`DIR`(기본 `.`)에 새 사이트를 만들고, 필요하면 디렉터리도 만듭니다. 이미
`sqzass.toml`이 있으면 실행을 거부합니다.
[첫 사이트 만들기](@/start/first-site.md)를 참고하세요.

## build

| 플래그 | 기본값 | |
|---|---|---|
| `-i`, `--input` | `.` | 사이트 루트 — `sqzass.toml`이 있는 디렉터리. |
| `-o`, `--output` | `<input>/public` | 사이트 루트가 아니라 **셸의 현재 디렉터리** 기준으로 해석합니다. |
| `--drafts` | | `draft = true`인 페이지도 포함합니다. |
| `--base-url` | 설정값 | 프리뷰 배포에 씁니다. |

출력 디렉터리는 매번 비우고 시작합니다. 지운 페이지가 빌드된 사이트에 유령으로
남지 않습니다.

## serve

메모리에서 서빙하며 <http://127.0.0.1:3000>에 라이브 리로드를 붙입니다.
[개발 서버](@/features/dev-server.md)를 참고하세요.

## doctor

```bash
sqzass doctor -i mysite
```

`build`는 해석하지 못하는 것을 이미 거부합니다 — 깨진 `@/` 링크, 없는 템플릿, 같은
URL을 주장하는 두 페이지, 오타 난 설정 키. `doctor`는 **빌드가 받아들이지만 당신이
의도하지 않았을 수도 있는 것**을 봅니다.

| 검사 | | |
|---|---|---|
| `base-url` | warn | `base_url`이 아직 자리표시자 `https://example.com`입니다. |
| `untranslated` | warn | 어떤 언어에는 있고 어떤 언어에는 없는 페이지입니다. |
| `empty-section` | warn | 섹션에 페이지가 없어서 내비게이션 항목이 빈 곳으로 갑니다. |
| `description` | note | `description`이 없는 페이지입니다. |
| `draft` | note | 빌드에서 빠지는 페이지입니다. |
| `unused-template` | note | 어떤 페이지도 고르지 않고, 어떤 템플릿도 이름으로 부르지 않습니다. |

`--fail-on`으로 게이트를 정하고, 기본은 `warn`이며, 걸리면 `7`로 끝납니다. 기본을
`note`로 두지 않은 건 의도적입니다. note는 "알아 두라"는 말이고, 그것 때문에
파이프라인이 멈추면 사람들은 검사를 고치는 대신 doctor를 꺼 버립니다.

```bash
sqzass doctor -i mysite --fail-on note    # 엄격하게
sqzass doctor -i mysite --json            # 모든 지적을 데이터로
```

## exit code

빌드는 성공하거나, 사이트의 **어느 부분을** 받아들일 수 없었는지 알려 줍니다. 코드는
"누구 잘못이냐"에 대한 답이고, CI는 텍스트를 파싱하지 않고 여기에 조건을 걸면 됩니다.

| 코드 | 식별자 | |
|---|---|---|
| `0` | | 성공. |
| `1` | `SQZASS_E` | 분류되지 않은 실패. |
| `2` | | 잘못된 명령줄. 이건 우리 것이 아니라 clap의 코드입니다. |
| `3` | `SQZASS_E_CONFIG` | `sqzass.toml` — 못 읽거나, 형식이 틀렸거나, 없는 키를 썼습니다. |
| `4` | `SQZASS_E_CONTENT` | `content/` 아래 — front matter, 빠진 title, 같은 URL을 주장하는 두 페이지, 해석 안 되는 `@/` 링크. |
| `5` | `SQZASS_E_TEMPLATE` | `templates/`, `i18n/`, 또는 템플릿이 요청했는데 없는 에셋. |
| `6` | `SQZASS_E_IO` | 읽기나 쓰기 실패. |
| `7` | | `doctor`가 `--fail-on` 기준 이상을 찾았습니다. |

식별자는 메시지와 함께 찍히므로 그대로 검색할 수 있습니다.

```
error: [SQZASS_E_CONTENT] content/_index.md: 해석할 수 없는 내부 링크가 있습니다:
  @/nope.md
```

이 번호들은 계약입니다. 다른 뜻으로 재사용하지 않습니다 — 남의 파이프라인에 걸린
조건이 어느 날 조용히 다른 것을 검사하게 되어서는 안 되니까요.

## --json

stdout에 JSON 객체 하나만 냅니다. 스크립트는 파이프 하나만 읽으면 됩니다.

```console
$ sqzass build -i docs --json
{"ok":true,"output":"docs/public","pages":40}
```

실패도 스트림을 나누지 않고 **stdout**으로 갑니다.

```console
$ sqzass build -i broken --json
{"code":4,"error":"[SQZASS_E_CONTENT] …","kind":"SQZASS_E_CONTENT","ok":false}
$ echo $?
4
```

`init`과 `doctor`는 각자의 모양이 있습니다.

```console
$ sqzass init mysite --json
{"dir":"mysite","files":["sqzass.toml","content/_index.md","templates/page.html"],"ok":true}

$ sqzass doctor -i mysite --json
{"findings":[{"check":"base-url","file":"sqzass.toml","message":"…","severity":"warn"}],"gated":1,"ok":false}
```

`doctor`의 `ok`는 `--fail-on` 기준에 걸린 게 하나라도 있으면 false이고, `gated`가
그 개수입니다. `file`은 해당 없는 지적에서는 생략됩니다. `check` 문자열은
안정적입니다 — 스크립트가 잡아야 할 것은 메시지가 아니라 이쪽입니다.

`--json` 없이는 메시지가 stderr로, 결과가 stdout으로 갑니다. 터미널 앞의 사람이
기대하는 방식입니다.
