fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    for c in s.chars() {
        match c {
            '<' => {
                if in_tag {
                    out.push('<');
                    out.push_str(&tag_buf);
                }
                in_tag = true; 
                tag_buf.clear(); 
            }
            '>' => {
                if in_tag {
                    in_tag = false;
                    let t = tag_buf.trim();
                    if t == "p" || t == "/p" || t == "br" || t == "/div" || t.starts_with("br ") || t.starts_with("/h") || t.starts_with("h") {
                        out.push('\n');
                    }
                } else {
                    out.push('>');
                }
            }
            _ => {
                if in_tag { tag_buf.push(c); }
                else { out.push(c); }
            }
        }
    }
    if in_tag {
        out.push('<');
        out.push_str(&tag_buf);
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
    let input = "# Portswigger SSTI\n\nDetect if any error using\n${{<%[%'\"}}%\\\n#portswigger #ssti";
    println!("strip_html:\n{}", strip_html(input));
    println!("tags: {:?}", extract_tags(input));
}
