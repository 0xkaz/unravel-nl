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
  ok: true,
  best: { kind: "quantity", value: 1.8, unit: "m" },
  issues: [],
};

const timezoneParsed = {
  ok: false,
  best: null,
  issues: [
    {
      code: "TIMEZONE_UNSUPPORTED",
      severity: "error",
      rank: 90,
      recoverable: true,
      ref_text: "Europe/Paris",
      reason: "timezone policy required",
      span: { start: 4, end: 16, text: "Europe/Paris" },
    },
  ],
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
  ok: true,
  issues: [
    { code: "AMBIGUOUS_UNIT", severity: "warning", rank: 55, recoverable: true, ref_text: "cup" },
    { code: "APPROXIMATION", severity: "warning", rank: 30, recoverable: true, ref_text: "about" },
  ],
});
assert.deepEqual(
  issues.map((issue) => issue.code),
  ["AMBIGUOUS_UNIT", "APPROXIMATION"],
);

// Metadata is owned by the core envelope. Unknown codes retain the metadata
// the core supplied instead of falling through a second JavaScript table.
assert.deepEqual(
  rankIssues({
    ok: false,
    issues: [
      { code: "FUTURE_CODE", severity: "error", rank: 77, recoverable: false, ref_text: "x" },
    ],
  })[0],
  { code: "FUTURE_CODE", severity: "error", rank: 77, recoverable: false, ref_text: "x" },
);

const parsedMatches = parseAllForUi(
  () =>
    JSON.stringify([
      { start: 0, end: 2, text: "3m", parsed: cleanParsed },
      {
        start: 3,
        end: 5,
        text: "4m",
        parsed: { ok: true, best: { kind: "quantity", value: 4, unit: "m" }, issues: [] },
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
          ok: false,
          best: { kind: "number", value: 3640 },
          issues: [
            {
              code: "UNIT_ASSUMED",
              severity: "info",
              rank: 40,
              recoverable: true,
              ref_text: "3640",
              reason: "unitless",
              span: { start: 0, end: 4, text: "3640" },
            },
          ],
        },
      },
    ]),
  [],
);
assert.equal(values[0].issues[0].code, "UNIT_ASSUMED");

// The core has already ranked the envelope. The adapter preserves that exact
// order instead of maintaining a second sorting implementation.
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

  const coreOrder = [issue(55, "B"), issue(55, "Z"), issue(55, "a"), issue(55, "\uff11")];
  const ranked = rankIssues({ ok: true, issues: coreOrder });
  assert.deepEqual(
    ranked.map((entry) => entry.ref_text),
    ["B", "Z", "a", "\uff11"],
    "the adapter must preserve the core's tie-break",
  );
  assert.equal(ranked, coreOrder, "ranked issues should not be copied and re-sorted");
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

// A result the core never decided is outside the adapter contract and is never
// accepted. The adapter does not maintain a second acceptance policy.
assert.equal(acceptsParsed(cleanParsed), true);
assert.equal(acceptsParsed(timezoneParsed), false);
assert.equal(acceptsParsed({ best: { kind: "number", value: 1234 }, issues: [] }), false);

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
