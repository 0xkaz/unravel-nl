import assert from "node:assert/strict";
import React from "../web/node_modules/react/index.js";
import { renderToString } from "../web/node_modules/react-dom/server.node.js";
import { createUnravelReactAdapter } from "../web/unravel-adapters.js";

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
