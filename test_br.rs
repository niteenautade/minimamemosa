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
fn main() {
    println!("{:?}", strip_html("<br/>"));
    println!("{:?}", strip_html("<br />"));
}
