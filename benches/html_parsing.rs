//! HTML Parsing Benchmarks
//!
//! These benchmarks measure the performance of the HTML5 parser.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use html2pdf::html::parse_html;

/// Generate HTML of specified size
fn generate_html(paragraphs: usize) -> String {
    let mut html = String::from("<!DOCTYPE html><html><head><title>Benchmark</title></head><body>");
    
    for i in 0..paragraphs {
        html.push_str(&format!(
            "<p>This is paragraph {} with <strong>bold</strong> and <em>italic</em> text.</p>\n",
            i
        ));
    }
    
    html.push_str("</body></html>");
    html
}

fn benchmark_small_document(c: &mut Criterion) {
    let html = generate_html(10);
    
    c.bench_function("html_parse_small_10p", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

fn benchmark_medium_document(c: &mut Criterion) {
    let html = generate_html(100);
    
    c.bench_function("html_parse_medium_100p", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

fn benchmark_large_document(c: &mut Criterion) {
    let html = generate_html(1000);
    
    c.bench_function("html_parse_large_1000p", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

fn benchmark_complex_document(c: &mut Criterion) {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex Document</title>
    <style>
        body { font-family: Arial; }
        .container { max-width: 800px; }
    </style>
</head>
<body>
    <header>
        <nav>
            <ul>
                <li><a href="#">Home</a></li>
                <li><a href="#">About</a></li>
                <li><a href="#">Contact</a></li>
            </ul>
        </nav>
    </header>
    <main>
        <article>
            <h1>Article Title</h1>
            <p>Content with <a href="#">links</a> and <em>formatting</em>.</p>
            <table>
                <tr><th>Header 1</th><th>Header 2</th></tr>
                <tr><td>Data 1</td><td>Data 2</td></tr>
            </table>
        </article>
    </main>
    <footer>
        <p>&copy; 2024</p>
    </footer>
</body>
</html>"#.to_string();
    
    c.bench_function("html_parse_complex", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_parse_scaling");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        let html = generate_html(*size);
        group.throughput(Throughput::Bytes(html.len() as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _ = parse_html(black_box(&html));
            });
        });
    }
    
    group.finish();
}

fn benchmark_attributes(c: &mut Criterion) {
    let html = r#"<div id="test" class="class1 class2 class3" data-value="123" data-other="456" style="color: red;" disabled>Content</div>"#.to_string();
    
    c.bench_function("html_parse_attributes", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

fn benchmark_entities(c: &mut Criterion) {
    let html = r#"<p>&lt;test&gt; &amp; &quot;quotes&quot; &apos;apostrophe&apos; &nbsp; &copy; &reg;</p>"#.to_string();
    
    c.bench_function("html_parse_entities", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

fn benchmark_nested_elements(c: &mut Criterion) {
    let mut html = String::from("<div>");
    for _ in 0..100 {
        html.push_str("<div>");
    }
    html.push_str("Content");
    for _ in 0..100 {
        html.push_str("</div>");
    }
    html.push_str("</div>");
    
    c.bench_function("html_parse_deeply_nested", |b| {
        b.iter(|| {
            let _ = parse_html(black_box(&html));
        });
    });
}

criterion_group!(
    benches,
    benchmark_small_document,
    benchmark_medium_document,
    benchmark_large_document,
    benchmark_complex_document,
    benchmark_scaling,
    benchmark_attributes,
    benchmark_entities,
    benchmark_nested_elements
);
criterion_main!(benches);
