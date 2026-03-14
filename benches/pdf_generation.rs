//! PDF Generation Benchmarks
//!
//! These benchmarks measure the performance of PDF generation.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use html2pdf::pdf::{PdfWriter, PageContent};
use html2pdf::types::{Rect, Point, Color, PaperSize, Orientation};

fn benchmark_single_page(c: &mut Criterion) {
    c.bench_function("pdf_single_page_simple", |b| {
        b.iter(|| {
            let mut writer = PdfWriter::new();
            writer.init_document();
            writer.set_info("Benchmark", "Test", "HTML2PDF");
            
            let mut content = PageContent::new();
            content.begin_text();
            content.set_font("F1", 12.0);
            content.text_position(100.0, 700.0);
            content.show_text("Hello, PDF!");
            content.end_text();
            
            writer.add_page(content);
            
            let mut output = Vec::new();
            let _ = writer.write(black_box(&mut output));
        });
    });
}

fn benchmark_multiple_pages(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_multiple_pages");
    
    for pages in [1, 5, 10, 25, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(pages), pages, |b, pages| {
            b.iter(|| {
                let mut writer = PdfWriter::new();
                writer.init_document();
                writer.set_info("Benchmark", "Test", "HTML2PDF");
                
                for i in 0..*pages {
                    let mut content = PageContent::new();
                    content.begin_text();
                    content.set_font("F1", 12.0);
                    content.text_position(100.0, 700.0);
                    content.show_text(&format!("Page {}", i + 1));
                    content.end_text();
                    
                    writer.add_page(content);
                }
                
                let mut output = Vec::new();
                let _ = writer.write(black_box(&mut output));
            });
        });
    }
    
    group.finish();
}

fn benchmark_drawing_operations(c: &mut Criterion) {
    c.bench_function("pdf_drawing_operations", |b| {
        b.iter(|| {
            let mut writer = PdfWriter::new();
            writer.init_document();
            
            let mut content = PageContent::new();
            
            // Draw multiple rectangles
            for i in 0..100 {
                let x = 50.0 + (i % 10) as f32 * 50.0;
                let y = 500.0 - (i / 10) as f32 * 40.0;
                
                content.set_fill_color(Color::new(
                    (i * 2 % 256) as u8,
                    (i * 3 % 256) as u8,
                    (i * 5 % 256) as u8,
                ));
                content.draw_rect(Rect::new(x, y, 40.0, 30.0));
                content.fill();
            }
            
            // Draw lines
            content.set_stroke_color(Color::new(0, 0, 0));
            content.set_line_width(1.0);
            for i in 0..50 {
                content.draw_line(
                    Point::new(50.0, 100.0 + i as f32 * 5.0),
                    Point::new(550.0, 100.0 + i as f32 * 5.0),
                );
            }
            content.stroke();
            
            writer.add_page(content);
            
            let mut output = Vec::new();
            let _ = writer.write(black_box(&mut output));
        });
    });
}

fn benchmark_text_rendering(c: &mut Criterion) {
    c.bench_function("pdf_text_rendering", |b| {
        b.iter(|| {
            let mut writer = PdfWriter::new();
            writer.init_document();
            
            let mut content = PageContent::new();
            content.begin_text();
            content.set_font("F1", 10.0);
            
            for i in 0..50 {
                content.text_position(50.0, 750.0 - i as f32 * 14.0);
                content.show_text(&format!("Line {}: This is a test of text rendering in PDF documents.", i + 1));
            }
            
            content.end_text();
            writer.add_page(content);
            
            let mut output = Vec::new();
            let _ = writer.write(black_box(&mut output));
        });
    });
}

fn benchmark_paper_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_paper_sizes");
    
    let sizes = vec![
        ("A4", PaperSize::A4),
        ("Letter", PaperSize::Letter),
        ("A3", PaperSize::A3),
    ];
    
    for (name, size) in sizes {
        group.bench_function(format!("paper_{}", name), |b| {
            b.iter(|| {
                let mut writer = PdfWriter::new();
                writer.init_document();
                writer.set_paper_size(size, Orientation::Portrait);
                
                let content = PageContent::new();
                writer.add_page(content);
                
                let mut output = Vec::new();
                let _ = writer.write(black_box(&mut output));
            });
        });
    }
    
    group.finish();
}

fn benchmark_font_embedding(c: &mut Criterion) {
    c.bench_function("pdf_font_embedding", |b| {
        b.iter(|| {
            let mut writer = PdfWriter::new();
            writer.init_document();
            
            // Add multiple fonts
            for i in 0..5 {
                writer.add_standard_font(&format!("F{}", i + 1), "Helvetica");
            }
            
            let mut content = PageContent::new();
            content.begin_text();
            
            for i in 0..5 {
                content.set_font(&format!("F{}", i + 1), 12.0);
                content.text_position(100.0, 700.0 - i as f32 * 50.0);
                content.show_text(&format!("Text with font {}", i + 1));
            }
            
            content.end_text();
            writer.add_page(content);
            
            let mut output = Vec::new();
            let _ = writer.write(black_box(&mut output));
        });
    });
}

criterion_group!(
    benches,
    benchmark_single_page,
    benchmark_multiple_pages,
    benchmark_drawing_operations,
    benchmark_text_rendering,
    benchmark_paper_sizes,
    benchmark_font_embedding
);
criterion_main!(benches);
