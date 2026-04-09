use std::fmt::Display;

use anyhow::{anyhow, bail};
use ratatui::{style::Stylize, text::Text, widgets::ListState};
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Node, Selector};

use crate::client::{
    WebClientTrait,
    fetcher::get_req,
    parser::{Link, ParsedContent, ParsedPage, ParserTrait},
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
        _tab_id: i32,
    ) -> anyhow::Result<super::parser::ParsedPage> {
        bail!("FetchUrl does not implement searching")
    }

    async fn fetch_url(&self, url: Url, tab_id: i32) -> anyhow::Result<super::parser::ParsedPage> {
        let client = Client::builder().user_agent(APP_USER_AGENT).build()?;
        let req = get_req(client, url.clone()).await?;

        if !req.status().is_success() {
            return Err(anyhow!("URL Returned Error!\n {}", req.text().await?));
        }

        let mut f = FetchUrl::new(url.clone());

        let text = req.text().await?;
        f.data = text;
        f.to_parsed_page(url, tab_id)
    }
}

impl ParserTrait for FetchUrl {
    fn to_parsed_page(&self, url: Url, tab_id: i32) -> anyhow::Result<super::parser::ParsedPage> {
        let mut page_str: Text = Text::from("");
        let mut page_links: Vec<Link> = vec![];

        let doc = Html::parse_document(&self.data);

        let main_sel =
            Selector::parse("main, article, body").map_err(|e| anyhow!(e.to_string()))?;

        let visible: Vec<ElementRef> = doc
            .select(&main_sel)
            .filter(|node| {
                let style = node.value().attr("style").unwrap_or("");
                !style.contains("display: none")
                    || !style.contains("display:none")
                    || !style.contains("visibility: hidden")
                    || !style.contains("visibility:hidden")
            })
            .collect();

        // let start_nodes: Vec<ElementRef> = doc.select(&main_sel).collect();

        // let root = if let Some(el) = start_nodes.first() {
        let root = if let Some(el) = visible.first() {
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

        walk(&mut page_str, root, &url, &mut page_links);

        let raw_str = page_str.to_string();

        let doc_title = doc
            .select(&Selector::parse("title").map_err(|err| anyhow!(err.to_string()))?)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_else(|| "Title not found".to_string());

        Ok(ParsedPage {
            tab_id,
            title: doc_title,
            url: url.to_string(),
            page_links,
            parsed_content: ParsedContent::Text(page_str),
            raw_text: raw_str,
            ..Default::default()
        })
    }
}

impl FetchUrl {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            data: String::new(),
        }
    }
}

/// main recursive func. to handle element/page rendering
fn walk(parts: &mut Text, el: ElementRef, base_url: &Url, page_links: &mut Vec<Link>) {
    // TODO: ignore non-text empty divs (currently, pages and being rendered with a bunch of new lines)
    let name = el.value().name();

    if is_skippable(name) || should_skip(&el) {
        return;
    }

    match name {
        "p" | "section" | "article" | "main" | "div" => {
            push_newline(parts);
            iter_items(parts, el, base_url, page_links);
        }
        "ul" | "ol" => {
            let is_ol = name == "ol";
            handle_list(parts, el, base_url, is_ol, page_links)
        }
        "table" | "tbody" => {}
        "h1" | "h2" | "h3" => {
            push_newline(parts);
            iter_items(parts, el, base_url, page_links);
        }
        "img" => {
            push_non_empty_text(parts, "Image");
        }
        "a" => {
            handle_links(parts, el, base_url, page_links);
        }
        _ => {
            iter_items(parts, el, base_url, page_links);
        }
    }
}

