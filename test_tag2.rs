fn extract_title(content: &str) -> (String, String) {
    let trimmed = content.trim();
    let text = trimmed.to_string(); // Simplified strip_html
    let first_line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim().to_string();
    let clean = first_line.trim_start_matches(|c| c == '#' || c == ' ' || c == '*' || c == '_').trim().to_string();
    let final_title = if clean.is_empty() { "Note".to_string() } else { clean };
    (final_title, trimmed.to_string())
}

fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_word = false;
    let mut tag_start = 0;
    let chars: Vec<char> = content.chars().collect();
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
    let raw1 = "#abc #xyz";
    let (t1, c1) = extract_title(raw1);
    println!("c1: {}, tags: {:?}", c1, extract_tags(&c1));
    
    let raw2 = "tags #abc #xyz";
    let (t2, c2) = extract_title(raw2);
    println!("c2: {}, tags: {:?}", c2, extract_tags(&c2));
}
