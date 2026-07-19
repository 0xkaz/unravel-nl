import assert from "node:assert/strict";
import React from "../web/node_modules/react/index.js";
import { renderToString } from "../web/node_modules/react-dom/server.node.js";
import { createUnravelReactAdapter } from "../web/unravel-adapters.js";

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

const parser = (text) => (text.includes("Europe/Paris") ? timezoneParsed : cleanParsed);
const { UnravelInput } = createUnravelReactAdapter(React, parser);

const okMarkup = renderToString(React.createElement(UnravelInput, { defaultValue: "180cm" }));
assert.match(okMarkup, /data-unravel-ok="true"/);
assert.match(okMarkup, /data-unravel-kind="quantity"/);
assert.doesNotMatch(okMarkup, /aria-invalid/);

const failedMarkup = renderToString(
  React.createElement(UnravelInput, { defaultValue: "3pm Europe/Paris" }),
);
assert.match(failedMarkup, /data-unravel-ok="false"/);
assert.match(failedMarkup, /aria-invalid="true"/);
assert.match(failedMarkup, /TIMEZONE_UNSUPPORTED/);
