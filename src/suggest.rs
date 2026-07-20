use crate::*;

pub(crate) fn suggestions_for(text: &str) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    for token in ascii_tokens(text) {
        if unit_by_alias(&token).is_some() {
            continue;
        }
        if let Some(suggestion) = suggest_unit(&token).or_else(|| suggest_legacy_word(&token)) {
            suggestions.push(suggestion);
        }
    }
    suggestions
}

pub(crate) fn suggest_unit(token: &str) -> Option<Suggestion> {
    let normalized = normalize_alias(token);
    if normalized.len() > 32 {
        return None;
    }
    let mut best: Option<(&'static str, usize)> = None;
    for unit in UNIT_DEFS {
        for alias in unit_lookup_aliases(unit) {
            let alias = alias.trim();
            if alias.is_empty() || !alias.is_ascii() || alias.contains(char::is_whitespace) {
                continue;
            }
            let limit = if normalized.len() <= 5 { 1 } else { 2 };
            if normalized.len().abs_diff(alias.len()) > limit {
                continue;
            }
            if !same_ascii_first_char(&normalized, alias) {
                continue;
            }
            let distance = levenshtein_ascii_case_insensitive(&normalized, alias);
            if distance > 0 && distance <= limit && best.is_none_or(|(_, best)| distance < best) {
                best = Some((unit.id, distance));
            }
        }
    }
    best.map(|(to, distance)| {
        let max_len = normalized.len().max(to.len()) as f64;
        Suggestion {
            from: token.to_owned(),
            to: to.to_owned(),
            score: Some(1.0 - distance as f64 / max_len),
        }
    })
    .or_else(|| suggest_non_ascii_unit(token))
}

pub(crate) fn suggest_non_ascii_unit(token: &str) -> Option<Suggestion> {
    if token.is_ascii() || token.chars().count() > 8 {
        return None;
    }
    let mut best: Option<(&'static str, usize, usize)> = None;
    for unit in UNIT_DEFS {
        for alias in unit_lookup_aliases(unit) {
            if alias.is_ascii() {
                continue;
            }
            let alias_len = alias.chars().count();
            if token.chars().count().abs_diff(alias_len) > 2 {
                continue;
            }
            let distance = levenshtein_chars(token, alias);
            let limit = if alias_len <= 2 { 1 } else { 2 };
            if distance > 0 && distance <= limit && best.is_none_or(|(_, best, _)| distance < best)
            {
                best = Some((unit.id, distance, alias_len));
            }
        }
    }
    best.map(|(to, distance, alias_len)| {
        let max_len = token.chars().count().max(alias_len) as f64;
        Suggestion {
            from: token.to_owned(),
            to: to.to_owned(),
            score: Some(1.0 - distance as f64 / max_len),
        }
    })
}

pub(crate) fn same_ascii_first_char(left: &str, right: &str) -> bool {
    match (left.as_bytes().first(), right.as_bytes().first()) {
        (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
        _ => false,
    }
}

pub(crate) fn levenshtein_ascii_case_insensitive(left: &str, right: &str) -> usize {
    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0; right.len() + 1];

    for (i, left_byte) in left.bytes().enumerate() {
        curr[0] = i + 1;
        for (j, right_byte) in right.bytes().enumerate() {
            let substitution = prev[j] + usize::from(!left_byte.eq_ignore_ascii_case(&right_byte));
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}

pub(crate) fn levenshtein_chars(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut prev: Vec<usize> = (0..=right_chars.len()).collect();
    let mut curr = vec![0; right_chars.len() + 1];

    for (i, left_char) in left.chars().enumerate() {
        curr[0] = i + 1;
        for (j, right_char) in right_chars.iter().enumerate() {
            let substitution = prev[j] + usize::from(left_char != *right_char);
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[right_chars.len()]
}

pub(crate) fn suggest_legacy_word(token: &str) -> Option<Suggestion> {
    if token.len() > 32 {
        return None;
    }
    let dictionary = ["tsubo", "shaku", "sun", "tatami"];
    for candidate in dictionary {
        let distance = levenshtein(token, candidate);
        let limit = if token.len() <= 5 { 1 } else { 2 };
        if distance > 0 && distance <= limit {
            let max_len = token.len().max(candidate.len()) as f64;
            return Some(Suggestion {
                from: token.to_owned(),
                to: candidate.to_owned(),
                score: Some(1.0 - distance as f64 / max_len),
            });
        }
    }
    None
}

pub(crate) fn ascii_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(core::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub(crate) fn levenshtein(left: &str, right: &str) -> usize {
    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0; right.len() + 1];

    for (i, left_byte) in left.bytes().enumerate() {
        curr[0] = i + 1;
        for (j, right_byte) in right.bytes().enumerate() {
            let substitution = prev[j] + usize::from(left_byte != right_byte);
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_did_you_mean() {
        let parsed = parse("10 tsbo", None);
        assert!(parsed.best.is_none());
        assert_eq!(parsed.suggestions[0].from, "tsbo");
        assert_eq!(parsed.suggestions[0].to, "tsubo");
        assert_eq!(parsed.findings.skipped.len(), 1);
    }
}
