import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import {
  applyParseState,
  canonicalizeFieldsForUi,
  canonicalizeValuesForUi,
  createUnravelFieldController,
  parseAllForUi,
  parseForUi,
  rankIssues,
  acceptsParsed,
} from "../web/unravel-adapters.js";

const cleanParsed = {
  best: { kind: "quantity", value: 1.8, unit: "m" },
  findings: { skipped: [], ambiguities: [], approximations: [] },
};

const timezoneParsed = {
  best: null,
  findings: {
    skipped: [
      {
        code: "TIMEZONE_UNSUPPORTED",
        ref_text: "Europe/Paris",
        reason: "timezone policy required",
        span: { start: 4, end: 16, text: "Europe/Paris" },
      },
    ],
    ambiguities: [],
    approximations: [],
  },
};

assert.equal(parseForUi(() => cleanParsed, "180cm").ok, true);
assert.equal(parseForUi(() => JSON.stringify(cleanParsed), "180cm").ok, true);

const state = parseForUi(() => timezoneParsed, "3pm Europe/Paris");
assert.equal(state.ok, false);
assert.equal(state.issues[0].severity, "error");
assert.equal(state.issues[0].rank, 90);
assert.equal(state.message, "[TIMEZONE_UNSUPPORTED] timezone policy required");

const element = mockElement("3pm Europe/Paris");
applyParseState(element, state);
assert.equal(element.dataset.unravelOk, "false");
assert.equal(element.dataset.unravelIssueCode, "TIMEZONE_UNSUPPORTED");
assert.equal(element.attributes["aria-invalid"], "true");

let calls = 0;
const controller = createUnravelFieldController(
  element,
  () => {
    calls += 1;
    return cleanParsed;
  },
  { onChange: () => {} },
);
element.value = "180cm";
element.dispatch("input");
assert.equal(calls, 2);
assert.equal(controller.state.ok, true);
controller.disconnect();
element.dispatch("input");
assert.equal(calls, 2);

const issues = rankIssues({
  findings: {
    skipped: [{ code: "APPROXIMATION", ref_text: "about", reason: "approx", span: {} }],
    ambiguities: [{ code: "AMBIGUOUS_UNIT", ref_text: "cup", reason: "unit", span: {} }],
    approximations: [],
  },
});
assert.deepEqual(
  issues.map((issue) => issue.code),
  ["AMBIGUOUS_UNIT", "APPROXIMATION"],
);

// The adapter classifies codes itself rather than trusting the envelope, so
// every code the core can emit has to be listed here too. A code this file has
// never heard of falls through to warning/10, which is how COMPOUND_OVERFLOW
// and TRAILING_INPUT — both errors the core ranks at 90 — would silently sort
// below an approximation in a UI.
for (const code of ["COMPOUND_OVERFLOW", "TRAILING_INPUT"]) {
  const [issue] = rankIssues({
    findings: {
      skipped: [{ code, ref_text: "x", reason: "pinned", span: {} }],
      ambiguities: [],
      approximations: [],
    },
  });
  assert.equal(issue.code, code);
  assert.equal(issue.severity, "error", code);
  assert.equal(issue.rank, 90, code);
  assert.equal(issue.recoverable, true, code);
}

const parsedMatches = parseAllForUi(
  () =>
    JSON.stringify([
      { start: 0, end: 2, text: "3m", parsed: cleanParsed },
      {
        start: 3,
        end: 5,
        text: "4m",
        parsed: { best: { kind: "quantity", value: 4, unit: "m" }, issues: [] },
      },
    ]),
  "3m×4m",
);
assert.equal(parsedMatches.length, 2);
assert.equal(parsedMatches[0].start, 0);
assert.equal(parsedMatches[0].codeUnitStart, 0);
assert.equal(parsedMatches[0].ok, true);

const japaneseMatches = parseAllForUi(
  () =>
    JSON.stringify([
      {
        start: 6,
        end: 11,
        byteStart: 6,
        byteEnd: 11,
        charStart: 2,
        charEnd: 7,
        text: "105mm",
        parsed: cleanParsed,
      },
    ]),
  "壁厚105mm",
);
assert.equal(japaneseMatches[0].codeUnitStart, 2);
assert.equal("壁厚105mm".slice(japaneseMatches[0].codeUnitStart, japaneseMatches[0].codeUnitEnd), "105mm");

const fields = canonicalizeFieldsForUi(() => cleanParsed, [
  { field: "height", text: "180cm" },
]);
assert.equal(fields[0].field, "height");
assert.equal(fields[0].canonical.unit, "m");

const values = canonicalizeValuesForUi(
  () =>
    JSON.stringify([
      {
        field: "width",
        input: "3640",
        ok: false,
        canonical: null,
        parsed: {
          best: { kind: "number", value: 3640 },
          findings: {
            skipped: [],
            ambiguities: [
              {
                code: "UNIT_ASSUMED",
                ref_text: "3640",
                reason: "unitless",
                span: { start: 0, end: 4, text: "3640" },
              },
            ],
            approximations: [],
          },
        },
      },
    ]),
  [],
);
assert.equal(values[0].issues[0].code, "UNIT_ASSUMED");

