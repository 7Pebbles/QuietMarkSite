use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::path::PathBuf;
use std::fs::create_dir_all;
use std::env;

struct PostMeta {
    title: String,
    author: String,
    pub_date: String,
    slug: String,
    draft: bool,
    featured: bool,
    tags: Vec<String>,
    description: String,
    body: String,
}



fn compute_total_size(path: &str) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let meta = entry.metadata();
            if let Ok(m) = meta {
                if m.is_file() {
                    total += m.len();
                } else if m.is_dir() {
                    total += compute_total_size(entry.path().to_str().unwrap());
                }
            }
        }
    }
    total
}

fn parse_post_file(path: &Path) -> Option<PostMeta> {
    let content = fs::read_to_string(path).ok()?;
    let mut lines = content.lines();

    if lines.next()? != "---" {
        return None;
    }

    let mut meta = PostMeta {
        title: String::new(),
        author: String::new(),
        pub_date: String::new(),
        slug: String::new(),
        draft: false,
        featured: false,
        tags: Vec::new(),
        description: String::new(),
        body: String::new(),
    };

    while let Some(line) = lines.next() {
        if line.trim() == "---" {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            match key {
                "title" => meta.title = value.to_string(),
                "author" => meta.author = value.to_string(),
                "pubDatetime" => meta.pub_date = format_date(value),
                "slug" => meta.slug = value.to_string(),
                "draft" => meta.draft = value == "true",
                "featured" => meta.featured = value == "true",
                "description" => meta.description = value.to_string(),
                "tags" => {
                    while let Some(tag_line) = lines.next() {
                        let tag_line = tag_line.trim();
                        if !tag_line.starts_with('-') {
                            break;
                        }
                        meta.tags.push(tag_line[1..].trim().to_string());
                    }
                }
                _ => {}
            }
        }
    }

    meta.body = lines.collect::<Vec<_>>().join("\n");
    if meta.draft {
        return None;
    }

    Some(meta)
}

fn format_date(raw: &str) -> String {
    if let Some(date_part) = raw.split('T').next() {
        let parts: Vec<&str> = date_part.split('-').collect();
        if parts.len() == 3 {
            let year = parts[0];
            let month = match parts[1] {
                "01" => "January", "02" => "February", "03" => "March", "04" => "April",
                "05" => "May", "06" => "June", "07" => "July", "08" => "August",
                "09" => "September", "10" => "October", "11" => "November", "12" => "December",
                _ => "Unknown"
            };
            let day = parts[2].trim_start_matches('0');
            return format!("{} {}, {}", month, day, year);
        }
    }
    raw.to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--clean".to_string()) {
        let _ = fs::remove_dir_all("dist");
    }
    fs::create_dir_all("dist/posts").unwrap();
    create_dir_all("dist/static").unwrap();

// Copy pico.css
let source_css = PathBuf::from("template/static/pico.css");
let dest_css = PathBuf::from("dist/static/pico.css");

if let Err(e) = fs::copy(&source_css, &dest_css) {
    eprintln!("Failed to copy pico.css: {}", e);
} else {
    println!("\u{2713} Copied: pico.css → dist/static/");
}
    
    let template = fs::read_to_string("template/post.html").expect("Missing template/post.html");

    let mut post_links = Vec::new();
    let mut featured_links = Vec::new();

    for entry in fs::read_dir("content").unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Some(post) = parse_post_file(&path) {
                    let rendered_body = render_markdown(&post.body);
                    let tags_html = post.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");

                    let page = template
                        .replace("{{ content }}", &rendered_body)
                        .replace("{{ title }}", &post.title)
                        .replace("{{ author }}", &post.author)
                        .replace("{{ pub_date }}", &post.pub_date)
                        .replace("{{ description }}", &post.description)
                        .replace("{{ tags }}", &tags_html);

                    let output_path = format!("dist/posts/{}.html", post.slug);
                    let mut file = File::create(&output_path).unwrap();
                    file.write_all(page.as_bytes()).unwrap();

                    println!("\u{2713} Generated: {}", output_path); // ✓
                    post_links.push((post.title.clone(), format!("dist/posts/{}.html", post.slug)));

if post.featured {
    featured_links.push((post.title.clone(), post.description.clone(), format!("dist/posts/{}.html", post.slug)));
}
                }
            }
        }
    }

    let posts_template = fs::read_to_string("template/posts.html").expect("Missing template/posts.html");

    let mut links = String::new();

    for (title, link) in &post_links {
        links.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", link, title));
    }

    let posts_output = posts_template.replace("{{ post_links }}", &links);
    fs::write("dist/posts.html", posts_output).unwrap();

    // Generate index.html with featured posts
