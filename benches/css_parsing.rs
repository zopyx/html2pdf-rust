//! CSS Parsing Benchmarks
//!
//! These benchmarks measure the performance of the CSS parser.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use html2pdf::css::parse_stylesheet;

/// Generate CSS stylesheet of specified rule count
fn generate_stylesheet(rules: usize) -> String {
    let mut css = String::new();
    
    for i in 0..rules {
        css.push_str(&format!(
            ".class-{} {{
                color: #{:06x};
                background: #{:06x};
                font-size: {}px;
                margin: {}px;
                padding: {}px;
            }}\n",
            i,
            i * 1000 % 0xFFFFFF,
            i * 2000 % 0xFFFFFF,
            10 + (i % 20),
            i % 50,
            i % 30
        ));
    }
    
    css
}

fn benchmark_small_stylesheet(c: &mut Criterion) {
    let css = generate_stylesheet(10);
    
    c.bench_function("css_parse_small_10r", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

fn benchmark_medium_stylesheet(c: &mut Criterion) {
    let css = generate_stylesheet(100);
    
    c.bench_function("css_parse_medium_100r", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

fn benchmark_large_stylesheet(c: &mut Criterion) {
    let css = generate_stylesheet(1000);
    
    c.bench_function("css_parse_large_1000r", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

fn benchmark_complex_selectors(c: &mut Criterion) {
    let css = r#"
        div.container > ul.nav li a:hover { color: red; }
        section#main article.post h1.title + p:first-of-type { font-size: 1.2em; }
        .grid > *:nth-child(2n+1) { background: #f5f5f5; }
        input[type="text"]:focus, input[type="email"]:focus { border-color: blue; }
        #header .nav > li > a[href^="/"] { font-weight: bold; }
    "#.to_string();
    
    c.bench_function("css_parse_complex_selectors", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

fn benchmark_at_rules(c: &mut Criterion) {
    let css = r#"
        @import url("styles.css");
        @import url("print.css") print;
        
        @media screen and (min-width: 768px) {
            .container { max-width: 750px; }
        }
        
        @media print {
            body { color: black; }
        }
        
        @page {
            margin: 2cm;
            @top-center { content: "Header"; }
        }
        
        @font-face {
            font-family: "Custom";
            src: url("font.woff2");
        }
        
        @supports (display: grid) {
            .grid { display: grid; }
        }
        
        @keyframes fade {
            from { opacity: 0; }
            to { opacity: 1; }
        }
    "#.to_string();
    
    c.bench_function("css_parse_at_rules", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("css_parse_scaling");
    
    for size in [10, 50, 100, 500, 1000].iter() {
        let css = generate_stylesheet(*size);
        group.throughput(Throughput::Bytes(css.len() as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _ = parse_stylesheet(black_box(&css));
            });
        });
    }
    
    group.finish();
}

fn benchmark_css_functions(c: &mut Criterion) {
    let css = r#"
        .test1 { width: calc(100% - 20px); }
        .test2 { color: rgb(255, 128, 0); }
        .test3 { color: rgba(255, 128, 0, 0.5); }
        .test4 { background: linear-gradient(to right, red, blue); }
        .test5 { transform: translateX(50%) rotate(45deg); }
        .test6 { clip-path: polygon(50% 0%, 100% 100%, 0% 100%); }
        .test7 { width: min(50%, 300px); }
        .test8 { width: max(50%, 200px); }
        .test9 { width: clamp(200px, 50%, 500px); }
        .test10 { color: var(--primary-color, blue); }
    "#.to_string();
    
    c.bench_function("css_parse_functions", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

fn benchmark_printcss(c: &mut Criterion) {
    let css = r#"
        @page {
            size: A4 landscape;
            margin: 2cm 2.5cm;
            
            @top-center {
                content: "Document Title";
                font-size: 10pt;
            }
            
            @bottom-center {
                content: "Page " counter(page) " of " counter(pages);
            }
        }
        
        @page :first {
            margin: 0;
            @top-center { content: none; }
        }
        
        @page cover {
            margin: 0;
        }
        
        .chapter {
            page-break-before: always;
        }
        
        h1 {
            page-break-after: avoid;
        }
        
        table, figure {
            page-break-inside: avoid;
        }
        
        p {
            widows: 3;
            orphans: 3;
        }
        
        .cover {
            page: cover;
        }
    "#.to_string();
    
    c.bench_function("css_parse_printcss", |b| {
        b.iter(|| {
            let _ = parse_stylesheet(black_box(&css));
        });
    });
}

criterion_group!(
    benches,
    benchmark_small_stylesheet,
    benchmark_medium_stylesheet,
    benchmark_large_stylesheet,
    benchmark_complex_selectors,
    benchmark_at_rules,
    benchmark_scaling,
    benchmark_css_functions,
    benchmark_printcss
);
criterion_main!(benches);
