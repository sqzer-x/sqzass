# Set up sqzass in this project

sqzass is a static site generator: markdown in, a directory of HTML out. One
binary, no runtime — no Node, no Python, no system libraries.

## 1. Install

It is **not on crates.io**. `cargo install sqzass` will fail.

```bash
curl -fsSL https://sqzass.sqzer.com/install.sh | sh
# with Rust: cargo install --git https://github.com/sqzer-x/sqzass
# on Arch:   yay -S sqzass
```

## 2. Scaffold and build

`init` writes three files and refuses to run where a `sqzass.toml` already
exists, so it cannot half-overwrite a site that is already there.

```bash
sqzass init .        # sqzass.toml, content/_index.md, templates/page.html
sqzass build -i .    # writes public/
sqzass serve -i .    # live reload on http://127.0.0.1:3000
```

Two config keys are required: `title` and `base_url`. Everything else has a
default.

## 3. Rules you cannot infer from the file tree

- Front matter is **TOML fenced with `+++`**, not YAML with `---`. `title` is
  the only required field.
- Internal links are `@/section/page.md` — the *source path* under `content/`,
  not the URL. The build rewrites it and resolves it to the reader's language.
- Images are different: use a root-absolute path, `![](/images/x.png)`. `@/`
  resolves against pages only, so `@/images/x.png` stops the build.
- Translations sit beside each other: `page.md` and `page.ko.md`.
- `page.content` is already HTML in templates, so it needs `| safe`. Templates
  autoescape by default and reading an undefined value is an error.

## 4. Errors are deliberate

An unresolved link, a template that does not exist, two pages claiming one URL,
a misspelled key in `sqzass.toml` or in front matter — each one stops the build
and names the file and line. There is no flag to downgrade any of them to a
warning. Fix the cause.

## 5. Reading the output as a program

Runtime messages are in Korean. The identifiers are not: error codes and
`doctor` check names are ASCII and stable, so match on those rather than on
message text.

| Exit | Identifier | |
|---|---|---|
| `0` | | Success. |
| `1` | `SQZASS_E` | Unclassified failure. |
| `2` | | Bad command line (clap's code, not ours). |
| `3` | `SQZASS_E_CONFIG` | `sqzass.toml`. |
| `4` | `SQZASS_E_CONTENT` | Anything under `content/`. |
| `5` | `SQZASS_E_TEMPLATE` | `templates/`, `i18n/`, or a missing asset. |
| `6` | `SQZASS_E_IO` | Reading or writing failed. |
| `7` | | `doctor` found something at or above `--fail-on`. |

`--json` works on every command and prints one object to stdout — on failure
too, so a script reads one pipe.

```console
$ sqzass build -i . --json
{"ok":true,"output":"public","pages":1}

$ sqzass doctor -i . --json
{"findings":[{"check":"base-url","file":"sqzass.toml","message":"…","severity":"warn"}],"gated":1,"ok":false}
```

## Full documentation

<https://sqzass.sqzer.com> — and <https://sqzass.sqzer.com/llms.txt> is a flat
list of every page with its description.
