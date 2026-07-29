# 벤치마크 재현

<https://sqzass.sqzer.com/benchmark/> 의 수치를 만든 도구다. 수치는 머신마다
다르니 **비율을 보되, 반드시 직접 돌려서** 확인할 것.

```bash
cargo build --release                 # sqzass (기본 = onig 엔진)
python3 bench/gen.py                  # 도구 5종 × 시나리오 4종 코퍼스 생성
python3 bench/bench.py sqzass         # 도구별 실행 (hugo/zola/jekyll/astro)
```

- 대조군 설치는 각자: hugo, zola, jekyll(gem), astro(`bench/astro-base`에
  `package.json`을 만들고 `npm install astro`).
- `gen.py`는 자기 디렉터리 아래에 코퍼스를 만든다. 본문 마크다운은 도구 간
  바이트 동일, front matter 형식만 다르다.
- `bench.py`는 매 회 출력물과 도구별 캐시(.jekyll-cache, astro cacheDir·dist,
  hugo resources·lock)를 지운 콜드 빌드다. `taskset -c 0-3`, 3회, 중앙값,
  GNU time으로 peak RSS.
- 결과는 `results/<tool>-<scenario>.json`. 공개 수치의 원본은
  `results-2026-07-29.json`.

시나리오 정의는 `gen.py`가 곧 명세다 — 특히 heavy가 **repeat**(페이지당 동일
코드 블록 5개)와 **unique**(전부 다른 블록)로 나뉘는 이유는 코드 반복도가
순위를 가르는 변수라서다. 하나의 heavy 수치만 공개하는 벤치마크는 그 변수를
숨긴 것이다.
