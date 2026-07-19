use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use unravel_nl::{
    Date, Locale, ParseCtx, parse, parse_all, parse_date_fast, parse_dimensions_for_editor,
    parse_quantity_fast,
};

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

const SENTENCES: &[&str] = &[
    "延床100㎡、敷地面積120㎡、高さ3.5m、予算¥1,234",
    "ship 2 boxes at 5 kg each by friday",
    "幅１．５ｍ；重量五キログラム；面積百二十平米",
    "convert 72 in to cm and keep pressure under 10 inH₂O",
    "dose 20 mSv, activity 5 MBq, flow 5 gpm",
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

    let quantity_ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    });
    let date_ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        ..ParseCtx::default()
    });

    run_parse(
        "parse() quantity corpus",
        QUANTITY_INPUTS,
        quantity_ctx.clone(),
        iterations,
        parse,
    );
    run_parse(
        "parse_quantity_fast() corpus",
        QUANTITY_INPUTS,
        quantity_ctx,
        iterations,
        parse_quantity_fast,
    );
    run_parse(
        "parse() date corpus",
        DATE_INPUTS,
        date_ctx.clone(),
        iterations,
        parse,
    );
    run_parse(
        "parse_date_fast() corpus",
        DATE_INPUTS,
        date_ctx,
        iterations,
        parse_date_fast,
    );
    run_scan("parse_all() sentence corpus", SENTENCES, iterations);
    run_editor_scan(
        "parse_dimensions_for_editor() corpus",
        EDITOR_SENTENCES,
        iterations,
    );
}

fn run_parse(
    label: &str,
    inputs: &[&str],
    ctx: Option<ParseCtx>,
    iterations: usize,
    parser: fn(&str, Option<ParseCtx>) -> unravel_nl::Parsed,
) {
    let started = Instant::now();
    let mut matched = 0_usize;

    for idx in 0..iterations {
        let input = inputs[idx % inputs.len()];
        let parsed = parser(black_box(input), black_box(ctx.clone()));
        if parsed.best.is_some() {
            matched += 1;
        }
        black_box(parsed);
    }

    print_result(label, iterations, matched, started.elapsed());
}

fn run_scan(label: &str, inputs: &[&str], iterations: usize) {
    let started = Instant::now();
    let mut matched = 0_usize;
    let ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        ..ParseCtx::default()
    });

    for idx in 0..iterations {
        let input = inputs[idx % inputs.len()];
        let matches = parse_all(black_box(input), black_box(ctx.clone()));
        matched += matches.len();
        black_box(matches);
    }

    print_result(label, iterations, matched, started.elapsed());
}

fn run_editor_scan(label: &str, inputs: &[&str], iterations: usize) {
    let started = Instant::now();
    let mut matched = 0_usize;
    let ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    });

    for idx in 0..iterations {
        let input = inputs[idx % inputs.len()];
        let matches = parse_dimensions_for_editor(black_box(input), black_box(ctx.clone()));
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
