use reqwest::Client;
use scraper::{Html, Selector};

pub async fn fetch(word: &str) -> String {
    let url = format!("https://www.wordreference.com/pten/{}", word);
    let client = Client::new();

    let body = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0",
        )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let doc = Html::parse_document(&body);
    let table_sel = Selector::parse("table.WRD").unwrap();
    let row_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    // Result is the string of translations
    let mut translations = String::new();

    for table in doc.select(&table_sel) {
        let rows_vec: Vec<_> = table.select(&row_sel).collect();

        let mut rows = rows_vec.into_iter();
        if let Some(header_row) = rows.next() {
            let header_text: String = header_row.text().collect::<Vec<_>>().join(" ");
            if !header_text.contains("Traduções principais") {
                continue; // skip other tables
            }
        }

        for row in rows {
            if row
                .value()
                .attr("class")
                .map_or(false, |c| c.contains("langHeader"))
            {
                continue;
            }

            let tds: Vec<scraper::ElementRef> = row.select(&td_sel).collect();

            if tds.len() != 3 {
                continue; // skip rows that don't have exactly two columns
            }

            // TODO: improve formatting
            translations.push_str(&format!(
                "<b>{}</b> {} => {}\n",
                get_from_word_text(&tds[0]),
                tds[1].text().collect::<Vec<_>>().join(" "),
                get_translation_text(&tds[2])
            ));
        }

        break; // only first matching table
    }

    translations
}

fn get_from_word_text(td: &scraper::ElementRef) -> String {
    let strong_selector = scraper::Selector::parse("strong").unwrap();

    td.select(&strong_selector)
        .flat_map(|strong| {
            strong
                .text()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

use scraper::{ElementRef, Node};

fn get_translation_text(td: &ElementRef) -> String {
    td.children()
        .filter_map(|child| {
            match child.value() {
                Node::Element(e) => {
                    // Check if it's <em class="POS2"> — skip its text
                    // or class = "conjugate" in <a> tag
                    if e.name() == "a" && e.attr("class") == Some("conjugate") {
                        return None;
                    }

                    if e.name() == "em" {
                        if let Some(class) = e.attr("class") {
                            if class.contains("POS2") {
                                return None;
                            }
                        }
                    }
                    // Recursively extract text from allowed elements
                    ElementRef::wrap(child).map(|el| {
                        el.text()
                            .map(str::trim)
                            .filter(|t| !t.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                }
                Node::Text(text) => Some(text.trim().to_string()),
                _ => None,
            }
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