let index_template = fs::read_to_string("template/index.html").expect("Missing index.html");

let mut featured_html = String::new();
for (title, desc, link) in &featured_links {
    featured_html.push_str(&format!(
        r#"<li><a href="{}"><strong>{}</strong></a><br><small>{}</small></li>"#,
        link, title, desc
    ));
}

let index_output = index_template.replace("{{ featured_posts }}", &featured_html);
fs::write("dist/index.html", index_output).unwrap();

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    println!("\u{2192} Listening on http://127.0.0.1:3000"); // →

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);
    let path = extract_path(&request);

    let file_path = if path == "/" {
        "dist/index.html".to_string()
    } else {
        format!("dist/{}", &path[1..])
    };

    let mut contents = fs::read_to_string(&file_path).unwrap_or_else(|_| fs::read_to_string("dist/404.html").unwrap_or_else(|_| "<h1>404 Not Found</h1>".to_string()));


    if file_path == "dist/index.html" {
        let total_bytes = compute_total_size(".");
        let (size_str, unit) = if total_bytes >= 1024 * 1024 {
            (format!("{:.2}", total_bytes as f64 / (1024.0 * 1024.0)), "MB")
        } else {
            (format!("{:.1}", total_bytes as f64 / 1024.0), "KB")
        };
        let dynamic_header = format!("Welcome to my {}{} of the internet", size_str, unit);
        contents = contents.replace(
            r#"<h2 id="site-size">Welcome...</h2>"#,
            &format!(r#"<h2 id="site-size">{}</h2>"#, dynamic_header),
        );
    }

    let status_line = if Path::new(&file_path).exists() {
        "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 404 NOT FOUND"
    };
    
    let content_type = get_content_type(&file_path);
    let response = format!(
        "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
}

fn extract_path(request: &str) -> &str {
    request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
}

fn get_content_type(path: &str) -> &str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "text/plain"
    }
}

fn render_markdown(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;
    let mut list_type = "";

    for line in md.lines() {
        let line = line.trim();

        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre>\n");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(line);
            html.push('\n');
            continue;
        }

        if line.is_empty() {
            if in_list {
                html.push_str(&format!("</{}>\n", list_type));
                in_list = false;
                list_type = "";
            }
            continue;
        }

        if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", &line[4..]));
        } else if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", &line[3..]));
        } else if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", &line[2..]));
        } else if line.starts_with("> ") {
            html.push_str(&format!("<blockquote>{}</blockquote>\n", &line[2..]));
        } else if line.starts_with("- ") {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
                list_type = "ul";
            }
            html.push_str(&format!("<li>{}</li>\n", format_inline(&line[2..])));
        } else if let Some(num) = line.split('.').next() {
            if num.chars().all(char::is_numeric) && line[num.len()..].starts_with(". ") {
                if !in_list {
                    html.push_str("<ol>\n");
                    in_list = true;
                    list_type = "ol";
                }
                html.push_str(&format!("<li>{}</li>\n", format_inline(&line[(num.len() + 2)..])));
            } else {
                html.push_str(&format!("<p>{}</p>\n", format_inline(line)));
            }
        } else {
            html.push_str(&format!("<p>{}</p>\n", format_inline(line)));
        }
    }

    if in_list {
        html.push_str(&format!("</{}>\n", list_type));
    }

    if in_code_block {
        html.push_str("</pre>\n"); // auto-close unbalanced block
    }

    html
}

fn format_inline(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'*') {
            chars.next(); // consume second *
            let mut bold = String::new();
            while let Some(&next) = chars.peek() {
                if next == '*' {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        break;
                    } else {
                        bold.push('*');
                    }
                } else {
                    bold.push(next);
                    chars.next();
                }
            }
            result.push_str(&format!("<strong>{}</strong>", bold));
        } else {
            result.push(c);
        }
    }

    result
}



