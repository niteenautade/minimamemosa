fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        if c == '<' { in_tag = true; }
        else if c == '>' { in_tag = false; }
        else if !in_tag { out.push(c); }
    }
    out
}

fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_word = false;
    let mut tag_start = 0;
    let clean_content = strip_html(content).replace("&nbsp;", " ");
    let chars: Vec<char> = clean_content.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '#' && !in_word && (i == 0 || chars[i-1].is_whitespace() || chars[i-1] == '(') {
            in_word = true;
            tag_start = i + 1;
        } else if in_word {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                continue;
            } else {
                let tag_name: String = chars[tag_start..i].iter().collect();
                if !tag_name.is_empty() {
                    tags.push(tag_name.to_lowercase());
                }
                in_word = false;
            }
        }
    }
    if in_word {
        let tag_name: String = chars[tag_start..].iter().collect();
        if !tag_name.is_empty() {
            tags.push(tag_name.to_lowercase());
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn main() {
    let raw1 = "<p>#abc #xyz</p>";
    println!("tags 1: {:?}", extract_tags(raw1));
    let raw2 = "<p>&nbsp;#abc #xyz</p>";
    println!("tags 2: {:?}", extract_tags(raw2));
}
