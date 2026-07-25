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
<code class="language-rust"><span class="hl-source hl-rust">…</span></code>
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

## 줄 번호와 복사 버튼은 당신 몫입니다

둘 다 설정 키가 아니고, `line_numbers`는 아무 일도 안 하는 채로 남기느니 **삭제**했습니다.
아무것도 하지 않는 설정은 없는 것보다 나쁩니다. 누군가 그걸 켜 놓고 기다리게 되니까요.

줄 번호는 하이라이터가 이미 내보내는 줄에 CSS 카운터를 얹는 것입니다.

```css
.prose pre code { counter-reset: line; }
.prose pre code > .line::before {
  counter-increment: line;
  content: counter(line);
  display: inline-block;
  width: 2.5em;
  text-align: right;
  margin-right: 1em;
  color: var(--ink-3);
  user-select: none;   /* 블록을 복사할 때 번호까지 딸려가지 않게 */
}
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
