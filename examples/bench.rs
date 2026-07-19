use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use unravel_nl::{Locale, ParseCtx, parse};

const DEFAULT_INPUTS: &[&str] = &[
    "5尺3寸",
    "6帖",
    "延床100㎡",
    "1,234",
    "3640",
    "5ft 11",
    "1.5 cups",
    "100-120㎡",
    "約20kg",
    "約3坪",
    "4畳半",
    "1間半",
    "10 ± 0.5 mm",
    "a few minutes",
    "under 10 minutes",
    "10mm以下",
    "10平目",
    "2〜3日",
    "180cm",
    "1m80",
    "1,5 kg",
    "twenty-five kg",
    "三十五公斤",
    "1½ cups",
    "2 lb 3 oz",
    "3 yd 2 ft",
    "4 stone 6 lb",
    "72 in to cm",
    "an hour and a half",
    "3pm",
    "3pm-4pm",
    "午後3時",
    "every monday",
    "毎週月曜日",
    "USD 12.34",
    "$12",
    "¥1,234",
    "12 bucks",
    "99 pence",
    "50 cents",
    "20°C",
    "68 F",
    "293.15 K",
    "摂氏20度",
    "500 GB",
    "20 MB/s",
    "5 gpm",
    "500 mAh",
    "5 uM",
    "10 Nm",
    "500 lux",
    "20 mSv",
    "5 MBq",
    "10 inH₂O",
    "1 kgf/cm²",
    "20 MB/s to Mbit/s",
    "5 gpm to L/min",
    "1,5 litros",
    "2 mètres carrés",
    "10 quilômetros",
    "3公斤",
];

const HOSTILE_INPUTS: &[&str] = &[
    "meters meters meters",
    "1,,,,,,,,kg",
    "nextnextnextnextnext",
    "3pm EST",
    "every other monday",
    "毎月第2月曜日",
    "(((((((((((((((((((((((((((((((((",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "尺尺尺尺尺",
];

#[cfg(feature = "dates-jiff")]
const DATE_INPUTS: &[&str] = &[
    "next friday",
    "in 3 days",
    "明日",
    "3日後",
    "来週金曜日",
    "yesterday",
    "2 days ago",
    "this friday",
    "last friday",
    "05/06/2026",
    "demain",
    "vendredi prochain",
    "amanhã",
    "sexta-feira que vem",
    "明天",
    "下周五",
    "今日〜明日",
];

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(200_000);

    let default_ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    });
    run("default corpus", DEFAULT_INPUTS, default_ctx, iterations);
    run("hostile no-match corpus", HOSTILE_INPUTS, None, iterations);

    #[cfg(feature = "dates-jiff")]
    {
        let date_ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: unravel_nl::Date::new(2026, 7, 19),
            ..ParseCtx::default()
        });
        run("date corpus", DATE_INPUTS, date_ctx, iterations);
    }
}

fn run(label: &str, inputs: &[&str], ctx: Option<ParseCtx>, iterations: usize) {
    let started = Instant::now();
    let mut matched = 0_usize;

    for idx in 0..iterations {
        let input = inputs[idx % inputs.len()];
        let parsed = parse(black_box(input), black_box(ctx.clone()));
        if parsed.best.is_some() {
            matched += 1;
        }
        black_box(parsed);
    }

    let elapsed = started.elapsed();
    print_result(label, iterations, matched, elapsed);
}

fn print_result(label: &str, iterations: usize, matched: usize, elapsed: Duration) {
    let seconds = elapsed.as_secs_f64();
    let per_input_us = seconds * 1_000_000.0 / iterations as f64;
    let per_second = iterations as f64 / seconds;
    println!(
        "{label}: {iterations} parses, {matched} matched, {per_input_us:.3} us/input, {per_second:.0} parses/s"
    );
}
