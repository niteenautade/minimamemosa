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
    println!("{:?}", extract_tags("#abc #xyz"));
    println!("{:?}", extract_tags("tags #abc #xyz"));
}
