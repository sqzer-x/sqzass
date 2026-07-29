#!/usr/bin/env python3
"""벤치마크 하네스. 사용: bench.py <tool> [scenario...]
콜드 규율: 매 회 출력 디렉터리와 도구별 캐시를 지운다. taskset -c 0-3, 3회, 중앙값.
결과는 results/<tool>-<scenario>.json 으로 append-safe하게 저장."""
import json, os, re, shutil, statistics, subprocess, sys, time

S = os.path.dirname(os.path.abspath(__file__))
RUNS = 3
GEM_BIN = subprocess.run(["ruby", "-e", "puts Gem.user_dir"], capture_output=True, text=True).stdout.strip() + "/bin"

TOOLS = {
    "sqzass": {
        "cmd": lambda d: [os.environ.get("SQZASS", "target/release/sqzass"),
                          "build", "-i", d, "-o", f"{d}/out"],
        "clean": lambda d: [f"{d}/out"],
        "cwd": None,
    },
    "hugo": {
        "cmd": lambda d: ["hugo", "--quiet", "--source", d, "--destination", f"{d}/out"],
        "clean": lambda d: [f"{d}/out", f"{d}/resources", f"{d}/.hugo_build.lock"],
        "cwd": None,
    },
    "zola": {
        "cmd": lambda d: ["zola", "--root", d, "build", "--output-dir", f"{d}/out", "--force"],
        "clean": lambda d: [f"{d}/out"],
        "cwd": None,
    },
    "jekyll": {
        "cmd": lambda d: [f"{GEM_BIN}/jekyll", "build", "-s", d, "-d", f"{d}/out", "--disable-disk-cache"],
        "clean": lambda d: [f"{d}/out", f"{d}/.jekyll-cache"],
        "cwd": None,
    },
    "astro": {
        "cmd": lambda d: ["./node_modules/.bin/astro", "build"],
        "clean": lambda d: [f"{d}/dist", f"{d}/.astro-cache"],
        "cwd": lambda d: d,
    },
}

def clean(paths):
    for p in paths:
        if os.path.isdir(p):
            shutil.rmtree(p, ignore_errors=True)
        elif os.path.exists(p):
            os.remove(p)

def run_one(tool, scenario):
    t = TOOLS[tool]
    d = f"{S}/{tool}/{scenario}"
    times, rss = [], []
    for i in range(RUNS):
        clean(t["clean"](d))
        cmd = ["taskset", "-c", "0-3", "/usr/bin/time", "-v"] + t["cmd"](d)
        cwd = t["cwd"](d) if t["cwd"] else None
        t0 = time.monotonic()
        r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
        wall = time.monotonic() - t0
        if r.returncode != 0:
            print(f"FAIL {tool}/{scenario}: {r.stderr[-500:]}", file=sys.stderr)
            sys.exit(1)
        m = re.search(r"Maximum resident set size \(kbytes\): (\d+)", r.stderr)
        times.append(wall * 1000)
        rss.append(int(m.group(1)) if m else 0)
        time.sleep(2)
    result = {
        "tool": tool, "scenario": scenario,
        "median_ms": round(statistics.median(times)),
        "runs_ms": [round(x) for x in times],
        "peak_rss_mb": round(max(rss) / 1024),
    }
    os.makedirs(f"{S}/results", exist_ok=True)
    with open(f"{S}/results/{tool}-{scenario}.json", "w") as f:
        json.dump(result, f)
    print(f"{tool:7} {scenario:8} 중앙값 {result['median_ms']:>7} ms  "
          f"(runs: {', '.join(str(x) for x in result['runs_ms'])})  peakRSS {result['peak_rss_mb']} MB")

tool = sys.argv[1]
scenarios = sys.argv[2:] or ["minimal", "blog", "heavyr", "heavyu"]
for sc in scenarios:
    run_one(tool, sc)