// The core breaks a rank tie with `String::cmp` (UTF-8 bytes, i.e. code points).
// `localeCompare` disagreed with that on plain ASCII — it puts "a" before "B" —
// so the adapter and the Rust envelope could disagree about which issue is
// issues[0], which is the one a UI surfaces.
{
  const issue = (rank, refText) => ({
    code: "AMBIGUOUS_NUMBER",
    severity: "warning",
    rank,
    recoverable: true,
    ref_text: refText,
    reason: "tie",
    span: { start: 0, end: 1, text: refText },
  });

  const ranked = rankIssues({
    issues: [issue(55, "a"), issue(55, "B"), issue(55, "\uff11"), issue(55, "Z")],
  });
  assert.deepEqual(
    ranked.map((entry) => entry.ref_text),
    ["B", "Z", "a", "\uff11"],
    "equal ranks must break the tie by code point, as String::cmp does",
  );

  // Rank still dominates the tie-break.
  const byRank = rankIssues({ issues: [issue(30, "a"), issue(90, "z")] });
  assert.deepEqual(byRank.map((entry) => entry.rank), [90, 30]);
}


// The browser adapter does not decide acceptance; the Rust core does, and this
// module reports its answer verbatim. It used to derive `ok` from `error`
// severity with no view of the strictness, so a `confirm` field showed green on
// an ambiguity `canonicalize_values` had already refused. Whatever the core
// says — including when it contradicts what the issue list looks like from
// here — is what comes back.
for (const decided of [true, false]) {
  const fromCore = {
    ok: decided,
    best: { kind: "number", value: 1234 },
    issues: [
      {
        code: "AMBIGUOUS_NUMBER",
        severity: "warning",
        rank: 55,
        ref_text: "1,234",
        reason: "grouping",
        span: { start: 0, end: 5, text: "1,234" },
      },
    ],
  };
  assert.equal(acceptsParsed(fromCore), decided);
  assert.equal(parseForUi(() => fromCore, "1,234").ok, decided);
  assert.equal(parseForUi(() => JSON.stringify(fromCore), "1,234").ok, decided);
  assert.equal(
    parseAllForUi(() => [{ start: 0, end: 5, text: "1,234", parsed: fromCore }], "1,234")[0].ok,
    decided,
  );
  assert.equal(canonicalizeFieldsForUi(() => fromCore, [{ field: "w", text: "1,234" }])[0].ok, decided);
}

// A result the core never decided still goes through the one fallback, and the
// fallback can only be stricter: a skipped fragment blocks, and so does any
// finding once a strictness above `forgiving` is declared.
assert.equal(acceptsParsed(cleanParsed), true);
assert.equal(acceptsParsed(timezoneParsed), false);
assert.equal(
  acceptsParsed({
    best: { kind: "number", value: 1234 },
    strictness: "confirm",
    findings: {
      skipped: [],
      ambiguities: [
        {
          code: "AMBIGUOUS_NUMBER",
          ref_text: "1,234",
          reason: "grouping",
          span: { start: 0, end: 5, text: "1,234" },
        },
      ],
      approximations: [],
    },
  }),
  false,
);
assert.equal(acceptsParsed({ best: null, findings: { skipped: [], ambiguities: [], approximations: [] } }), false);

function mockElement(value) {
  const listeners = new Map();
  return {
    value,
    dataset: {},
    attributes: {},
    setAttribute(name, nextValue) {
      this.attributes[name] = String(nextValue);
    },
    removeAttribute(name) {
      delete this.attributes[name];
    },
    addEventListener(name, listener) {
      listeners.set(name, listener);
    },
    removeEventListener(name, listener) {
      if (listeners.get(name) === listener) {
        listeners.delete(name);
      }
    },
    dispatch(name) {
      const listener = listeners.get(name);
      if (listener) {
        listener();
      }
    },
  };
}

// The hand-written `.d.ts` is a second copy of the export list, so it can fall
// behind the module it describes. It already did: `acceptsParsed` shipped
// without a declaration, which left it unimportable from TypeScript while every
// runtime test passed. Compare the two lists instead of trusting them to agree.
{
  const here = new URL(".", import.meta.url);
  const source = await readFile(new URL("../web/unravel-adapters.js", here), "utf8");
  const types = await readFile(new URL("../web/unravel-adapters.d.ts", here), "utf8");
  const named = (text, pattern) =>
    [...text.matchAll(pattern)].map((match) => match[1]).sort();

  const exported = named(source, /^export\s+(?:async\s+)?(?:function|const|class)\s+([A-Za-z_$][\w$]*)/gm);
  const declared = named(types, /^export\s+(?:declare\s+)?(?:function|const|class)\s+([A-Za-z_$][\w$]*)/gm);

  assert.ok(exported.length > 0, "no runtime exports found — the pattern stopped matching");
  assert.deepEqual(
    exported.filter((name) => !declared.includes(name)),
    [],
    "exported from unravel-adapters.js but not declared in unravel-adapters.d.ts",
  );
  assert.deepEqual(
    declared.filter((name) => !exported.includes(name)),
    [],
    "declared in unravel-adapters.d.ts but not exported from unravel-adapters.js",
  );
}
