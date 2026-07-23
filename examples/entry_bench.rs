use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use unravel_nl::{Date, Locale, ParseCtx, ParsePurpose, Parser};

const QUANTITY_INPUTS: &[&str] = &[
    "5尺3寸",
    "５尺３寸",
    "延床100㎡",
    "1.234,56 kg",
    "1 234,56 m",
    "3.5万円",
    "180cm",
    "1m80",
    "twenty-five kg",
    "三十五公斤",
    "20 MB/s",
    "5 gpm",
    "10 inH₂O",
];

const DATE_INPUTS: &[&str] = &[
    "next friday",
    "in 3 days",
    "明日",
    "来週金曜日",
    "demain",
    "vendredi prochain",
    "amanhã",
    "sexta-feira que vem",
    "明天",
    "下周五",
];

const EDITOR_SENTENCES: &[&str] = &[
    "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
    "壁厚105mm、高さ2.9m、備考1234",
    "部材3640、north 800、room w 900",
    "延床100㎡、敷地面積120㎡、予算¥1,234",
    "4畳半 / 6帖 / room h 2400",
];

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(200_000);

    let quantity_general = Parser::unrestricted_with_context(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    });
    let quantity_only = Parser::unrestricted_with_context(ParseCtx {
        locale: Some(Locale::Ja),
        purpose: ParsePurpose::Quantity,
        ..ParseCtx::default()
    });
    let date_general = Parser::unrestricted_with_context(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        ..ParseCtx::default()
    });
    let date_only = Parser::unrestricted_with_context(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        purpose: ParsePurpose::Date,
        ..ParseCtx::default()
    });

    run_parse(
        "Parser general quantity corpus",
        QUANTITY_INPUTS,
        &quantity_general,
        iterations,
    );
    run_parse(
        "Parser quantity-purpose corpus",
        QUANTITY_INPUTS,
        &quantity_only,
        iterations,
    );
    run_parse(
        "Parser general date corpus",
        DATE_INPUTS,
        &date_general,
        iterations,
    );
    run_parse(
        "Parser date-purpose corpus",
        DATE_INPUTS,
        &date_only,
        iterations,
    );
    run_editor_scan(
        "Parser editor corpus",
        EDITOR_SENTENCES,
        &Parser::japanese_building(),
        iterations,
    );
}

fn run_parse(label: &str, inputs: &[&str], parser: &Parser, iterations: usize) {
    let started = Instant::now();
    let mut matched = 0_usize;

    for idx in 0..iterations {
        let input = inputs[idx % inputs.len()];
        let parsed = parser.parse(black_box(input));
        if parsed.best.is_some() {
            matched += 1;
        }
        black_box(parsed);
    }

    print_result(label, iterations, matched, started.elapsed());
}

fn run_editor_scan(label: &str, inputs: &[&str], parser: &Parser, iterations: usize) {
    let started = Instant::now();
    let mut matched = 0_usize;
    for idx in 0..iterations {
        let input = inputs[idx % inputs.len()];
        let matches = parser.parse_dimensions_for_editor(black_box(input));
        matched += matches.len();
        black_box(matches);
    }

    print_result(label, iterations, matched, started.elapsed());
}

fn print_result(label: &str, iterations: usize, matched: usize, elapsed: Duration) {
    let seconds = elapsed.as_secs_f64();
    let per_input_us = seconds * 1_000_000.0 / iterations as f64;
    let per_second = iterations as f64 / seconds;
    println!(
        "{label}: {iterations} parses, {matched} matched, {per_input_us:.3} us/input, {per_second:.0} parses/s"
    );
}
