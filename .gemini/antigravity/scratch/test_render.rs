// Quick test to verify pulldown_cmark rendering of &nbsp;
fn main() {
    let test_cases = vec![
        ("Simple paragraphs", "Line 1\n\nLine 2"),
        ("With nbsp blank line", "Line 1\n\n&nbsp;\n\nLine 2"),
        ("With two nbsp blank lines", "Line 1\n\n&nbsp;\n\n&nbsp;\n\nLine 2"),
        ("Three lines with blanks", "Line 1\n\n&nbsp;\n\nLine 2\n\n&nbsp;\n\n&nbsp;\n\nLine 3"),
    ];

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    for (label, md) in test_cases {
        let parser = pulldown_cmark::Parser::new_ext(md, options).map(|event| match event {
            pulldown_cmark::Event::SoftBreak => pulldown_cmark::Event::HardBreak,
            _ => event,
        });
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        
        println!("=== {} ===", label);
        println!("Markdown:\n{}\n", md);
        println!("HTML:\n{}", html);
        let p_count = html.matches("<p>").count();
        println!("Paragraph count: {}", p_count);
        println!();
    }
}
