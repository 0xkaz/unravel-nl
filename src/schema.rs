pub(crate) const PARSE_INPUT_SCHEMA_JSON: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://0xkaz.github.io/unravel-nl/schema/parse-input.json",
  "title": "unravel-nl parse input",
  "type": "object",
  "additionalProperties": false,
  "required": ["text"],
  "properties": {
    "text": {
      "type": "string",
      "minLength": 1,
      "description": "Informal or ambiguous natural-language value to parse."
    },
    "locale": {
      "type": "string",
      "description": "Optional BCP-47 style locale hint such as ja, en, en-US, or en-GB."
    },
    "expect": {
      "type": "string",
      "enum": ["quantity", "date", "range", "number"],
      "description": "Optional expected top-level reading kind. This does not constrain parsing; it only filters completion candidates and adds a millimeter length alternative for a bare number when set to quantity. Use purpose to restrict what is parsed."
    },
    "expected_dimensions": {
      "type": "string",
      "pattern": "^(length|area|mass|time|volume|currency|temperature|speed|data|data_rate|flow_rate|concentration|acceleration|force|torque|pressure|power|charge|voltage|current|resistance|illuminance|radiation_equivalent_dose|radioactivity)( *, *(length|area|mass|time|volume|currency|temperature|speed|data|data_rate|flow_rate|concentration|acceleration|force|torque|pressure|power|charge|voltage|current|resistance|illuminance|radiation_equivalent_dose|radioactivity))*$",
      "description": "Optional measurement domains the field accepts, as one name or a comma-separated list such as \"length,area\". This is a hard filter, not a hint: a reading from any other measurement domain is refused with REJECTED_BY_POLICY rather than returned. Readings that carry no dimension at all — a bare number, a date — are not refused. Omit it to accept every dimension; a name that is not on this list is refused rather than ignored."
    },
    "registry_dimensions": {
      "type": "string",
      "pattern": "^(|(length|area|mass|time|volume|currency|temperature|speed|data|data_rate|flow_rate|concentration|acceleration|force|torque|pressure|power|charge|voltage|current|resistance|illuminance|radiation_equivalent_dose|radioactivity)( *, *(length|area|mass|time|volume|currency|temperature|speed|data|data_rate|flow_rate|concentration|acceleration|force|torque|pressure|power|charge|voltage|current|resistance|illuminance|radiation_equivalent_dose|radioactivity))*)$",
      "description": "Optional built-in measurement domains present in the parser vocabulary. This is applied before grammar dispatch, registry lookup, typo correction, and completion. An empty string explicitly loads no measurement units; omission leaves the registry choice to the adapter. This is independent of expected_dimensions, which is an output acceptance policy."
    },
    "number_format": {
      "type": "string",
      "enum": ["auto", "comma_decimal", "dot_decimal"],
      "default": "auto",
      "description": "Explicit numeric punctuation policy. Use comma_decimal for 1,5 and dot_decimal for 1,234 grouping."
    },
    "purpose": {
      "type": "string",
      "enum": ["general", "quantity", "number", "date", "dimension_editor"],
      "default": "general",
      "description": "Selects the parser grammar. This is a hard filter, not a hint: input the selected grammar does not read is refused with NO_VALUE. quantity, number, and date parse exactly as the matching narrow entry point would. dimension_editor is for UI fields that only accept building dimensions; it runs the editor grammar over the whole input and is not equivalent to the parse_dimensions_for_editor extractor, which additionally scans free text for candidates and infers the expected dimension from a neighbouring label."
    },
    "accept": {
      "type": "object",
      "additionalProperties": false,
      "description": "Optional acceptance controls for broad parser shapes.",
      "properties": {
        "ranges": { "type": "boolean", "default": true },
        "conversions": { "type": "boolean", "default": true },
        "compounds": { "type": "boolean", "default": true },
        "fuzzy": { "type": "boolean", "default": true }
      }
    },
    "reference_date": {
      "type": "string",
      "format": "date",
      "description": "Civil reference date for relative dates. The parser never reads the host system clock or timezone."
    },
    "timezone": {
      "type": "string",
      "description": "Reserved for adapter layers and currently ignored by the parser. Setting it does not change any parse result and never populates a reading timezone, which is read only from the input text. The core parser also never infers a timezone from the host environment."
    },
    "strictness": {
      "type": "string",
      "enum": ["forgiving", "confirm", "strict"],
      "default": "forgiving"
    }
  }
}"#;

