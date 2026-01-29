use std::str::FromStr;

use anyhow::{anyhow, bail};
use ratatui::{
    widgets::ListState,
};
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Node, Selector};

use crate::client::{
    fetcher::get_req, page_part::Part, parser::{Link, ParsedContent, ParsedPage, ParserTrait}, WebClientTrait
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

    async fn fetch_url(
        &self,
        url: Url
    ) -> anyhow::Result<super::parser::ParsedPage> {
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
        let parts: Vec<Part> = vec![];
        let mut page_str: String = String::new().to_owned();

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
                    parsed_content: ParsedContent::Text(self.data.clone()),
                    state,
                    ..Default::default()
                });
            }
        };

        walk(&mut page_str, root, &url);

        // Post-process: collapse consecutive empty text parts
        let mut collapsed: Vec<Part> = Vec::with_capacity(parts.len());
        let mut last_empty = false;

        for p in parts.into_iter() {
            // Very simple check: if it's a text Part with empty text, skip duplicates
            let is_empty_text = matches!(
                // crude inspection via Debug strings isn't ideal; rely on Part::text("") we created
                // Better: add getters in Part, but keep minimal changes
                &p,
                Part { .. } if format!("{:?}", &p).contains("text: Some(\"\"")
            );
            if is_empty_text {
                if !last_empty {
                    collapsed.push(p);
                }
                last_empty = true;
            } else {
                collapsed.push(p);
                last_empty = false;
            }
        }

        collapsed.push(Part::text(page_str));

        // Ensure we have at least one item
        if collapsed.is_empty() {
            collapsed.push(Part::text(self.data.clone()));
        }

        // Initialize list state
        let mut state = ListState::default();
        state.select(Some(0));

        let doc_title = doc
            .select(&Selector::parse("title").map_err(|err| anyhow!(err.to_string()))?)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_else(|| "Title not found".to_string());

        Ok(ParsedPage {
            title: doc_title,
            url: url.to_string(),
            parsed_content:ParsedContent::Text(String::from("oi")),
            state,
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

fn push_non_empty_text(parts: &mut String, s: &str) {
    let text = s.trim();
    if !text.is_empty() {
        parts.push_str(text);
        parts.push(' ');
        // newline(parts);
    }
}

fn walk(parts: &mut String, el: ElementRef, base_url: &Url) {
    let name = el.value().name();
    if is_skippable(name) {
        return;
    }

    // For block separators, add spacing to improve readability
    let is_block = matches!(
        name,
        "p" | "div" | "section" | "article" | "main" | "header" | "footer" | "li" | "ul" | "ol"
    );

    // If this is an <a>, capture it as a link Part first (in order)
    if name == "a" {
        // Visible text of the link
        let link_text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();

        if let Some(href) = el.value().attr("href") {
            let resolved = base_url.join(href).unwrap_or_else(|_| base_url.clone());
            // TODO: make url rendering better
            let link = Link {
                title: link_text.clone(),
                text: link_text,
                url: resolved.to_string(),
            };
            parts.push_str(&link.text);
            parts.push(' ');
            // newline(parts);
        } else {
            // No href, treat as text
            let text = el.text().collect::<Vec<_>>().join(" ");
            push_non_empty_text(parts, &text);
        }
    } else if name == "p" {
        // TODO: make paragraph separation better here
        for child in el.children() {
            match child.value() {
                Node::Text(t) => {
                    push_non_empty_text(parts, t);
                    newline(parts);
                }
                Node::Element(_) => {
                    // Recurse into child elements
                    if let Some(child_ref) = ElementRef::wrap(child) {
                        walk(parts, child_ref, base_url);
                    }
                }
                _ => {}
            }
        }
    } else {
        // For other elements, gather direct text nodes first to preserve flow
        // Collect only text nodes that are immediate children (to avoid double-capturing)
        for child in el.children() {
            match child.value() {
                Node::Text(t) => {
                    push_non_empty_text(parts, t);
                }
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

    if is_block {
        // Add a small separator between blocks to keep reading flow
        parts.push_str("");
    }
}

/// Skip non-content tags
fn is_skippable(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "noscript" | "template" | "svg" | "canvas" | "iframe"
    )
}

fn newline(str: &mut String) {
    str.push('\n');
}
