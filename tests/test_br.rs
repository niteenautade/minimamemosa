fn main() {
    let md = "Line 1  \n  \n  \nLine 2";

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);

    let parser = pulldown_cmark::Parser::new_ext(md, options).map(|event| match event {
        pulldown_cmark::Event::SoftBreak => pulldown_cmark::Event::HardBreak,
        _ => event,
    });
    
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    
    println!("Markdown:\n{:?}", md);
    println!("HTML:\n{}", html);
}
