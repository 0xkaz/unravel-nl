import assert from "node:assert/strict";
import { parse_json, parse_json_with_locale } from "../pkg-node/unravel_nl.js";

const length = JSON.parse(parse_json_with_locale("5尺3寸", "ja"));
assert.equal(length.ok, true);
assert.equal(length.best.kind, "quantity");
assert.equal(length.best.unit, "m");
assert.equal(length.best.dimension, "length");

const recurrence = JSON.parse(parse_json("every third business day"));
assert.equal(recurrence.ok, true);
assert.equal(
  recurrence.best.recurrence,
  "FREQ=MONTHLY;BYSETPOS=3;BYDAY=MO,TU,WE,TH,FR",
);

const unsupportedTimezone = JSON.parse(parse_json("3pm Europe/Paris"));
assert.equal(unsupportedTimezone.ok, false);
assert.equal(unsupportedTimezone.issues[0].code, "TIMEZONE_UNSUPPORTED");
assert.equal(unsupportedTimezone.issues[0].severity, "error");
