import assert from "node:assert/strict";
import {
  applyParseState,
  createUnravelFieldController,
  parseForUi,
  rankIssues,
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
