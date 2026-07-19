import type { Dimension, ParsedMatchState, Parser, Scanner } from "../web/unravel-adapters.js";
import { parseAllForUi, parseForUi } from "../web/unravel-adapters.js";

const dimension: Dimension = "length";

const parser: Parser = () => ({
  best: {
    kind: "quantity",
    value: 3,
    unit: "m",
    dimension,
  },
  issues: [],
});

const scanner: Scanner = () => [
  {
    start: 0,
    end: 2,
    byteStart: 0,
    byteEnd: 2,
    charStart: 0,
    charEnd: 2,
    text: "3m",
    parsed: {
      best: {
        kind: "quantity",
        value: 3,
        unit: "m",
        dimension,
      },
      issues: [],
    },
  },
];

const state = parseForUi(parser, "3m", { purpose: "dimension_editor" });
const matches: ParsedMatchState[] = parseAllForUi(scanner, "3m");

state.ok satisfies boolean;
matches[0].codeUnitStart satisfies number | undefined;