pub(crate) const PARSED_OUTPUT_SCHEMA_JSON: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://0xkaz.github.io/unravel-nl/schema/parsed-output.json",
  "title": "unravel-nl parsed output",
  "type": "object",
  "additionalProperties": false,
  "required": ["input", "best", "alternatives", "suggestions", "findings"],
  "properties": {
    "input": { "type": "string" },
    "locale": { "type": ["string", "null"] },
    "best": {
      "anyOf": [
        { "$ref": "#/$defs/reading" },
        { "type": "null" }
      ]
    },
    "alternatives": {
      "type": "array",
      "items": { "$ref": "#/$defs/reading" }
    },
    "suggestions": {
      "type": "array",
      "items": { "$ref": "#/$defs/suggestion" }
    },
    "findings": { "$ref": "#/$defs/findings" }
  },
  "$defs": {
    "kind": { "type": "string", "enum": ["quantity", "date", "range", "number"] },
    "dimension": {
      "type": "string",
      "enum": ["length", "area", "mass", "time", "volume", "currency", "temperature", "speed", "data", "data_rate", "flow_rate", "concentration", "acceleration", "force", "torque", "pressure", "power", "charge", "voltage", "current", "resistance", "illuminance", "radiation_equivalent_dose", "radioactivity"]
    },
    "provenance": {
      "type": "string",
      "enum": ["international_exact", "japanese_statute", "trade_custom", "si_multiple"]
    },
    "reading": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "$ref": "#/$defs/kind" },
        "customKind": { "type": ["string", "null"] },
        "value": { "type": ["number", "null"] },
        "unit": { "type": ["string", "null"] },
        "dimension": {
          "anyOf": [
            { "$ref": "#/$defs/dimension" },
            { "type": "null" }
          ]
        },
        "date": { "type": ["string", "null"], "format": "date" },
        "timezone": { "type": ["string", "null"], "description": "Canonical timezone for timezone-normalized readings." },
        "range": {
          "anyOf": [
            { "$ref": "#/$defs/range" },
            { "type": "null" }
          ]
        },
        "provenance": {
          "anyOf": [
            { "$ref": "#/$defs/provenance" },
            { "type": "null" }
          ]
        },
        "approximate": { "type": ["boolean", "null"] },
        "confidence": { "type": ["number", "null"], "minimum": 0, "maximum": 1 }
      }
    },
    "range": {
      "type": "object",
      "additionalProperties": false,
      "required": ["from", "to"],
      "properties": {
        "from": { "$ref": "#/$defs/reading" },
        "to": { "$ref": "#/$defs/reading" }
      }
    },
    "suggestion": {
      "type": "object",
      "additionalProperties": false,
      "required": ["from", "to"],
      "properties": {
        "from": { "type": "string" },
        "to": { "type": "string" },
        "score": { "type": ["number", "null"], "minimum": 0, "maximum": 1 }
      }
    },
    "span": {
      "type": "object",
      "additionalProperties": false,
      "required": ["start", "end", "text"],
      "properties": {
        "start": { "type": "integer", "minimum": 0 },
        "end": { "type": "integer", "minimum": 0 },
        "text": { "type": "string" }
      }
    },
    "issue": {
      "type": "object",
      "additionalProperties": true,
      "required": ["code", "ref_text", "reason", "span"],
      "properties": {
        "code": { "type": "string" },
        "ref_text": { "type": "string" },
        "reason": { "type": "string" },
        "span": { "$ref": "#/$defs/span" }
      }
    },
    "findings": {
      "type": "object",
      "additionalProperties": false,
      "required": ["skipped", "ambiguities", "approximations"],
      "properties": {
        "skipped": { "type": "array", "items": { "$ref": "#/$defs/issue" } },
        "ambiguities": { "type": "array", "items": { "$ref": "#/$defs/issue" } },
        "approximations": { "type": "array", "items": { "$ref": "#/$defs/issue" } }
      }
    }
  }
}"##;

pub(crate) const MCP_TOOL_SCHEMA_JSON: &str = r#"{
  "name": "unravel_nl_parse",
  "description": "Parse informal or ambiguous natural-language quantities, dates, ranges, and values into deterministic canonical readings.",
  "inputSchema": {
    "$ref": "https://0xkaz.github.io/unravel-nl/schema/parse-input.json"
  },
  "outputSchema": {
    "$ref": "https://0xkaz.github.io/unravel-nl/schema/parsed-output.json"
  }
}"#;
