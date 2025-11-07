use aurora_lexer::Lexer;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

/// Small source (~1KB) - typical function
const SMALL_SOURCE: &str = r#"
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(20);
    println("Fibonacci(20) = {}", result);
}
"#;

/// Medium source (~10KB) - typical module
fn generate_medium_source() -> String {
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!(
            r#"
fn function_{i}(x: i64, y: i64) -> i64 {{
    let sum = x + y;
    let product = x * y;
    let result = sum * product;
    if result > 1000 {{
        return result / 2;
    }} else {{
        return result * 2;
    }}
}}
"#
        ));
    }
    source
}

/// Large source (~100KB) - full application
fn generate_large_source() -> String {
    let medium = generate_medium_source();
    medium.repeat(10)
}

fn bench_small_source(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_small");
    group.throughput(Throughput::Bytes(SMALL_SOURCE.len() as u64));

    group.bench_function("tokenize_1kb", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(SMALL_SOURCE), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

fn bench_medium_source(c: &mut Criterion) {
    let source = generate_medium_source();
    let source_len = source.len();

    let mut group = c.benchmark_group("lexer_medium");
    group.throughput(Throughput::Bytes(source_len as u64));

    group.bench_function("tokenize_10kb", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&source), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

fn bench_large_source(c: &mut Criterion) {
    let source = generate_large_source();
    let source_len = source.len();

    let mut group = c.benchmark_group("lexer_large");
    group.throughput(Throughput::Bytes(source_len as u64));
    group.sample_size(20); // Reduce samples for large benchmark

    group.bench_function("tokenize_100kb", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&source), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });

    group.finish();
}

fn bench_keywords(c: &mut Criterion) {
    let source = "fn let mut const if else match for while loop return";

    c.bench_function("keyword_lookup", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });
}

fn bench_operators(c: &mut Criterion) {
    let source = "+ - * / % == != < <= > >= && || ! & | ^ << >> += -= *= /= **";

    c.bench_function("operator_parsing", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });
}

fn bench_numbers(c: &mut Criterion) {
    let source = "0 1 42 1234567890 0x1F 0o77 0b1010 3.14 2.71828 1.0e10";

    c.bench_function("number_literals", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });
}

fn bench_strings(c: &mut Criterion) {
    let source = r#""hello" "world" "a very long string with many characters" "unicode: 你好世界""#;

    c.bench_function("string_literals", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source), "bench.ax".to_string()).unwrap();
            let mut count = 0;
            loop {
                let tok = lexer.next_token().unwrap();
                count += 1;
                if tok.kind == aurora_lexer::TokenKind::Eof {
                    break;
                }
            }
            black_box(count)
        });
    });
}

criterion_group!(
    benches,
    bench_small_source,
    bench_medium_source,
    bench_large_source,
    bench_keywords,
    bench_operators,
    bench_numbers,
    bench_strings
);
criterion_main!(benches);
