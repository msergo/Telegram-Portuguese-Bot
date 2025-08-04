use reqwest::Client;
use scraper::{ElementRef, Html, Node, Selector};

pub fn get_translation_table_header(lang_direction: &str) -> &'static str {
    match lang_direction {
        "pten" => "Traduções principais",
        "iten" => "Principal Translations/Traduzioni principali",
        _ => "Traduções principais", // default case
    }
}

pub fn get_raw_translations(body: &str, lang_direction: &str) -> String {
    let doc = Html::parse_document(body);
    let table_sel = Selector::parse("table.WRD").unwrap();
    let all_tables: Vec<_> = doc.select(&table_sel).collect();

    if all_tables.is_empty() {
        return String::new();
    }

    for table in all_tables {
        let header_sel = Selector::parse("tr").unwrap();
        let header_row = table.select(&header_sel).next();

        if let Some(header_row) = header_row {
            let header_text: String = header_row.text().collect::<Vec<_>>().join(" ");
            if header_text.contains(get_translation_table_header(lang_direction)) {
                // Return the HTML of the table as a String
                return table.html();
            }
        }
    }
    String::new()
}

pub fn get_translations(table_html: &str) -> String {
    let doc = Html::parse_document(table_html);
    let row_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let mut translations = String::new();

    for row in doc.select(&row_sel) {
        let tds: Vec<ElementRef> = row.select(&td_sel).collect();

        if tds.len() != 3 {
            continue; // skip rows that don't have exactly three columns
        }

        // TODO: improve formatting
        translations.push_str(&format!(
            "<b>{}</b> {} ⮕ {}\n",
            get_from_word_text(&tds[0]),
            tds[1].text().collect::<Vec<_>>().join(" "),
            get_translation_text(&tds[2])
        ));
    }

    translations
}

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

    return body;
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
