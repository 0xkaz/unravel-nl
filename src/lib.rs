//! Deterministic parsing for informal natural-language quantities and values.
//!
//! `unravel-nl` turns informal or ambiguous text such as `5尺3寸`, `about 20kg`,
//! `1.234,56 kg`, or `6帖` into canonical readings, and renders canonical
//! readings back into human-readable strings.
//!
//! # Guarantees
//!
//! - **Deterministic.** The same input and context always produce the same
//!   result. No randomness, no models, no host clock, no locale environment.
//! - **No silent loss.** Anything skipped, ambiguous, or approximate is
//!   reported in [`Findings`] instead of being quietly dropped.
//! - **No forced choice.** When a fragment has several plausible readings, the
//!   competing readings are returned in [`Parsed::alternatives`] rather than
//!   the parser committing to one.
//! - **No I/O and no runtime dependencies** on the default compute path.
//!
//! # Getting started
//!
//! Use [`parse`] when the kind of value is unknown, or a narrower entry point
//! such as [`parse_quantity_fast`], [`parse_date_fast`], or
//! [`parse_dimensions_for_editor`] when the field type is already known — see
//! the entry point table in those functions' documentation.
//!
//! ```
//! use unravel_nl::{humanize, parse, HumanizeCtx, Locale, ParseCtx};
//!
//! let parsed = parse(
//!     "5尺3寸",
//!     Some(ParseCtx {
//!         locale: Some(Locale::Ja),
//!         ..ParseCtx::default()
//!     }),
//! );
//!
//! let best = parsed.best.expect("a canonical reading");
//! assert_eq!(best.unit.as_deref(), Some("m"));
//! assert_eq!(
//!     humanize(&best, Some(HumanizeCtx { locale: Some(Locale::Ja) })),
//!     "5尺3寸 (approx.)"
//! );
//! ```
//!
//! This crate is an independent Rust implementation inspired by the public API
//! shape of `pascalorg/lingo` (MIT). It does not copy source code from that
//! project.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod adapters;
mod completion;
mod currency;
mod dates;
mod duration;
mod entry;
mod findings;
mod fuzzy;
mod json_out;
mod normalize;
mod number;
mod quantity;
mod recurrence;
mod scan;
mod schema;
mod suggest;
#[cfg(test)]
mod test_util;
mod types;
mod unit_aliases;
mod unit_defs;
mod units;
mod wasm_json;

#[cfg(any(feature = "wasm", test))]
pub(crate) use adapters::format_number;
pub use adapters::{
    CanonicalizeRequest, CanonicalizedValue, canonicalize_values, describe_parsed,
    describe_reading, humanize, repair_tool_call_message,
};
pub use completion::{complete, complete_readings};
pub(crate) use currency::*;
pub(crate) use dates::*;
pub(crate) use duration::*;
pub(crate) use entry::*;
pub use entry::{
    parse, parse_all, parse_date_fast, parse_dimensions_for_editor, parse_number_fast,
    parse_quantity_fast, parse_recurrence_fast,
};
pub(crate) use findings::*;
pub use findings::{
    Ambiguity, Approximation, Findings, IssueCode, IssueSeverity, RankedIssue, Skipped, Span,
    ranked_findings,
};
pub(crate) use fuzzy::*;
pub(crate) use json_out::*;
pub(crate) use normalize::*;
pub(crate) use number::*;
pub(crate) use quantity::*;
pub(crate) use recurrence::*;
pub(crate) use scan::*;
pub(crate) use schema::*;
pub(crate) use suggest::*;
pub(crate) use types::*;
pub use types::{
    AcceptOptions, Completion, CompletionKind, CompletionReading, CurrencyRate, CustomUnit, Date,
    Dimension, FuzzyProfile, FuzzyTerm, HumanizeCtx, Kind, Locale, NumberFormat, ParseCtx,
    ParsePurpose, Parsed, ParsedMatch, Provenance, RangeReading, Reading, ResourceField,
    ResourceView, Strictness, Suggestion,
};
pub(crate) use unit_aliases::*;
pub use unit_defs::UnitDef;
pub(crate) use unit_defs::*;
pub(crate) use units::*;
pub use units::{unit_definitions, units_of};
pub use wasm_json::{
    CONTRACT_VERSION, contract_version, mcp_tool_schema_json, parse_input_schema_json,
    parsed_output_schema_json,
};
#[cfg(feature = "wasm")]
pub use wasm_json::{
    parse_all_json, parse_all_json_with_context, parse_all_json_with_locale,
    parse_dimensions_for_editor_json, parse_dimensions_for_editor_json_with_context, parse_json,
    parse_json_with_context, parse_json_with_locale,
};
