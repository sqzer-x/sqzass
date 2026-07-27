//! 페이지 목록을 섹션 트리로 조립하고, 언어별 내비게이션과 번역 연결을 만든다.
//!
//! 트리는 **언어별로 따로** 만든다. 미번역 페이지는 해당 언어 트리에 아예 존재하지
//! 않으므로, 사이드바에 죽은 링크가 생기지 않는다.

use crate::config::{Config, SortBy};
use crate::content::Page;
use serde::Serialize;
use std::collections::BTreeMap;

/// 언어별 섹션 트리 + 번역 색인.
pub struct Site<'a> {
    pub cfg: &'a Config,
    pub pages: &'a [Page],
    /// 언어 코드 → 최상위 섹션들
    trees: BTreeMap<String, Vec<Section>>,
    /// translation_key → (언어 코드 → 페이지 인덱스)
    translations: BTreeMap<String, BTreeMap<String, usize>>,
    /// 페이지 URL → 소속 섹션 안에서의 순번 (정렬 후).
    ///
    /// `neighbours_of`가 페이지마다 섹션 목록을 URL 비교로 훑으면 평평한 대형
    /// 섹션에서 O(k²)가 된다 — 한 번 만들어 두면 조회다. 키가 URL인 근거:
    /// URL 충돌은 discover 단계의 하드 에러라 이 시점에 URL은 전역 유일하다.
    pos_in_section: BTreeMap<String, usize>,
}

/// 내부 트리 노드. 페이지는 인덱스로 참조해 소유권 문제를 피한다.
#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub description: String,
    pub url: String,
    pub weight: i64,
    /// `content/` 기준 디렉터리 경로 (언어 접두사 제외)
    pub path: Vec<String>,
    /// 이 섹션의 `_index.md`
    pub index: Option<usize>,
    /// 직속 자식 페이지 (정렬됨)
    pub pages: Vec<usize>,
    pub subsections: Vec<Section>,
    /// `_index.md`의 `page_template` — 자식 페이지의 기본 템플릿
    pub page_template: Option<String>,
}

impl<'a> Site<'a> {
    pub fn build(pages: &'a [Page], cfg: &'a Config) -> Self {
        let mut translations: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for (i, p) in pages.iter().enumerate() {
            translations
                .entry(p.translation_key.clone())
                .or_default()
                .insert(p.language.clone(), i);
        }

        let mut by_lang: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, p) in pages.iter().enumerate() {
            by_lang.entry(p.language.clone()).or_default().push(i);
        }

        let trees: BTreeMap<String, Vec<Section>> = by_lang
            .into_iter()
            .map(|(lang, idxs)| (lang, build_tree(pages, &idxs, cfg)))
            .collect();

        let mut pos_in_section = BTreeMap::new();
        for sections in trees.values() {
            index_positions(sections, pages, &mut pos_in_section);
        }

