use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, bail};
use ratatui::{style::Stylize, text::Text, widgets::ListState};
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Node, Selector};

use crate::client::{
    WebClientTrait,
    fetcher::get_req,
    parser::{ParsedContent, ParsedPage, ParserTrait},
};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Clone, PartialEq, Eq)]
pub struct FetchUrl {
    url: Url,
    /// Represents page content (HTML)
    data: String,
}

impl WebClientTrait for FetchUrl {
    async fn search(
        &self,
        _query: String,
        _state: &mut crate::state::webclient_state::WebClientState,
    ) -> anyhow::Result<super::parser::ParsedPage> {
        bail!("FetchUrl does not implement searching")
    }

    async fn fetch_url(&self, url: Url) -> anyhow::Result<super::parser::ParsedPage> {
        let client = Client::builder().user_agent(APP_USER_AGENT).build()?;
        let req = get_req(client, url.clone()).await?;

        if !req.status().is_success() {
            return Err(anyhow!("URL Returned Error!\n {}", req.text().await?));
        }
        let mut f = FetchUrl::default();

        let text = req.text().await?;
        f.data = text;
        f.to_parsed_page(url)
    }
}

impl ParserTrait for FetchUrl {
    fn to_parsed_page(&self, url: Url) -> anyhow::Result<super::parser::ParsedPage> {
        // let mut page_str: String = String::new().to_owned();
        let mut page_str: Text = Text::from("");

        let doc = Html::parse_document(&self.data);

        let main_sel =
            Selector::parse("main, article, body").map_err(|e| anyhow!(e.to_string()))?;
        let start_nodes: Vec<ElementRef> = doc.select(&main_sel).collect();

        let root = if let Some(el) = start_nodes.first() {
            *el
        } else {
            // Fallback to document root as ElementRef by selecting html tag if present
            if let Some(html_el) = doc.select(&Selector::parse("html").unwrap()).next() {
                html_el
            } else {
                // If no html tag, bail out with simple text
                let mut state = ListState::default();
                state.select(Some(0));

                return Ok(ParsedPage {
                    title: url.to_string(),
                    url: url.to_string(),
                    parsed_content: ParsedContent::Text(Text::from(self.data.clone())),
                    state,
                    ..Default::default()
                });
            }
        };

        walk(&mut page_str, root, &url);

        let doc_title = doc
            .select(&Selector::parse("title").map_err(|err| anyhow!(err.to_string()))?)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_else(|| "Title not found".to_string());

        Ok(ParsedPage {
            title: doc_title,
            url: url.to_string(),
            // parsed_content: ParsedContent::Text(page_str),
            parsed_content: ParsedContent::Text(page_str),
            ..Default::default()
        })
    }
}

impl Default for FetchUrl {
    fn default() -> Self {
        let url = Url::from_str("https://example.com").expect("Failed to load default URL");
        Self {
            url,
            data: String::new(),
        }
    }
}

fn walk(parts: &mut Text, el: ElementRef, base_url: &Url) {
    let name = el.value().name();
    if is_skippable(name) {
        return;
    }

    match name {
        "div" | "p" | "section" | "article" | "main" | "header" | "footer" | "li" | "ul" | "ol" => {
            push_newline(parts);
            iter_items(parts, el, base_url);
        }
        "h1" | "h2" | "h3" => {
            push_newline(parts);
            iter_items(parts, el, base_url);
        }
        "a" => {
            handle_links(parts, el, base_url);
        }
        _ => {
            iter_items(parts, el, base_url);
        }
    }
}

fn handle_links(parts: &mut Text, el: ElementRef, base_url: &Url) {
    // Visible text of the link
    let link_text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
    push_italic(parts, link_text + "(link)");

    if let Some(href) = el.value().attr("href") {
        // TODO: links make the page unreadable. Make this readable
        // this block would render link inside href
        let _resolved = base_url.join(href).unwrap_or_else(|_| base_url.clone());
        // push_italic(parts, format!("({})",resolved));
    } else {
        // No href, treat as text
        let text = el.text().collect::<Vec<_>>().join(" ");
        push_non_empty_text(parts, text);
    }
}

fn iter_items(parts: &mut Text, el: ElementRef, base_url: &Url) {
    let name = el.value().name();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => match name {
                "b" => push_bold(parts, t.to_string()),
                "i" => push_italic(parts, t.to_string()),
                "h1" | "h2" | "h3" => push_underline_bold(parts, t.to_string()),
                _ => push_non_empty_text(parts, t.to_string()),
            },
            Node::Element(_) => {
                // Recurse into child elements
                if let Some(child_ref) = ElementRef::wrap(child) {
                    walk(parts, child_ref, base_url);
                }
            }
            _ => {}
        }
    }
}

/// Skip non-content tags
fn is_skippable(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "noscript" | "template" | "svg" | "canvas" | "iframe"
    )
}

fn push_newline(parts: &mut Text) {
    parts.push_line("");
}

fn push_non_empty_text<S>(parts: &mut Text, s: S)
where
    S: Display + ToString,
{
    let text = s.to_string();
    if !text.trim().is_empty() {
        parts.push_span(text);
        parts.push_span(" ");
    } else {
        parts.push_span(" ");
    }
}

fn push_underline_bold<S>(parts: &mut Text, s: S)
where
    S: Display + ToString,
{
    parts.push_span(String::from(s.to_string()).underlined().bold());
    parts.push_span(" ");
}

// fn push_underline<S>(parts: &mut Text, s: S)
// where
//     S: Display + ToString,
// {
//     parts.push_span(String::from(s.to_string()).underlined());
//     parts.push_span(" ");
// }

fn push_italic<S>(parts: &mut Text, s: S)
where
    S: Display + ToString,
{
    parts.push_span(String::from(s.to_string()).italic());
    parts.push_span(" ");
}

fn push_bold<S>(parts: &mut Text, s: S)
where
    S: Display + ToString,
{
    parts.push_span(String::from(s.to_string()).bold());
    parts.push_span(" ");
}
