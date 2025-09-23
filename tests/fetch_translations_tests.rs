use pt_dict_bot::fetch_translations::{
    get_raw_translations, get_translation_table_header, get_translations,
};

#[test]
fn test_get_translation_table_header() {
    assert_eq!(get_translation_table_header("pten"), "Traduções principais");
    assert_eq!(
        get_translation_table_header("iten"),
        "Principal Translations/Traduzioni principali"
    );
    // Default case
    assert_eq!(get_translation_table_header("xyz"), "Traduções principais");
}

#[test]
fn test_get_raw_translations_found() {
    // Simulate minimal WordReference-like HTML containing table with "Traduções principais"
    let body = r#"
        <html>
            <body>
                <table class="WRD">
                    <tr><td>Traduções principais</td></tr>
                    <tr><td>foo</td></tr>
                </table>
            </body>
        </html>
    "#;
    let result = get_raw_translations(body, "pten");
    assert!(result.contains("<table")); // Should return some table HTML
    assert!(result.contains("Traduções principais"));
}

#[test]
fn test_get_raw_translations_not_found() {
    let body = r#"
        <html>
            <body>
                <table class="WRD">
                    <tr><td>Not the header</td></tr>
                </table>
            </body>
        </html>
    "#;
    let result = get_raw_translations(body, "pten");
    assert_eq!(result, "");
}

#[test]
fn test_get_translations_basic() {
    let table_html = r#"
        <table class="WRD">
            <tr class>
                <td><strong>Português</strong></td>
                <td>nf</td>
                <td><strong>Inglês</strong></td>
            </tr>
            <tr class="odd">
                <td><strong>casa</strong></td>
                <td>nf</td>
                <td>house</td>
            </tr>
        </table>
    "#;
    let result = get_translations(table_html);
    assert!(
        result.contains("<b>casa</b> nf ⮕ house"),
        "Output: {}",
        result
    );
}

#[test]
fn test_get_translations_skips_bad_rows() {
    let table_html = r#"
        <table class="WRD">
            <tr>
                <td>only one td</td>
            </tr>
            <tr>
                <td>one</td><td>two</td><td>three</td>
            </tr>
        </table>
    "#;
    let result = get_translations(table_html);
    // Only the second row should be processed
    assert!(result.contains("⮕ three"));
    assert!(!result.contains("only one td"));
}