        Self {
            cfg,
            pages,
            trees,
            translations,
            pos_in_section,
        }
    }

    /// 해당 언어의 최상위 섹션들.
    pub fn sections(&self, language: &str) -> &[Section] {
        self.trees
            .get(language)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// 페이지가 속한 섹션. 최상위 페이지는 소속 섹션이 없다.
    fn section_of(&self, page: &Page) -> Option<&Section> {
        let dirs: Vec<String> = page
            .rel
            .parent()?
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        if dirs.is_empty() {
            return None;
        }
        find_section(self.sections(&page.language), &dirs)
    }

    /// 페이지가 속한 섹션의 `page_template`. 템플릿 선택 단계에서 쓴다.
    pub fn page_template_for(&self, page: &Page) -> Option<&str> {
        self.section_of(page)?.page_template.as_deref()
    }

    /// 페이지가 속한 섹션을 참조 형태로. 브레드크럼과 검색 결과의 갈래 라벨이 쓴다.
    pub fn section_ref_of(&self, page: &Page) -> Option<PageRefCtx> {
        let s = self.section_of(page)?;
        Some(PageRefCtx {
            title: s.title.clone(),
            description: s.description.clone(),
            url: s.url.clone(),
            weight: s.weight,
        })
    }

    /// `@/` 링크 해석용 표. translation_key → (언어 → URL)
    pub fn link_index(&self) -> crate::links::LinkIndex {
        self.translations
            .iter()
            .map(|(key, by_lang)| {
                let urls = by_lang
                    .iter()
                    .map(|(lang, &i)| (lang.clone(), self.pages[i].url.clone()))
                    .collect();
                (key.clone(), urls)
            })
            .collect()
    }

    /// 섹션 인덱스 페이지의 직속 자식들. 섹션 페이지가 자기 하위 목록을 보여줄 때 쓴다.
    pub fn children_of(&self, page: &Page) -> Vec<PageRefCtx> {
        if !page.is_section {
            return Vec::new();
        }
        let dirs: Vec<String> = page
            .rel
            .parent()
            .map(|d| {
                d.components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();

        let sections = self.sections(&page.language);
        // 루트 `_index.md`의 자식은 최상위 섹션들이다.
        let Some(node) = (if dirs.is_empty() {
            None
        } else {
            find_section(sections, &dirs)
        }) else {
            return sections
                .iter()
                .map(|s| PageRefCtx {
                    title: s.title.clone(),
                    description: s.description.clone(),
                    url: s.url.clone(),
                    weight: s.weight,
                })
                .collect();
        };

        let mut out: Vec<PageRefCtx> = node
            .pages
            .iter()
            .map(|&i| PageRefCtx {
                title: self.pages[i].title.clone(),
                description: self.pages[i].front.description.clone(),
                url: self.pages[i].url.clone(),
                weight: self.pages[i].front.weight,
            })
            .collect();
        out.extend(node.subsections.iter().map(|s| PageRefCtx {
            title: s.title.clone(),
            description: s.description.clone(),
            url: s.url.clone(),
            weight: s.weight,
        }));
        out
    }

    /// 같은 섹션 안에서의 (이전, 다음) 페이지. 섹션이 정한 순서를 그대로 따른다.
    ///
    /// 섹션 인덱스에는 이웃이 없다 — 섹션은 자기 자식들과 같은 줄에 서 있지 않다.
    pub fn neighbours_of(&self, page: &Page) -> (Option<PageRefCtx>, Option<PageRefCtx>) {
        if page.is_section {
            return (None, None);
        }
        let Some(section) = self.section_of(page) else {
            return (None, None);
        };
        let Some(&at) = self.pos_in_section.get(&page.url) else {
            return (None, None);
        };

        let make = |i: usize| {
            let p = &self.pages[section.pages[i]];
            PageRefCtx {
                title: p.title.clone(),
                description: p.front.description.clone(),
                url: p.url.clone(),
                weight: p.front.weight,
            }
        };
        let prev = at.checked_sub(1).map(&make);
        let next = (at + 1 < section.pages.len()).then(|| make(at + 1));
        (prev, next)
    }

    /// 이 페이지가 존재하는 **모든** 언어판, 자기 자신 포함. `(언어 코드, URL)`.
    ///
    /// `translations_of`와 달리 자기 자신을 뺀 목록이 아니다. sitemap의 hreflang
    /// 세트는 각 URL이 자기 자신까지 포함해 전부를 나열해야 유효하다.
    pub fn language_set_of(&self, page: &Page) -> Vec<(String, String)> {
        let Some(by_lang) = self.translations.get(&page.translation_key) else {
            return Vec::new();
        };
        let mut out: Vec<(String, String, i64)> = by_lang
            .iter()
            .map(|(code, &idx)| {
                let weight = self.cfg.languages.get(code).map_or(0, |l| l.weight);
                (code.clone(), self.pages[idx].url.clone(), weight)
            })
            .collect();
        out.sort_by(|a, b| (a.2, &a.0).cmp(&(b.2, &b.0)));
        out.into_iter().map(|(c, u, _)| (c, u)).collect()
    }

    /// 이 페이지의 다른 언어판. **번역이 실제로 있는 것만** 담는다.
    pub fn translations_of(&self, page: &Page) -> Vec<LanguageLink> {
        let Some(by_lang) = self.translations.get(&page.translation_key) else {
            return Vec::new();
        };
        let mut out: Vec<LanguageLink> = by_lang
            .iter()
            .filter(|(code, _)| code.as_str() != page.language)
            .filter_map(|(code, &idx)| {
                let lang = self.cfg.languages.get(code)?;
                Some(LanguageLink {
                    code: code.clone(),
                    name: lang.name.clone(),
                    url: self.pages[idx].url.clone(),
                    weight: lang.weight,
                })
            })
            .collect();
        out.sort_by_key(|l| (l.weight, l.code.clone()));
        out
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageLink {
    pub code: String,
    pub name: String,
    pub url: String,
    #[serde(skip)]
    pub weight: i64,
}

/// 한 언어의 페이지들로 섹션 트리를 만든다.
fn build_tree(pages: &[Page], idxs: &[usize], cfg: &Config) -> Vec<Section> {
    // 디렉터리 경로 → 섹션. 루트는 빈 경로.
    let mut root = Section {
        title: String::new(),
        description: String::new(),
        url: "/".into(),
        weight: 0,
        path: Vec::new(),
        index: None,
        pages: Vec::new(),
        subsections: Vec::new(),
        page_template: None,
    };

    // 1) 섹션 노드부터 만든다 (`_index.md` 기준)
    for &i in idxs {
        let p = &pages[i];
        if !p.is_section {
            continue;
        }
        let dirs = dirs_of(p);
        let node = ensure_section(&mut root, &dirs);
        node.index = Some(i);
        node.title = p.title.clone();
        node.description = p.front.description.clone();
        node.url = p.url.clone();
        node.weight = p.front.weight;
        node.page_template = p.front.page_template.clone();
    }

    // 2) 일반 페이지를 붙인다. `_index.md`가 없는 디렉터리도 섹션으로 승격시킨다 —
    //    없으면 페이지가 트리에서 사라져 사이드바에 안 나온다.
    for &i in idxs {
        let p = &pages[i];
        if p.is_section {
            continue;
        }
        let dirs = dirs_of(p);
        let node = ensure_section(&mut root, &dirs);
        node.pages.push(i);
    }

    sort_section(&mut root, pages, cfg);
    root.subsections
}

/// `content/` 기준 디렉터리 세그먼트.
fn dirs_of(p: &Page) -> Vec<String> {
    p.rel
        .parent()
        .map(|d| {
            d.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// 경로를 따라 내려가며 없으면 만든다.
fn ensure_section<'s>(root: &'s mut Section, dirs: &[String]) -> &'s mut Section {
    let mut cur = root;
    for (depth, seg) in dirs.iter().enumerate() {
        let pos = cur
            .subsections
            .iter()
            .position(|s| s.path.last() == Some(seg));
        let pos = match pos {
            Some(p) => p,
            None => {
                let mut path = cur.path.clone();
                path.push(seg.clone());
                cur.subsections.push(Section {
                    // `_index.md`가 없으면 디렉터리 이름을 제목으로 쓴다.
                    title: seg.clone(),
                    description: String::new(),
                    url: format!("/{}/", path.join("/")),
                    weight: 0,
                    path,
                    index: None,
                    pages: Vec::new(),
                    subsections: Vec::new(),
                    page_template: None,
                });
                let _ = depth;
                cur.subsections.len() - 1
            }
        };
        cur = &mut cur.subsections[pos];
    }
    cur
}

/// 정렬이 끝난 트리에서 페이지 URL → 섹션 내 순번을 한 번에 적는다.
fn index_positions(sections: &[Section], pages: &[Page], out: &mut BTreeMap<String, usize>) {
    for s in sections {
        for (pos, &i) in s.pages.iter().enumerate() {
            out.insert(pages[i].url.clone(), pos);
        }
        index_positions(&s.subsections, pages, out);
    }
}

fn find_section<'s>(sections: &'s [Section], dirs: &[String]) -> Option<&'s Section> {
    let (first, rest) = dirs.split_first()?;
    let node = sections.iter().find(|s| s.path.last() == Some(first))?;
    if rest.is_empty() {
        Some(node)
    } else {
        find_section(&node.subsections, rest)
    }
}

/// 섹션 자신의 `sort_by`가 있으면 그걸, 없으면 `[nav] sort_by`를 쓴다.
fn sort_section(sec: &mut Section, pages: &[Page], cfg: &Config) {
    let sort_by = sec
        .index
        .and_then(|i| pages[i].front.sort_by)
        .unwrap_or(cfg.nav.sort_by);

    // 날짜순은 **내림차순**이고 날짜 없는 페이지는 뒤로 간다. 정렬 키를 뒤집는
    // 대신 없는 날짜를 0으로 두면 그것들이 맨 앞으로 올라온다.
    let date_key = |i: &usize| -> (std::cmp::Reverse<i64>, String) {
        let p = &pages[*i];
        let k = p
            .front
            .date
            .as_ref()
            .and_then(crate::feed::PageDate::from_toml)
            .map_or(0, |d| d.sort_key());
        (std::cmp::Reverse(k), p.title.clone())
    };

    match sort_by {
        SortBy::Date => sec.pages.sort_by_key(date_key),
        _ => sec.pages.sort_by_key(|i: &usize| {
            let p = &pages[*i];
            match sort_by {
                SortBy::Title => (0, p.title.clone()),
                _ => (p.front.weight, p.title.clone()),
            }
        }),
    }

    // 하위 섹션에는 날짜가 없다. 날짜순 섹션에서도 제목순으로 둔다.
    sec.subsections.sort_by_key(|s| match sort_by {
        SortBy::Weight => (s.weight, s.title.clone()),
        _ => (0, s.title.clone()),
    });

    for sub in &mut sec.subsections {
        sort_section(sub, pages, cfg);
    }
}

// --- 템플릿 컨텍스트 ---

#[derive(Debug, Clone, Serialize)]
pub struct SectionCtx {
    pub title: String,
    pub description: String,
    pub url: String,
    pub weight: i64,
    pub pages: Vec<PageRefCtx>,
    pub subsections: Vec<SectionCtx>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageRefCtx {
    pub title: String,
    pub description: String,
    pub url: String,
    pub weight: i64,
}

pub fn section_ctx(sections: &[Section], pages: &[Page]) -> Vec<SectionCtx> {
    sections
        .iter()
        .map(|s| SectionCtx {
            title: s.title.clone(),
            description: s.description.clone(),
            url: s.url.clone(),
            weight: s.weight,
            pages: s
                .pages
                .iter()
                .map(|&i| PageRefCtx {
                    title: pages[i].title.clone(),
                    description: pages[i].front.description.clone(),
                    url: pages[i].url.clone(),
                    weight: pages[i].front.weight,
                })
                .collect(),
            subsections: section_ctx(&s.subsections, pages),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;
    use std::path::{Path, PathBuf};

    fn cfg() -> Config {
        toml::from_str(
            r#"
            title = "t"
            base_url = "https://example.com"
            default_language = "en"
            [languages.en]
            name = "English"
            weight = 1
            [languages.ko]
            name = "한국어"
            weight = 2
            "#,
        )
        .unwrap()
    }

    /// 임시 사이트를 만들고 discover까지 돌린다.
    struct Fixture(PathBuf);

    impl Fixture {
        fn new(name: &str, files: &[(&str, &str)]) -> Self {
            let root =
                std::env::temp_dir().join(format!("sqzass-site-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            for (rel, body) in files {
                let path = root.join("content").join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, body).unwrap();
            }
            Self(root)
        }
        fn pages(&self, cfg: &Config) -> Vec<Page> {
            content::discover(&self.0, cfg, false).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn fm(title: &str, extra: &str) -> String {
        format!("+++\ntitle = \"{title}\"\n{extra}+++\n\nbody\n")
    }

    /// weight와 title은 오름차순인데 date만 내림차순이다. 날짜순 목록에서
    /// 사람이 기대하는 건 최신 글이고, 날짜 없는 글은 뒤로 가야 한다.
    #[test]
    fn date_sorting_is_newest_first_and_undated_last() {
        let cfg = cfg();
        let f = Fixture::new(
            "dates",
            &[
                ("_index.md", &fm("Home", "")),
                ("posts/_index.md", &fm("Posts", "sort_by = \"date\"\n")),
                ("posts/a.md", &fm("A", "date = 2026-01-15\n")),
                ("posts/b.md", &fm("B", "date = 2026-09-01\n")),
                ("posts/c.md", &fm("C", "date = 2026-05-20\n")),
                ("posts/none.md", &fm("None", "")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let posts = &site.sections("en")[0];
        let titles: Vec<&str> = posts
            .pages
            .iter()
            .map(|&i| pages[i].title.as_str())
            .collect();
        assert_eq!(titles, vec!["B", "C", "A", "None"], "실제: {titles:?}");
    }

    #[test]
    fn neighbours_follow_the_section_order() {
        let cfg = cfg();
        let f = Fixture::new(
            "neighbours",
            &[
                ("_index.md", &fm("Home", "")),
                ("start/_index.md", &fm("Getting started", "weight = 10\n")),
                ("start/a.md", &fm("A", "weight = 1\n")),
                ("start/b.md", &fm("B", "weight = 2\n")),
                ("start/c.md", &fm("C", "weight = 3\n")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let by_title = |t: &str| pages.iter().find(|p| p.title == t).unwrap();

        let (prev, next) = site.neighbours_of(by_title("B"));
        assert_eq!(prev.map(|p| p.title), Some("A".into()));
        assert_eq!(next.map(|p| p.title), Some("C".into()));

        // 양 끝은 한쪽만 있다. 다른 섹션으로 넘어가지 않는다.
        let (prev, next) = site.neighbours_of(by_title("A"));
        assert!(prev.is_none(), "첫 페이지에 이전이 있다");
        assert_eq!(next.map(|p| p.title), Some("B".into()));
        let (prev, next) = site.neighbours_of(by_title("C"));
        assert_eq!(prev.map(|p| p.title), Some("B".into()));
        assert!(next.is_none(), "마지막 페이지에 다음이 있다");

        // 섹션 인덱스는 자식들과 같은 줄에 서 있지 않다.
        let (prev, next) = site.neighbours_of(by_title("Getting started"));
        assert!(prev.is_none() && next.is_none());
    }

    #[test]
    fn language_set_includes_the_page_itself() {
        let cfg = cfg();
        let f = Fixture::new(
            "langset",
            &[
                ("_index.md", &fm("Home", "")),
                ("_index.ko.md", &fm("홈", "")),
                ("solo.md", &fm("Solo", "")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let by_title = |t: &str| pages.iter().find(|p| p.title == t).unwrap();

        // sitemap의 hreflang 세트는 자기 자신까지 나열해야 유효하다.
        let set = site.language_set_of(by_title("Home"));
        assert_eq!(
            set,
            vec![
                ("en".to_string(), "/".to_string()),
                ("ko".to_string(), "/ko/".to_string())
            ]
        );
        assert_eq!(site.language_set_of(by_title("Solo")).len(), 1);
    }

    #[test]
    fn builds_nested_sections_per_language() {
        let cfg = cfg();
        let f = Fixture::new(
            "tree",
            &[
                ("_index.md", &fm("Home", "")),
                ("_index.ko.md", &fm("홈", "")),
                ("start/_index.md", &fm("Getting started", "weight = 10\n")),
                ("start/_index.ko.md", &fm("시작하기", "weight = 10\n")),
                ("start/install.md", &fm("Install", "weight = 1\n")),
                ("start/install.ko.md", &fm("설치", "weight = 1\n")),
                ("start/first.md", &fm("First site", "weight = 2\n")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);

        let en = site.sections("en");
        assert_eq!(en.len(), 1, "최상위 섹션은 start 하나여야 한다");
        assert_eq!(en[0].title, "Getting started");
        assert_eq!(en[0].url, "/start/");
        assert_eq!(en[0].pages.len(), 2);

        let ko = site.sections("ko");
        assert_eq!(ko.len(), 1);
        assert_eq!(ko[0].title, "시작하기");
        assert_eq!(ko[0].url, "/ko/start/");
        // 한국어판은 install만 번역돼 있다
        assert_eq!(ko[0].pages.len(), 1, "미번역 페이지가 트리에 새어들어왔다");
    }

    #[test]
    fn sorts_pages_by_weight_then_title() {
        let cfg = cfg();
        let f = Fixture::new(
            "sort",
            &[
                ("docs/_index.md", &fm("Docs", "")),
                ("docs/c.md", &fm("C", "weight = 1\n")),
                ("docs/a.md", &fm("A", "weight = 3\n")),
                ("docs/b.md", &fm("B", "weight = 2\n")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let titles: Vec<&str> = site.sections("en")[0]
            .pages
            .iter()
            .map(|&i| pages[i].title.as_str())
            .collect();
        assert_eq!(titles, vec!["C", "B", "A"]);
    }

    #[test]
    fn section_sort_by_overrides_global() {
        let cfg = cfg();
        let f = Fixture::new(
            "sortoverride",
            &[
                ("docs/_index.md", &fm("Docs", "sort_by = \"title\"\n")),
                ("docs/c.md", &fm("C", "weight = 1\n")),
                ("docs/a.md", &fm("A", "weight = 3\n")),
                ("docs/b.md", &fm("B", "weight = 2\n")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let titles: Vec<&str> = site.sections("en")[0]
            .pages
            .iter()
            .map(|&i| pages[i].title.as_str())
            .collect();
        assert_eq!(
            titles,
            vec!["A", "B", "C"],
            "섹션의 sort_by가 전역을 못 이겼다"
        );
    }

    #[test]
    fn directory_without_index_still_appears() {
        // `_index.md`를 깜빡해도 페이지가 사이드바에서 사라지면 안 된다.
        let cfg = cfg();
        let f = Fixture::new("noindex", &[("guides/howto.md", &fm("How to", ""))]);
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let en = site.sections("en");
        assert_eq!(en.len(), 1);
        assert_eq!(
            en[0].title, "guides",
            "디렉터리 이름이 제목 대체값이어야 한다"
        );
        assert_eq!(en[0].pages.len(), 1);
    }

    #[test]
    fn translations_only_include_existing_ones() {
        let cfg = cfg();
        let f = Fixture::new(
            "trans",
            &[
                ("a.md", &fm("A", "")),
                ("a.ko.md", &fm("가", "")),
                ("b.md", &fm("B", "")), // 번역 없음
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);

        let a = pages.iter().find(|p| p.title == "A").unwrap();
        let t = site.translations_of(a);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].code, "ko");
        assert_eq!(t[0].url, "/ko/a/");
        assert_eq!(t[0].name, "한국어");

        let b = pages.iter().find(|p| p.title == "B").unwrap();
        assert!(
            site.translations_of(b).is_empty(),
            "번역이 없는데 언어 링크가 생겼다"
        );
    }

    #[test]
    fn page_template_comes_from_parent_section() {
        let cfg = cfg();
        let f = Fixture::new(
            "ptmpl",
            &[
                (
                    "docs/_index.md",
                    &fm("Docs", "page_template = \"doc.html\"\n"),
                ),
                ("docs/x.md", &fm("X", "")),
                ("other.md", &fm("Other", "")),
            ],
        );
        let pages = f.pages(&cfg);
        let site = Site::build(&pages, &cfg);
        let x = pages.iter().find(|p| p.title == "X").unwrap();
        assert_eq!(site.page_template_for(x), Some("doc.html"));
        let o = pages.iter().find(|p| p.title == "Other").unwrap();
        assert_eq!(site.page_template_for(o), None);
    }

    #[test]
    fn anchorizer_passes_hangul_through() {
        // 계획 단계에서 "착수 즉시 확인"으로 표시했던 항목. 한글이 버려지면
        // 모든 한국어 제목의 id가 충돌해 TOC와 anchor가 붕괴한다.
        let mut a = comrak::Anchorizer::new();
        assert_eq!(a.anchorize("설치"), "설치");
        assert_eq!(a.anchorize("정적 사이트 생성기"), "정적-사이트-생성기");
        assert_eq!(
            a.anchorize("설치"),
            "설치-1",
            "중복 제목 dedupe가 동작해야 한다"
        );
        assert_eq!(
            a.anchorize("한글 제목 with English"),
            "한글-제목-with-english"
        );
        let _ = Path::new("");
    }
}
