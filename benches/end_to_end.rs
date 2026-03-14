//! End-to-End Benchmarks
//!
//! These benchmarks measure the complete HTML to PDF conversion pipeline.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use html2pdf::html_to_pdf;

fn get_simple_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head><title>Simple</title></head>
<body>
    <h1>Hello World</h1>
    <p>This is a simple test document.</p>
</body>
</html>"#.to_string()
}

fn get_medium_html() -> String {
    let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head><title>Medium</title></head>
<body>
    <h1>Document Title</h1>
"#);
    
    for i in 0..20 {
        html.push_str(&format!(
            r#"<h2>Section {}</h2>
            <p>This is paragraph {} with <strong>bold</strong> and <em>italic</em> text.</p>
            <ul>
                <li>Item A</li>
                <li>Item B</li>
                <li>Item C</li>
            </ul>
"#,
            i + 1,
            i + 1
        ));
    }
    
    html.push_str("</body></html>");
    html
}

fn get_complex_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex</title>
    <style>
        body { font-family: Arial; margin: 40px; }
        .header { background: #333; color: white; padding: 20px; }
        .content { max-width: 800px; margin: 0 auto; }
        .card { border: 1px solid #ddd; padding: 20px; margin: 20px 0; }
        table { width: 100%; border-collapse: collapse; }
        th, td { border: 1px solid #ddd; padding: 12px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Complex Document</h1>
    </div>
    <div class="content">
        <p>This document includes various HTML elements and CSS styling.</p>
        
        <div class="card">
            <h2>Card Title</h2>
            <p>Card content with <a href="#">links</a> and formatting.</p>
        </div>
        
        <table>
            <thead>
                <tr><th>Name</th><th>Value</th></tr>
            </thead>
            <tbody>
                <tr><td>Item 1</td><td>100</td></tr>
                <tr><td>Item 2</td><td>200</td></tr>
                <tr><td>Item 3</td><td>300</td></tr>
            </tbody>
        </table>
        
        <h2>Lists</h2>
        <ol>
            <li>First item</li>
            <li>Second item</li>
            <li>Third item</li>
        </ol>
    </div>
</body>
</html>"#.to_string()
}

fn get_css_heavy_html() -> String {
    let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <title>CSS Heavy</title>
    <style>
"#);
    
    // Generate lots of CSS rules
    for i in 0..100 {
        html.push_str(&format!(
            ".class-{} {{ color: #{:06x}; font-size: {}px; margin: {}px; padding: {}px; }}\n",
            i,
            i * 1000 % 0xFFFFFF,
            10 + (i % 20),
            i % 50,
            i % 30
        ));
    }
    
    html.push_str(r#"
    </style>
</head>
<body>
    <h1>CSS Heavy Document</h1>
"#);
    
    for i in 0..50 {
        html.push_str(&format!(r#"<div class="class-{}">Content {}</div>"#, i, i + 1));
    }
    
    html.push_str("</body></html>");
    html
}

fn benchmark_simple_conversion(c: &mut Criterion) {
    let html = get_simple_html();
    
    c.bench_function("e2e_simple_html", |b| {
        b.iter(|| {
            let _ = html_to_pdf(black_box(&html));
        });
    });
}

fn benchmark_medium_conversion(c: &mut Criterion) {
    let html = get_medium_html();
    
    c.bench_function("e2e_medium_html", |b| {
        b.iter(|| {
            let _ = html_to_pdf(black_box(&html));
        });
    });
}

fn benchmark_complex_conversion(c: &mut Criterion) {
    let html = get_complex_html();
    
    c.bench_function("e2e_complex_html", |b| {
        b.iter(|| {
            let _ = html_to_pdf(black_box(&html));
        });
    });
}

fn benchmark_css_heavy_conversion(c: &mut Criterion) {
    let html = get_css_heavy_html();
    
    c.bench_function("e2e_css_heavy", |b| {
        b.iter(|| {
            let _ = html_to_pdf(black_box(&html));
        });
    });
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_scaling");
    
    for paragraph_count in [10, 50, 100, 500].iter() {
        let mut html = String::from("<!DOCTYPE html><html><body>");
        for i in 0..*paragraph_count {
            html.push_str(&format!("<p>Paragraph {} with some text content.</p>", i + 1));
        }
        html.push_str("</body></html>");
        
        group.throughput(Throughput::Bytes(html.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(paragraph_count),
            paragraph_count,
            |b, _| {
                b.iter(|| {
                    let _ = html_to_pdf(black_box(&html));
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_fixture_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("e2e_fixtures");
    
    let fixtures = vec![
        ("simple", "tests/fixtures/simple.html"),
        ("css_test", "tests/fixtures/css_test.html"),
        ("printcss", "tests/fixtures/printcss_test.html"),
        ("complex", "tests/fixtures/complex_layout.html"),
    ];
    
    for (name, path) in fixtures {
        if std::path::Path::new(path).exists() {
            let html = std::fs::read_to_string(path).unwrap();
            group.throughput(Throughput::Bytes(html.len() as u64));
            
            group.bench_function(format!("fixture_{}", name), |b| {
                b.iter(|| {
                    let _ = html_to_pdf(black_box(&html));
                });
            });
        }
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_simple_conversion,
    benchmark_medium_conversion,
    benchmark_complex_conversion,
    benchmark_css_heavy_conversion,
    benchmark_scaling,
    benchmark_fixture_files
);
criterion_main!(benches);
