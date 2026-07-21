import assert from "node:assert/strict";
import {
  parse_dimensions_for_editor_json_with_context,
  parse_json,
  parse_json_with_context,
  parse_json_with_locale,
} from "../pkg-node/unravel_nl.js";

const length = JSON.parse(parse_json_with_locale("5尺3寸", "ja"));
assert.equal(length.ok, true);
assert.equal(length.best.kind, "quantity");
assert.equal(length.best.unit, "m");
assert.equal(length.best.dimension, "length");

// There is no recurrence surface. `every third business day` used to come back
// as an RRULE string; it now comes back refused, over the same findings channel
// any other unreadable input uses. The removal is only honest if it is loud.
const recurrence = JSON.parse(parse_json("every third business day"));
assert.equal(recurrence.ok, false);
assert.equal(recurrence.best, null);
assert.equal(recurrence.issues[0].code, "NO_VALUE");
assert.equal(recurrence.best?.recurrence, undefined);

const unsupportedTimezone = JSON.parse(parse_json("3pm Europe/Paris"));
assert.equal(unsupportedTimezone.ok, false);
assert.equal(unsupportedTimezone.issues[0].code, "TIMEZONE_UNSUPPORTED");
assert.equal(unsupportedTimezone.issues[0].severity, "error");

const room = JSON.parse(
  parse_dimensions_for_editor_json_with_context("3m×4m のLDK", "ja", "", ""),
);
assert.equal(room.length, 2);
assert.equal(room[0].text, "3m");
assert.equal(room[0].start, 0);
assert.equal(room[0].byteStart, 0);
assert.equal(room[0].charStart, 0);
assert.equal(room[1].text, "4m");
assert.equal(room[1].start, 4);
assert.equal(room[1].byteStart, 4);
assert.equal(room[1].charStart, 3);

const plainLength = JSON.parse(
  parse_dimensions_for_editor_json_with_context("寸法3640", "ja", "length", ""),
);
assert.equal(plainLength.length, 1);
assert.equal(plainLength[0].parsed.best.kind, "number");
assert.equal(plainLength[0].parsed.issues[0].code, "UNIT_ASSUMED");

const editorDimensions = JSON.parse(
  parse_dimensions_for_editor_json_with_context(
    "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
    "ja",
    "",
    "",
  ),
);
assert.deepEqual(
  editorDimensions.map((match) => match.text),
  ["3m", "4m", "6帖", "3640"],
);
assert.equal(editorDimensions[3].parsed.best.kind, "number");
assert.equal(editorDimensions[3].parsed.issues[0].code, "UNIT_ASSUMED");

const strictApproximation = JSON.parse(parse_json_with_context("約3m", "ja", "length", "strict"));
assert.equal(strictApproximation.ok, false);
assert.equal(strictApproximation.issues[0].code, "APPROXIMATION");
