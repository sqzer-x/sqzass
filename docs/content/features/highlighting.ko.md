+++
title = "구문 강조"
description = "빌드 시점에, 클래스로, 두 테마를 함께"
weight = 10
toc = true
+++

코드 블록은 사이트를 빌드하는 동안 강조됩니다. 색을 입히려고 독자의 브라우저에서
JavaScript가 도는 일이 없으므로, 페이지는 첫 페인트부터 색이 입혀져 있습니다.

```toml
[highlight]
enabled     = true
theme_light = "InspiredGitHub"
theme_dark  = "base16-ocean.dark"
```

## 인라인 스타일이 아니라 클래스

강조된 블록은 이렇게 생겼습니다.

```html
<pre class="highlight"><code class="language-rust" data-lang="rust"><span class="hl-source hl-rust">…</span></code></pre>
```

`style="color:#268bd2"`가 아닙니다. 이 차이가 세 가지를 결정합니다.

**다크 모드가 가능해집니다.** 인라인 색상은 지금까지 만든 모든 문서에 테마 하나를
박아 넣습니다. 테마를 바꾸려면 사이트를 다시 빌드해야 하고, 독자의 설정을 따르려면
두 테마를 마크업 안에 같이 실어 보내야 합니다. 클래스라면 두 테마는 CSS 두 덩어리일
뿐이고, 전환은 CSS 전환입니다.

**엄격한 `style-src`를 쓸 수 있습니다.** 인라인 스타일을 금지하는 CSP는 HTML이
인라인 스타일로 가득 찬 순간 선택지에서 사라집니다.

**스타일시트가 파일 하나입니다.** 색 하나를 바꾸면 HTML 페이지를 하나도 다시 만들지
않고 모든 페이지가 바뀝니다.

## 두 테마, 하나의 스타일시트

`theme_light`과 `theme_dark`이 둘 다 생성된 스타일시트에 들어갑니다. 다크 규칙은
두 번 나갑니다. 토글을 쓰지 않는 독자를 위해 `prefers-color-scheme: dark` 아래
한 번, 이 사이트처럼 전환 버튼을 제공하는 경우를 위해 `[data-theme="dark"]` 아래
한 번입니다.

syntect 기본 세트의 테마 이름이면 무엇이든 됩니다. 없는 이름을 쓰면 빌드 에러이고,
메시지에 쓸 수 있는 이름들이 같이 나옵니다.

## 접두사

클래스에는 `hl-` 접두사가 붙습니다. 접두사가 없으면 syntect는 `source`, `keyword`,
`string` 같은 이름을 내보내는데, 프로그래밍을 다루는 사이트에서 당신의 스타일시트와
부딪히기 딱 좋은 일반적인 단어들입니다.

## 줄 표시와 파일명

옵션은 언어 뒤에, 공백으로 구분해, `key=value`로 적습니다.

````markdown
```rust hl_lines=2-3 name=src/main.rs
fn main() {
    let marked = 2;
    let also_marked = 3;
}
```
````

이 사이트의 스타일시트로 렌더하면 이렇게 됩니다.

```rust hl_lines=2-3 name=src/main.rs
fn main() {
    let marked = 2;
    let also_marked = 3;
}
```

`hl_lines`는 1부터 세는 줄 번호와 닫힌 구간을 쉼표로 나열합니다:
`hl_lines=2-4,7`. 지목된 줄은 `<mark class="hl-line">`로 감싸집니다.
`<mark>`는 브라우저가 기본으로 칠해 주므로 CSS가 하나도 없어도 표시가 보이고,
사이트는 `.highlight mark`로 다시 꾸미면 됩니다. 강조는 줄 경계를 건너 이어집니다.
여러 줄 주석의 가운데 줄만 표시해도 그 줄은 여전히 주석입니다.

`name`은 블록에 파일명을 답니다. 마크업이 아니라 속성으로 나가므로 —
`<code … data-name="src/main.rs">` — 보여줄지는 당신의 CSS가 정합니다.

```css
.prose pre code { display: block; width: max-content; min-width: 100%; }
.prose pre code[data-name]::before {
  content: attr(data-name);
  display: block;
}
```

첫 규칙은 `code`를 가장 긴 줄까지 넓힙니다. 블록이 옆으로 스크롤될 때 라벨과
`hl_lines` 표시가 보이는 폭에서 끊기지 않고 줄 끝까지 닿는 것은 이 규칙
덕입니다.

Zola식 `rust,hl_lines=2-4`가 아닌 이유: 쉼표는 옵션을 언어 토큰에 붙여 버려서
문법 조회가 실패하고 `class="language-…"`가 오염됩니다. 첫 공백 뒤는 자유롭고,
comrak 자신이 정확히 거기서 정보 문자열을 가릅니다.

오타는 조용한 무시가 아니라 빌드 에러입니다. `hl_line=3`도, `linenos=true`도,
세 줄짜리 블록의 `hl_lines=9`도, 두 번 적은 키도 전부 파일명과 함께 빌드를
멈춥니다 — 그 반대는 멀쩡해 보이는 채로 강조가 빠진 페이지가 배포되는
것입니다. 언어 없이 옵션부터 시작한 펜스도 마찬가지입니다. 언어 이름에 `=`가
들어가는 일은 없으므로, `` ```hl_lines=2 ``는 모르는 언어가 아니라 언어 자리에
온 옵션입니다. 옵션은 하이라이터의 일부이므로 `enabled = false`면 적용도 검사도
되지 않습니다.

## 줄 번호와 복사 버튼은 당신 몫입니다

둘 다 설정 키가 아니고, `line_numbers`는 아무 일도 안 하는 채로 남기느니 **삭제**했습니다.
아무것도 하지 않는 설정은 없는 것보다 나쁩니다. 누군가 그걸 켜 놓고 기다리게 되니까요.

보통의 블록에는 CSS 카운터를 얹을 줄 단위 마크업이 없습니다 — 모든 블록의 모든
줄에 래퍼를 감으면, 대부분의 블록이 안 쓰는 기능의 값을 모든 페이지가 치르게
됩니다. 줄 번호 거터는 줄 수를 세는 JS 몇 줄입니다.

```js
document.querySelectorAll(".prose pre > code").forEach(function (code) {
  var lines = code.textContent.split("\n").length - 1;
  var gutter = document.createElement("span");
  gutter.className = "linenos";
  for (var i = 1; i <= lines; i++) gutter.textContent += i + "\n";
  code.parentElement.prepend(gutter);
});
```

```css
.prose pre { display: flex; gap: 1em; }
.prose pre .linenos { text-align: right; color: var(--ink-3); user-select: none; }
```

복사 버튼은 열 줄 남짓이고, 언어는 이미 엘리먼트에 붙어 있습니다.

```js
document.querySelectorAll(".prose pre").forEach(function (pre) {
  var b = document.createElement("button");
  b.textContent = "copy";
  b.addEventListener("click", function () {
    navigator.clipboard.writeText(pre.textContent);
  });
  pre.appendChild(b);
});
```

둘 다 당신의 `static/`에 있고, 둘 다 당신이 다시 꾸밀 수 있으며, 둘 다 영원히 같은
뜻을 유지해야 하는 설정 파일에 키를 하나도 더하지 않습니다.

## 끄기

`enabled = false`면 강조를 건너뛰고 스타일시트도 만들지 않습니다. 코드 블록에는
여전히 `class="language-rust"`가 붙으므로 클라이언트 쪽 하이라이터가 받아 쓸 수
있습니다.
