export type LocaleTag = "ja" | "en" | "en-US" | "en-GB" | string;

export type ReadingKind = "quantity" | "date" | "range" | "number" | "recurrence";

export type Dimension =
  | "length"
  | "area"
  | "mass"
  | "time"
  | "volume"
  | "currency"
  | "temperature"
  | "speed"
  | "data"
  | "data_rate"
  | "flow_rate"
  | "concentration"
  | "acceleration"
  | "force"
  | "torque"
  | "pressure"
  | "power"
  | "charge"
  | "voltage"
  | "current"
  | "resistance"
  | "illuminance"
  | "radiation_equivalent_dose"
  | "radioactivity";

export type IssueSeverity = "error" | "warning" | "info";

export interface ParseContext {
  locale?: LocaleTag | null;
  expect?: ReadingKind | null;
  /**
   * Measurement domains the field accepts. A hard filter: a reading from any
   * other domain is refused with REJECTED_BY_POLICY rather than returned.
   * Readings with no dimension at all are not refused. Several domains are
   * written as a comma-separated list, e.g. "length,area".
   */
  expected_dimension?: Dimension | `${Dimension},${string}` | null;
  number_format?: "auto" | "comma_decimal" | "dot_decimal";
  purpose?: "general" | "quantity" | "number" | "date" | "recurrence" | "dimension_editor";
  strictness?: "forgiving" | "confirm" | "strict";
}

export interface Reading {
  kind: ReadingKind;
  customKind?: string | null;
  value?: number | null;
  unit?: string | null;
  dimension?: Dimension | null;
  date?: string | null;
  recurrence?: string | null;
  timezone?: string | null;
  range?: { from: Reading; to: Reading } | null;
  provenance?: string | null;
  approximate?: boolean | null;
  confidence?: number | null;
}

export interface Issue {
  code: string;
  severity: IssueSeverity;
  rank: number;
  recoverable: boolean;
  ref_text?: string;
  reason?: string;
  span?: { start: number; end: number; text: string };
}

export interface Parsed {
  input?: string;
  locale?: LocaleTag | null;
  best?: Reading | null;
  alternatives?: Reading[];
  suggestions?: Array<{ from: string; to: string; score?: number | null }>;
  findings?: Record<string, unknown>;
  issues?: Issue[];
}

export interface ParseState {
  ok: boolean;
  parsed: Parsed | null;
  best: Reading | null;
  issues: Issue[];
  message: string | null;
}

export interface ParsedMatch {
  start: number;
  end: number;
  byteStart?: number;
  byteEnd?: number;
  charStart?: number;
  charEnd?: number;
  codeUnitStart?: number;
  codeUnitEnd?: number;
  text: string;
  parsed: Parsed;
}

export interface ParsedMatchState extends ParsedMatch {
  ok: boolean;
  best: Reading | null;
  issues: Issue[];
  message: string | null;
}

export type Parser = (text: string, ctx?: ParseContext) => string | Parsed;
export type Scanner = (text: string, ctx?: ParseContext) => string | ParsedMatch[];

export function parseForUi(parser: Parser, text: string, ctx?: ParseContext): ParseState;

export function parseAllForUi(
  parser: Scanner,
  text: string,
  ctx?: ParseContext,
): ParsedMatchState[];

export function canonicalizeFieldsForUi(
  parser: Parser,
  requests: Array<{ field: string; text?: string; input?: string; ctx?: ParseContext }>,
): Array<{
  field: string;
  input: string;
  ok: boolean;
  canonical: Reading | null;
  parsed: Parsed | null;
  issues: Issue[];
  message: string | null;
}>;

export function canonicalizeValuesForUi(
  canonicalizer: (requests: unknown) => string | unknown[],
  requests: unknown,
): unknown[];

export function applyParseState<T extends HTMLElement>(element: T, state: ParseState): ParseState;

export function parseInputElement<T extends HTMLInputElement | HTMLTextAreaElement>(
  element: T,
  parser: Parser,
  ctx?: ParseContext,
): ParseState;

export function createUnravelFieldController<T extends HTMLInputElement | HTMLTextAreaElement>(
  element: T,
  parser: Parser,
  options?: {
    ctx?: ParseContext;
    delayMs?: number;
    onChange?: (state: ParseState) => void;
  },
): { readonly state: ParseState; parse(): void; disconnect(): void };

export function createUnravelReactAdapter(
  React: unknown,
  parser: Parser,
): {
  useUnravelValue(initialValue?: string, options?: { ctx?: ParseContext }): unknown;
  UnravelInput(props: Record<string, unknown>): unknown;
};

export function defineUnravelElement(
  parser: Parser,
  options?: {
    ctx?: ParseContext;
    customElements?: CustomElementRegistry;
    HTMLElement?: typeof HTMLElement;
    tagName?: string;
  },
): CustomElementConstructor | null;

export function rankIssues(parsed: Parsed | null | undefined): Issue[];
