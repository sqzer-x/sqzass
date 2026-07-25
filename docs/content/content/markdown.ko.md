+++
title = "마크다운"
description = "기본으로 켜져 있는 확장과, 끌 수 있는 것들"
weight = 40
toc = true
+++

[comrak]을 통한 CommonMark이고, 확장 몇 가지가 기본으로 켜져 있습니다. 각각이
`sqzass.toml`의 `[markdown]` 아래 키입니다.

```toml
[markdown]
footnotes             = true
tables                = true
tasklist              = true
strikethrough         = true
autolink              = true
alerts                = true
cjk_friendly_emphasis = true
heading_anchors       = "right"   # none | left | right
```

## 알림 상자

인용문 위에 얹힌 GitHub의 콜아웃 문법입니다.

```markdown
> [!NOTE]
> 미리 빌드된 바이너리는 계획에 있지만 아직 배포하지 않았습니다.
```

> [!NOTE]
> 미리 빌드된 바이너리는 계획에 있지만 아직 배포하지 않았습니다.

`NOTE`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`을 인식합니다.

## 표

```markdown
| 키 | 기본값 |
|---|---|
| `output_dir` | `public` |
```

| 키 | 기본값 |
|---|---|
| `output_dir` | `public` |

## 체크리스트

```markdown
- [x] 검색
- [ ] 피드
```

- [x] 검색
- [ ] 피드

## 각주

각주가 붙은 문장입니다[^1].

[^1]: 각주 본문입니다.

```markdown
각주가 붙은 문장입니다[^1].

[^1]: 각주 본문입니다.
```

## 제목

모든 제목에 `id`가 붙습니다. anchor를 보여주든 말든 붙는데, 목차와 당신이 남에게
건네는 딥링크가 둘 다 여기에 기대기 때문입니다. 같은 제목이 반복되면 `-1`, `-2`가
붙고, anchor와 목차 항목은 반드시 일치합니다. 한 번의 순회에서 같은 카운터로
만들어지니까요.

`heading_anchors`는 눈에 보이는 `#` 링크를 정합니다. `"right"`(기본), `"left"`,
`"none"` 중 하나입니다.

## 원시 HTML

그대로 통과시킵니다. 여기 콘텐츠는 신뢰된 것입니다 — 당신의 저장소에, 당신이 쓰고,
코드와 같은 커밋에서 리뷰됩니다. 이걸 새니타이즈하는 건 시늉일 뿐입니다.

## 키가 아닌 설정 둘

둘 다 고정이고, 둘 다 의도한 것입니다. 옵션을 찾다가 없어서 이슈를 쓰기 전에 여기서
끝나라고 있는 절입니다.

**원시 HTML은 항상 통과합니다.** 콘텐츠는 신뢰된 것입니다. 당신의 저장소에, 당신이
쓰고, 코드와 같은 커밋에서 리뷰됩니다. 이걸 새니타이즈하는 건 시늉입니다. sqzass가
언젠가 신뢰되지 않은 마크다운을 받게 되면, 그건 전역 스위치가 아니라 소스별 신뢰
등급이 될 것입니다.

**코드 펜스는 `<pre lang="rust">`가 아니라 `<code class="language-rust">`를
만듭니다.** 클라이언트 하이라이터와 복사 버튼 스니펫이 기대하는 형태가 클래스 쪽입니다.

## 코드

빌드 시점에 CSS 클래스로 강조됩니다. [구문 강조](@/features/highlighting.md)를
참고하세요.

[comrak]: https://github.com/kivikakk/comrak
