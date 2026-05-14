use std::fmt::Display;

use anyhow::{Result, anyhow, bail};
use ratatui::{style::Stylize, text::Text};
use reqwest::{Client, Response, Url};
use scraper::{ElementRef, Html, Node, Selector};

use crate::client::{
    WebClientTrait,
    fetcher::get_req,
    parser::{Link, ParsedContent, ParsedPage, ParserTrait},
};

const MAX_PAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_PAGE_LINKS: usize = 1500;

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
        _client: &Client,
    ) -> anyhow::Result<super::parser::ParsedPage> {
        bail!("FetchUrl does not implement searching")
    }

    async fn fetch_url(
        &self,
        url: Url,
        tab_id: i32,
        client: &Client,
    ) -> anyhow::Result<super::parser::ParsedPage> {
        let req = get_req(client, url.clone()).await?;

        if !req.status().is_success() {
            return Err(anyhow!("URL Returned Error!\n {}", req.text().await?));
        }

        let mut f = FetchUrl::new(url.clone());

        // tries to read HTML with limit
        let text = read_capped_body(req).await?;
        f.data = text;
        f.to_parsed_page(url, tab_id)
    }
}

impl FetchUrl {
    /// Handles getting specific selectors from HTML
    pub fn html_selector<'h>(&self, html: &'h Html) -> Result<ElementRef<'h>> {
        let main_sel =
            Selector::parse("main, article, body").map_err(|e| anyhow!(e.to_string()))?;

        if let Some(el) = html.select(&main_sel).find(|node| {
            let style = node.value().attr("style").unwrap_or("");
            // hiding hidden elements
            !style.contains("display: none")
                && !style.contains("display:none")
                && !style.contains("visibility: hidden")
                && !style.contains("visibility:hidden")
        }) {
            Ok(el)
        } else {
            // Fallback to document root as ElementRef by selecting html tag if present
            // if let Some(html_el) = html.select(&Selector::parse("html").unwrap()).next() {
            //     html_el
            // } else {
            //     // If no html tag, bail out with simple text
            //     return Err(anyhow!("no HTML tag"));
            // }
            Err(anyhow!("Invalid HTML"))
        }
    }
}

