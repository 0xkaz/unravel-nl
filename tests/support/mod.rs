#![allow(dead_code)]

use unravel_nl::{
    Completion, CompletionReading, ParseCtx, ParsePurpose, Parsed, ParsedMatch, Parser,
};

pub fn parse(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    Parser::unrestricted_with_context(ctx.unwrap_or_default()).parse(text)
}

pub fn parse_quantity_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    parse_for_purpose(text, ctx, ParsePurpose::Quantity)
}

pub fn parse_number_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    parse_for_purpose(text, ctx, ParsePurpose::Number)
}

pub fn parse_date_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    parse_for_purpose(text, ctx, ParsePurpose::Date)
}

pub fn parse_dimensions_for_editor(text: &str, ctx: Option<ParseCtx>) -> Vec<ParsedMatch> {
    Parser::unrestricted_with_context(ctx.unwrap_or_default()).parse_dimensions_for_editor(text)
}

pub fn complete(text: &str, ctx: Option<ParseCtx>) -> Vec<Completion> {
    Parser::unrestricted_with_context(ctx.unwrap_or_default()).complete(text)
}

pub fn complete_readings(text: &str, ctx: Option<ParseCtx>) -> Vec<CompletionReading> {
    Parser::unrestricted_with_context(ctx.unwrap_or_default()).complete_readings(text)
}

fn parse_for_purpose(text: &str, ctx: Option<ParseCtx>, purpose: ParsePurpose) -> Parsed {
    let mut ctx = ctx.unwrap_or_default();
    ctx.purpose = purpose;
    Parser::unrestricted_with_context(ctx).parse(text)
}
