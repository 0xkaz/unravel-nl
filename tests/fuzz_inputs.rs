use unravel_nl::{Locale, ParseCtx, complete, parse, parse_all, ranked_findings};

#[test]
fn hostile_unicode_inputs_do_not_panic() {
    let mut seed = 0x5eed_2026_u64;
    let alphabet = [
        '0', '1', '2', '3', '4', '5', '９', '．', ',', '，', ' ', '\u{202f}', '\u{200b}', 'm', 'k',
        'g', '㎡', '尺', '寸', '平', '米', '万', '億', '円', '/', '-', '±', '毎', '週', '月', '曜',
        '日', '€', '¥', 'a', 'e', 'r', 'y',
    ];

    for len in 0..96 {
        let mut input = String::new();
        for _ in 0..len {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push(alphabet[(seed as usize) % alphabet.len()]);
        }

        let ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: unravel_nl::Date::new(2026, 7, 19),
            ..ParseCtx::default()
        });
        let parsed = parse(&input, ctx.clone());
        let _issues = ranked_findings(&parsed);
        let _matches = parse_all(&input, ctx);
        let _completions = complete(&input, None);
    }
}
