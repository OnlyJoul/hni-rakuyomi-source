#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, Manga, MangaPageResult, MangaStatus, Page, PageContent,
    Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, format, string::ToString, vec},
    imports::net::Request,
    register_source,
};
use serde_json::Value;

const BASE_URL: &str = "https://hni-scantrad.net";
const API_URL: &str = "https://hni-scantrad.net/api";
const MANGA_KEY: &str = "hajime-no-ippo";
const COVER: &str = "https://hni-scantrad.net/storage/img/cover/imfinippo.jpg";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:149.0) Gecko/20100101 Firefox/149.0";

struct HniScantrad;

fn request_json(url: &str) -> Result<Value> {
    let body = Request::get(url)?
        .header("Accept", "application/json, text/plain, */*")
        .header("Referer", BASE_URL)
        .header("User-Agent", USER_AGENT)
        .string()?;
    Ok(serde_json::from_str(&body)?)
}

fn string_any(v: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| v.get(*k).and_then(Value::as_str).map(ToString::to_string))
}

fn f32_any(v: &Value, keys: &[&str]) -> Option<f32> {
    keys.iter().find_map(|k| {
        let x = v.get(*k)?;
        if let Some(n) = x.as_f64() {
            Some(n as f32)
        } else {
            x.as_str()?.parse::<f32>().ok()
        }
    })
}

fn array_any<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Vec<Value>> {
    keys.iter().find_map(|k| v.get(*k).and_then(Value::as_array))
}

fn manga_stub() -> Manga {
    Manga {
        key: MANGA_KEY.into(),
        title: "Hajime no Ippo".into(),
        cover: Some(COVER.into()),
        artists: None,
        authors: None,
        description: None,
        url: Some(format!("{BASE_URL}/comics/{MANGA_KEY}")),
        tags: None,
        status: MangaStatus::Ongoing,
        content_rating: ContentRating::Safe,
        viewer: Viewer::RightToLeft,
        update_strategy: UpdateStrategy::Always,
        next_update_time: None,
        chapters: None,
    }
}

fn query_matches(query: Option<&str>) -> bool {
    let Some(q) = query else { return true };
    let q = q.trim().to_lowercase();
    q.is_empty()
        || "hajime no ippo".contains(&q)
        || "hajime-no-ippo".contains(&q)
        || q.contains("ippo")
        || q == "hni"
}

fn chapter_from_json(ch: &Value) -> Option<Chapter> {
    let key = string_any(ch, &["url", "key"])?;
    let title = string_any(ch, &["full_title", "fullTitle", "title", "name"])
        .or_else(|| Some("Hajime no Ippo".into()));
    let chapter_number = f32_any(ch, &["chapter", "chapter_number", "chapterNumber"]);
    let subchapter = f32_any(ch, &["subchapter", "sub_chapter", "subChapter"]).unwrap_or(0.0);
    let number = chapter_number.map(|n| n + subchapter / 10.0);

    let scanlators = array_any(ch, &["teams", "scanlators"]).map(|teams| {
        teams.iter()
            .filter_map(|t| string_any(t, &["name", "title"]))
            .collect::<Vec<String>>()
    });

    Some(Chapter {
        key: key.clone(),
        title,
        chapter_number: number,
        volume_number: f32_any(ch, &["volume", "volume_number", "volumeNumber"]),
        date_uploaded: None,
        scanlators,
        url: Some(format!("{BASE_URL}{key}")),
        language: Some("en".into()),
        thumbnail: None,
        locked: false,
    })
}

fn parse_details(mut manga: Manga, root: Value, needs_details: bool, needs_chapters: bool) -> Manga {
    let comic = root.get("comic").unwrap_or(&root);

    if needs_details {
        if let Some(title) = string_any(comic, &["title", "name"]) {
            manga.title = title;
        }
        manga.cover = string_any(comic, &["thumbnail", "cover", "image"]).or(manga.cover);
        manga.description = string_any(comic, &["description", "synopsis", "summary"]);
        manga.authors = string_any(comic, &["author"])
            .map(|s| vec![s]);
        manga.artists = string_any(comic, &["artist"])
            .map(|s| vec![s]);
    }

    if needs_chapters {
        let chapters = array_any(comic, &["chapters", "chapter_list", "chapterList"])
            .map(|items| {
                items.iter()
                    .filter_map(chapter_from_json)
                    .collect::<Vec<Chapter>>()
            })
            .unwrap_or_default();
        manga.chapters = Some(chapters);
    }

    manga
}

impl Source for HniScantrad {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        _page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let entries = if query_matches(query.as_deref()) {
            vec![manga_stub()]
        } else {
            Vec::new()
        };
        Ok(MangaPageResult { entries, has_next_page: false })
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let root = request_json(&format!("{API_URL}/comics/{MANGA_KEY}"))?;
        Ok(parse_details(manga, root, needs_details, needs_chapters))
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        // PizzaReader's API mirrors the public chapter path under /api.
        // Example key: /read/hajime-no-ippo/en/ch/1515
        let path = if chapter.key.starts_with('/') {
            chapter.key
        } else {
            format!("/{}", chapter.key)
        };
        let root = request_json(&format!("{API_URL}{path}"))?;
        let chapter_json = root.get("chapter").unwrap_or(&root);
        let pages = array_any(chapter_json, &["pages", "images"]).cloned().unwrap_or_default();

        Ok(pages
            .into_iter()
            .filter_map(|p| {
                let url = if let Some(s) = p.as_str() {
                    Some(s.to_string())
                } else {
                    string_any(&p, &["url", "src", "image"])
                }?;
                Some(Page {
                    content: PageContent::url(url),
                    ..Default::default()
                })
            })
            .collect())
    }
}

register_source!(HniScantrad);
