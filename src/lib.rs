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
//! - **No panic.** The public API is written never to panic; input it cannot
//!   read comes back as a finding, not as an unwind.
//! - **No silent loss.** Anything skipped, ambiguous, or approximate is
//!   reported in [`Findings`] instead of being quietly dropped.
//! - **No forced choice.** When a fragment has several plausible readings, the
//!   competing readings are returned in [`Parsed::alternatives`] rather than
//!   the parser committing to one.
//! - **No I/O and no runtime dependencies** on the default compute path.
//!
//! # Choosing an entry point
//!
//! [`Parser`] is the configured entry point. Use [`Parser::japanese_building`]
//! for the small length-and-area preset, or [`Parser::new`] with an explicit
//! [`DimensionSet`]. Keeping the unit domains on one instance prevents
//! unrelated registries from competing for the same input.
//!
//! [`Parser::default`] is the locale-neutral length-and-area preset.
//! [`Parser::unrestricted`] keeps the old full catalog as an explicit
//! compatibility choice. Set [`ParseCtx::purpose`] through
//! [`Parser::with_context`] when a field is known to hold only a quantity,
//! date, or number.
//!
//! # Reading the result
//!
//! The reading the parser ranked first is in [`Parsed::best`], competing
//! readings are in [`Parsed::alternatives`], and anything the parser could not
//! resolve silently is in [`Parsed::findings`]. An empty [`Findings`] means the
//! whole input was consumed with no guesswork; a non-empty one tells you
//! exactly where the parser had to skip, choose, or approximate. Finding spans
//! address the string you passed in, so they can be used directly to highlight
//! it — see [`Span`].
//!
//! # Cargo features
//!
//! Everything above works with no features enabled. The optional ones are:
//!
//! | Feature | Adds |
//! | --- | --- |
//! | `dates-jiff` | Calendar arithmetic: relative dates such as `next friday` or `来週金曜日`, and three-part numeric dates. Without it these are reported as findings rather than resolved. |
//! | `timezones-jiff` | IANA time zone resolution, e.g. `3pm Europe/Paris` — but only together with an explicit [`ParseCtx::reference_date`], since a zone offset is undefined without a date. Without one the zone is still reported as [`IssueCode::TimezoneUnsupported`] and `best` is `None`, on this feature as on a default build. Implies `dates-jiff`. |
//! | `wasm` | `wasm-bindgen` exports for browser and Node adapters. |
//!
//! # Getting started
//!
//! ```
//! use unravel_nl::{humanize, HumanizeCtx, Locale, Parser};
//!
//! let parser = Parser::japanese_building();
//! let parsed = parser.parse("5尺3寸");
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

// Compile and run every Rust example in both READMEs as a doctest, so the
// published documentation cannot drift from the API. Gated on `dates-jiff`
// because several examples parse dates, which that feature provides.
#[cfg(all(doctest, feature = "dates-jiff"))]
#[doc = include_str!("../README.md")]
struct ReadmeExamples;

#[cfg(all(doctest, feature = "dates-jiff"))]
#[doc = include_str!("../README.ja.md")]
struct ReadmeJaExamples;

mod adapters;
mod completion;
mod currency;
mod dates;
mod duration;
mod entry;
mod findings;
mod fuzzy;
mod grammar;
mod json_out;
mod normalize;
mod number;
mod parser;
mod quantity;
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

pub use adapters::{
    CanonicalizeRequest, CanonicalizedValue, canonicalize_values, describe_parsed,
    describe_reading, humanize, repair_tool_call_message,
};
#[cfg(test)]
pub(crate) use completion::*;
pub(crate) use currency::*;
pub(crate) use dates::*;
pub(crate) use duration::*;
pub(crate) use entry::*;
pub(crate) use findings::*;
pub use findings::{
    Ambiguity, Approximation, Findings, IssueCode, IssueSeverity, RankedIssue, Skipped, Span,
    accepts, ranked_findings,
};
pub(crate) use fuzzy::*;
pub(crate) use grammar::*;
pub(crate) use json_out::*;
pub(crate) use normalize::*;
pub(crate) use number::*;
pub use parser::Parser;
pub(crate) use quantity::*;
pub(crate) use scan::*;
pub(crate) use schema::*;
pub(crate) use suggest::*;
pub(crate) use types::*;
pub use types::{
    AcceptOptions, Completion, CompletionKind, CompletionReading, CurrencyRate, CustomUnit, Date,
    Dimension, DimensionSet, FuzzyProfile, FuzzyTerm, HumanizeCtx, Kind, Locale, NumberFormat,
    ParseCtx, ParsePurpose, Parsed, ParsedMatch, Provenance, RangeReading, Reading, ResourceField,
    ResourceView, Strictness, Suggestion, UnitRegistry,
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
    parse_dimensions_for_editor_json, parse_dimensions_for_editor_json_with_context, parse_json,
    parse_json_with_context, parse_json_with_locale,
};