impl ParserTrait for FetchUrl {
    fn to_parsed_page(&self, url: Url, tab_id: i32) -> anyhow::Result<super::parser::ParsedPage> {
        let mut page_str: Text = Text::from("");
        let mut page_links: Vec<Link> = vec![];

        let doc = Html::parse_document(&self.data);

        let to_readable = self.html_selector(&doc)?;

        walk(&mut page_str, to_readable, &url, &mut page_links);

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

/// limits page size
async fn read_capped_body(mut req: Response) -> anyhow::Result<String> {
    let mut buf = Vec::with_capacity(64 * 1024);

    while let Some(chunk) = req.chunk().await? {
        if buf.len() + chunk.len() > MAX_PAGE_BYTES {
            bail!("Page payload exceeded {} bytes", MAX_PAGE_BYTES);
        }
        buf.extend_from_slice(&chunk);
    }

    Ok(String::from_utf8_lossy(&buf).into_owned())
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
        // TODO: implement inheritance
        // <h1> <span></span></h1> should also be treated as root h1
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
        if let Some(child_ref) = ElementRef::wrap(child)
            && child_ref.value().name() == "li"
        {
            let bullet = if ordered {
                Some(format!("{}. ", index))
            } else {
                Some("• ".to_string())
            };
            render_list_item(parts, child_ref, base_url, bullet.as_deref(), page_links);
            index += 1;
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

    if let Some(style) = el.value().attr("style")
        && (style.contains("display: none")
            || style.contains("visibility: hidden")
            || style.contains("visibility:hidden")
            || style.contains("display:none"))
    {
        return true;
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
    parts.push_span(s.to_string().underlined().bold());
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
    parts.push_span(s.to_string().italic());
    parts.push_span(" ");
}

fn push_bold<S>(parts: &mut Text, s: S)
where
    S: Display + ToString,
{
    parts.push_span(s.to_string().bold());
    parts.push_span(" ");
}

fn push_link_segment(segments: &mut Vec<Link>, label: String, url: String) {
    if segments.len() >= MAX_PAGE_LINKS {
        return;
    }

    if !label.trim().is_empty() {
        segments.push(Link {
            title: label.clone(),
            text: label,
            url,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::client::{
        fetch_url::{FetchUrl, walk},
        parser::Link,
    };
    use anyhow::Result;
    use ratatui::text::Text;
    use reqwest::Url;
    use scraper::Html;
    use std::str::FromStr;

    fn return_common() -> (Text<'static>, Vec<Link>, FetchUrl, Url) {
        let parts: Text = Text::from("");
        let page_links: Vec<Link> = vec![];
        let url = Url::from_str("https://example.com").expect("invalid URL");
        let f = FetchUrl::new(url.clone());
        (parts, page_links, f, url)
    }

    #[test]
    fn test_href() -> Result<()> {
        let (mut parts, mut page_links, f, url) = return_common();

        let val = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
            </head>
            <body>
            <a href="http://example.com">Example domain</a>
            <a href="http://example.com">Example domain</a>
            </body>
            </html>
            "#;

        let html = Html::parse_document(val);
        let el = f.html_selector(&html)?;

        walk(&mut parts, el, &url, &mut page_links);

        assert_eq!(page_links.len(), 2);

        for i in page_links.iter() {
            assert_eq!(i.url, String::from("http://example.com/"));
            assert_eq!(i.text, String::from("Example domain"));
        }

        Ok(())
    }

    #[test]
    fn test_ignore_fetchurl() -> Result<()> {
        let (mut parts, mut page_links, f, url) = return_common();
        let val = r#"
        <!DOCTYPE html>
                <html lang="en">
                <head>
                  <meta charset="UTF-8">
                  <meta name="viewport" content="width=device-width, initial-scale=1.0">
                  <title>Document</title>
                    <script>
                      console.log("oi")
                    </script>
                </head>
                <body>
                    <nav>
                      <ol>
                        <li></li>
                      </ol>
                    </nav>
                    <div class="dropdown">
                    </div>
                    <div hidden>
                    </div>
                    <input type="text" name="inputname" value="s">
                    <form>
                      
                    <label for="test"></label>
                    </form>
                    
                </body>

                </html>

        "#;
        let html = Html::parse_document(val);
        let el = f.html_selector(&html)?;

        walk(&mut parts, el, &url, &mut page_links);
        println!("{:#?}", parts);
        assert_eq!(parts.iter().len(), 1);

        Ok(())
    }

    #[test]
    fn nested_html_test() -> Result<()> {
        let (mut parts, mut page_links, f, url) = return_common();
        let val = r#"
        <html lang="en">
        <body>
            <h1>h1</h1> // +2
            <h2>h2</h2> // +1 
            <div> // +1 
              <div> // +1
                <p>div div p</p> // +1
              </div>
            </div>
            <div>   // +1
              div oi
            </div>
            <ul>
              <li>li1</li> +2
              <li>li2</li> +2 
            </ul> 
        </body> +2
        </html>
        "#;
        let html = Html::parse_document(val);
        let el = f.html_selector(&html)?;

        walk(&mut parts, el, &url, &mut page_links);
        assert_eq!(parts.iter().len(), 13);

        println!("{:#?}", parts);
        Ok(())
    }

    #[test]
    fn simple_html_test() -> Result<()> {
        let (mut parts, mut page_links, f, url) = return_common();
        let val = r#"
        <!DOCTYPE html>
        <html lang="en">
            <body>
            oi
            </body>
        </html>
        "#;
        let html = Html::parse_document(val);
        let el = f.html_selector(&html)?;

        walk(&mut parts, el, &url, &mut page_links);
        println!("{:?}", parts);
        assert_eq!(parts.iter().len(), 1);

        Ok(())
    }
}
