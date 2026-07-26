//! 날짜와 Atom 피드.
//!
//! **Atom이고 RSS 2.0이 아니다.** RSS의 `<pubDate>`는 RFC 2822라서
//! `Tue, 26 Jul 2026 …` 처럼 **요일과 영어 월 이름**을 우리가 만들어 넣어야 한다.
//! 요일은 직접 계산해야 하고, 영어 월 이름은 한국어 피드에도 그대로 박힌다.
//! Atom은 RFC 3339를 쓰는데 그건 TOML 날짜가 이미 갖고 있는 모양이라, 코드가
//! 줄고 틀릴 자리도 줄어든다. 요즘 리더는 둘 다 읽는다.
//!
//! **날짜가 하나도 없는 언어에는 피드를 만들지 않는다.** 빈 피드는 없는 것보다
//! 나쁘다 — 구독자는 아무것도 오지 않는 걸 고장으로 읽는다.

use crate::content::Page;
use serde::Serialize;

pub const MAX_ENTRIES: usize = 20;

/// 해당 언어 피드의 출력 경로. 검색 색인과 같은 규칙을 쓴다.
pub fn path(language: &str) -> String {
    format!("feed-{language}.xml")
}

/// 템플릿에 나가는 날짜.
///
/// 포맷 필터를 만들지 않고 조각을 준다. `{{ page.date.year }}년 {{ page.date.month }}월`
/// 처럼 템플릿이 조립하면 되고, 그러면 날짜 서식 미니 언어를 하나 더 배울 필요도,
/// 우리가 로케일별 서식을 떠안을 일도 없다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    /// `2026-07-26` — `<time datetime>` 속성에 그대로 쓸 수 있다.
    pub date: String,
    /// `2026-07-26T00:00:00Z` 또는 `2026-07-26T09:00:00+09:00` — RFC 3339.
    pub iso: String,
    /// 시간대를 UTC로 옮긴 뒤의 정렬용 값. 서로 다른 오프셋의 순간을 비교하려면
    /// 표기 그대로가 아니라 같은 기준으로 봐야 한다.
    utc_minutes: i64,
}

impl PageDate {
    /// TOML 날짜에서 만든다. 시각이 없으면 자정 UTC로 본다 — Atom은 날짜만으로는
    /// 유효하지 않고, 하루 안에서의 순서는 어차피 저자가 정하지 않은 것이다.
    pub fn from_toml(dt: &toml::value::Datetime) -> Option<Self> {
        let d = dt.date?;
        let t = dt.time;
        let (h, mi, s) = t.map_or((0, 0, 0), |t| (t.hour, t.minute, t.second.unwrap_or(0)));

        // 오프셋은 **그대로 싣는다.** 한때 전부 `Z`로 바꿔 적었는데, 그건
        // `09:00+09:00`(한국 아침)을 `09:00Z`(한국 저녁)라고 발표하는 것이라
        // 최대 하루까지 틀린 순간이 나간다. RFC 3339는 어떤 오프셋도 받으므로
        // 시각 산술 없이 저자가 쓴 것을 그대로 두면 된다.
        let (suffix, off_min) = match dt.offset {
            Some(toml::value::Offset::Custom { minutes }) if minutes != 0 => {
                let (sign, m) = if minutes < 0 {
                    ('-', -i32::from(minutes))
                } else {
                    ('+', i32::from(minutes))
                };
                (
                    format!("{sign}{:02}:{:02}", m / 60, m % 60),
                    i64::from(minutes),
                )
            }
            _ => ("Z".to_string(), 0),
        };

        // 정렬은 UTC 기준으로 한다. 표기가 다른 두 순간을 문자열로 비교하면
        // `+09:00`이 붙은 아침이 `Z`가 붙은 저녁보다 뒤에 온다.
        let days = i64::from(d.year) * 372 + i64::from(d.month) * 31 + i64::from(d.day);
        let utc_minutes = days * 1440 + i64::from(h) * 60 + i64::from(mi) - off_min;

        Some(Self {
            year: d.year,
            month: d.month,
            day: d.day,
            date: format!("{:04}-{:02}-{:02}", d.year, d.month, d.day),
            iso: format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{suffix}",
                d.year, d.month, d.day, h, mi, s
            ),
            utc_minutes,
        })
    }

    /// 정렬용 키. UTC로 옮긴 분 단위 값이라 시각과 시간대까지 반영된다.
    /// 날짜만 비교하면 같은 날의 아침과 저녁이 구분되지 않고, 그러면 피드의
    /// `updated`가 자기 항목보다 과거가 되는 일이 생긴다.
    pub fn sort_key(&self) -> i64 {
        self.utc_minutes
    }
}