fn handle_list(
    parts: &mut Text,
    el: ElementRef,
    base_url: &Url,
    ordered: bool,
    page_links: &mut Vec<Link>,
) {
    let title = el
        .value()
        .attr("aria-label")
        .map(|s| s.to_string())
        .or_else(|| el.value().attr("title").map(|s| s.to_string()))
        .or_else(|| {
            // Find first non-li child that is likely a title
            el.children().find_map(|child| {
                if let Some(child_ref) = ElementRef::wrap(child) {
                    let name = child_ref.value().name();
                    if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p") {
                        let t = child_ref
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string();
                        if !t.is_empty() {
                            return Some(t);
                        }
                    }
                }
                None
            })
        });

    // If title present, render it bold and add spacing
    if let Some(t) = title {
        push_bold(parts, t);
        push_newline(parts);
    } else {
        push_newline(parts);
    }

    // Render each list item
    let mut index = 1;
    for child in el.children() {
        if let Some(child_ref) = ElementRef::wrap(child) {
            if child_ref.value().name() == "li" {
                let bullet = if ordered {
                    Some(format!("{}. ", index))
                } else {
                    Some("• ".to_string())
                };
                render_list_item(parts, child_ref, base_url, bullet.as_deref(), page_links);
                index += 1;
            }
        }
    }

    // Add a blank line after the list for visual separation
    push_newline(parts);
}

fn render_list_item(
    parts: &mut Text,
    li: ElementRef,
    base_url: &Url,
    bullet: Option<&str>,
    page_links: &mut Vec<Link>,
) {
    push_newline(parts);
    // Prefix with bullet/number if provided
    if let Some(b) = bullet {
        push_non_empty_text(parts, b);
    }

    // Collect all descendant text nodes, respecting simple inline formatting
    iter_items(parts, li, base_url, page_links);

    // End the list item with a newline
    push_newline(parts);
}

fn handle_links(parts: &mut Text, el: ElementRef, base_url: &Url, page_links: &mut Vec<Link>) {
    // Visible text of the link
    let link_text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
    let label = if link_text.is_empty() {
        "link".to_string()
    } else {
        link_text
    };

    if let Some(href) = el.value().attr("href") {
        let resolved = base_url.join(href).unwrap_or_else(|_| base_url.clone());
        // push_italic(parts, format!("({})",resolved));
        parts.push_span(label.clone().blue().underlined());
        push_link_segment(page_links, label, resolved.to_string());
    } else {
        // No href, treat as text
        let text = el.text().collect::<Vec<_>>().join(" ");
        push_non_empty_text(parts, text);
    }
}

fn should_skip(el: &ElementRef) -> bool {
    if el.value().attr("hidden").is_some() {
        return true;
    }

    if let Some(class) = el.value().attr("class") {
        if class.contains("aria-hidden=\"true\"") {
            // ignore aria-hidden
            return true;
        }
        if class.contains("dropdown") {
            // ignore dropdowns
            return true;
        }
    }

    if let Some(style) = el.value().attr("style") {
        if style.contains("display: none")
            || style.contains("visibility: hidden")
            || style.contains("visibility:hidden")
            || style.contains("display:none")
        {
            return true;
        }
    }
    false
}

/// main fn for filtering how tags are rendered
fn iter_items(parts: &mut Text, el: ElementRef, base_url: &Url, page_links: &mut Vec<Link>) {
    let name = el.value().name();

    if is_skippable(name) || should_skip(&el) {
        return;
    }

    for child in el.children() {
        match child.value() {
            Node::Text(t) => match name {
                "b" | "strong" => push_bold(parts, t.to_string()),
                "i" => push_italic(parts, t.to_string()),
                "h1" | "h2" | "h3" => push_underline_bold(parts, t.to_string()),
                _ => push_non_empty_text(parts, t.to_string()),
            },
            Node::Element(_) => {
                // Recurse into child elements
                if let Some(child_ref) = ElementRef::wrap(child) {
                    walk(parts, child_ref, base_url, page_links);
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
        "script"
            | "style"
            | "noscript"
            | "template"
            | "svg"
            | "canvas"
            | "iframe"
            | "input"
            | "nav"
            | "label"
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

fn push_link_segment(segments: &mut Vec<Link>, label: String, url: String) {
    if !label.trim().is_empty() {
        segments.push(Link {
            title: label.clone(),
            text: label,
            url,
        });
    }
}