/// XML 텍스트/속성값 이스케이프.
///
/// ⚠️ 이스케이프만으로는 부족하다. XML 1.0은 C0 제어문자를 **문자 참조로도** 담을
/// 수 없다고 정한다. 제목에 그런 글자가 하나 섞이면 피드 전체가 파싱되지 않고,
/// 빌드는 성공했다고 말한다. 그래서 넣을 수 없는 건 지운다.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            c if c < '\u{20}' && c != '\t' && c != '\n' && c != '\r' => {}
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 날짜가 있는 페이지들로 Atom 피드를 만든다. 하나도 없으면 `None`.
///
/// `home_url`은 이 언어의 첫 페이지 URL이다(`/` 또는 `/ko/`).
pub fn atom(
    pages: &[&Page],
    language: &str,
    site_title: &str,
    origin: &str,
    home_url: &str,
    self_url: &str,
) -> Option<String> {
    let mut dated: Vec<(&Page, PageDate)> = pages
        .iter()
        .filter_map(|p| {
            p.front
                .date
                .as_ref()
                .and_then(PageDate::from_toml)
                .map(|d| (*p, d))
        })
        .collect();
    if dated.is_empty() {
        return None;
    }

    // 최신이 먼저. 같은 날이면 제목순 — 파일 순회 순서에 기대면 빌드가 결정적이지 않다.
    dated.sort_by(|a, b| {
        b.1.sort_key()
            .cmp(&a.1.sort_key())
            .then_with(|| a.0.title.cmp(&b.0.title))
    });

    dated.truncate(MAX_ENTRIES);
    // 정렬의 머리가 아니라 **최대값**이다. 같은 날 여러 글이 있으면 정렬은 제목으로
    // 갈리므로, 머리를 쓰면 피드가 자기 항목보다 과거 시각을 발표하게 된다.
    // 자르기 뒤에 계산해야 피드에 없는 항목의 시각을 쓰지 않는다.
    let updated = dated
        .iter()
        .max_by_key(|(_, d)| d.sort_key())
        .map(|(_, d)| d.iso.clone())
        .expect("비어 있지 않음을 위에서 확인했다");

    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str(&format!(
        "<feed xmlns=\"http://www.w3.org/2005/Atom\" xml:lang=\"{}\">\n",
        esc(language)
    ));
    out.push_str(&format!("  <title>{}</title>\n", esc(site_title)));
    // id는 영원히 바뀌지 않아야 하고, **다른 무엇과도 같으면 안 된다.** 홈 URL을
    // 쓰면 홈 페이지 entry의 id와 글자 그대로 같아져서, 리더가 피드와 그 안의 항목
    // 하나를 같은 것으로 본다. 피드 자기 주소는 그럴 일이 없다.
    out.push_str(&format!("  <id>{}{}</id>\n", esc(origin), esc(self_url)));
    out.push_str(&format!(
        "  <link rel=\"alternate\" href=\"{}{}\"/>\n",
        esc(origin),
        esc(home_url)
    ));
    out.push_str(&format!(
        "  <link rel=\"self\" href=\"{}{}\"/>\n",
        esc(origin),
        esc(self_url)
    ));
    out.push_str(&format!("  <updated>{}</updated>\n", esc(&updated)));
    // RFC 4287은 feed나 모든 entry 중 한쪽에 author를 **요구한다**. 없으면 검증기가
    // 무효로 판정하고, 일부 리더는 피드를 통째로 거른다. 사람 이름을 위한 설정 키를
    // 새로 만드는 대신 사이트 제목을 쓴다 — 사이트 피드의 저자는 사이트다.
    out.push_str(&format!(
        "  <author><name>{}</name></author>\n",
        esc(site_title)
    ));

    for (page, date) in &dated {
        let permalink = format!("{origin}{}", page.url);
        out.push_str("  <entry>\n");
        out.push_str(&format!("    <title>{}</title>\n", esc(&page.title)));
        out.push_str(&format!("    <id>{}</id>\n", esc(&permalink)));
        out.push_str(&format!(
            "    <link rel=\"alternate\" href=\"{}\"/>\n",
            esc(&permalink)
        ));
        out.push_str(&format!("    <updated>{}</updated>\n", esc(&date.iso)));
        if !page.front.description.is_empty() {
            out.push_str(&format!(
                "    <summary>{}</summary>\n",
                esc(&page.front.description)
            ));
        }
        out.push_str("  </entry>\n");
    }

    out.push_str("</feed>\n");
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> toml::value::Datetime {
        s.parse().unwrap()
    }

    #[test]
    fn a_date_without_a_time_becomes_midnight_utc() {
        // Atom은 날짜만으로는 유효하지 않다. `2026-07-26`을 그대로 내보내면
        // 리더가 항목을 조용히 버린다.
        let d = PageDate::from_toml(&dt("2026-07-26")).unwrap();
        assert_eq!(d.date, "2026-07-26");
        assert_eq!(d.iso, "2026-07-26T00:00:00Z");
        assert_eq!((d.year, d.month, d.day), (2026, 7, 26));
    }

    #[test]
    fn a_full_timestamp_keeps_its_time() {
        let d = PageDate::from_toml(&dt("2026-07-26T09:30:00Z")).unwrap();
        assert_eq!(d.iso, "2026-07-26T09:30:00Z");
    }

    /// 빈 피드는 없는 것보다 나쁘다. 구독자는 아무것도 오지 않는 걸 고장으로 읽는다.
    #[test]
    fn no_dated_pages_means_no_feed() {
        assert!(atom(&[], "en", "S", "https://e.com", "/", "/feed-en.xml").is_none());
    }

    #[test]
    fn sort_keys_order_by_calendar_not_by_string() {
        let a = PageDate::from_toml(&dt("2026-09-01")).unwrap();
        let b = PageDate::from_toml(&dt("2026-10-01")).unwrap();
        assert!(b.sort_key() > a.sort_key());
    }

    /// 오프셋을 전부 Z로 바꿔 적던 시절이 있었다. 그건 한국 아침 9시를 한국
    /// 저녁 6시라고 발표하는 것이라 최대 하루까지 틀린 순간이 나갔다.
    #[test]
    fn an_offset_is_carried_not_relabelled() {
        let d = PageDate::from_toml(&dt("2026-07-26T09:00:00+09:00")).unwrap();
        assert_eq!(d.iso, "2026-07-26T09:00:00+09:00");
        // 그리고 정렬은 UTC 기준이라, 같은 날 18:00Z가 09:00+09:00보다 뒤다.
        let z = PageDate::from_toml(&dt("2026-07-26T18:00:00Z")).unwrap();
        assert!(
            z.sort_key() > d.sort_key(),
            "표기가 아니라 순간으로 비교해야 한다"
        );
    }

    /// XML 1.0은 C0 제어문자를 문자 참조로도 담을 수 없다. 제목에 하나만 섞여도
    /// 피드 전체가 파싱되지 않는데, 빌드는 성공했다고 말한다.
    #[test]
    fn control_characters_are_dropped_not_escaped() {
        let out = esc("Bell \u{7} here\ttab");
        assert!(!out.contains('\u{7}'), "실제: {out:?}");
        assert!(out.contains('\t'), "탭은 XML에 넣을 수 있다: {out:?}");
        assert_eq!(out, "Bell  here\ttab");
    }
}
