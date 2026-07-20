const DEFAULT_DELAY_MS = 0;

export function parseForUi(parser, text, ctx = undefined) {
  const parsed = parseAdapterResult(parser(text, ctx));
  const issues = rankIssues(parsed);
  const best = parsed && parsed.best ? parsed.best : null;

  return {
    ok: Boolean(best) && !issues.some((issue) => issue.severity === "error"),
    parsed,
    best,
    issues,
    message: issues.length > 0 ? formatIssue(issues[0]) : null,
  };
}

export function parseAllForUi(parser, text, ctx = undefined) {
  const matches = parseAdapterResult(parser(text, ctx)) || [];
  let searchFrom = 0;
  return matches.map((rawMatch) => {
    const match = normalizeMatchSpan(rawMatch, text, searchFrom);
    if (Number.isInteger(match.codeUnitEnd)) {
      searchFrom = match.codeUnitEnd;
    }
    const parsed = match.parsed || match;
    const issues = rankIssues(parsed);
    const best = parsed && parsed.best ? parsed.best : null;
    const ok = Boolean(best) && !issues.some((issue) => issue.severity === "error");
    return {
      ...match,
      ok,
      parsed,
      best,
      issues,
      message: issues.length > 0 ? formatIssue(issues[0]) : null,
    };
  });
}

export function canonicalizeFieldsForUi(parser, requests) {
  return requests.map((request) => {
    const input = request.text ?? request.input ?? "";
    const state = parseForUi(parser, input, request.ctx);
    return {
      field: request.field,
      input,
      ok: state.ok,
      canonical: state.ok ? state.best : null,
      parsed: state.parsed,
      issues: state.issues,
      message: state.message,
    };
  });
}

export function canonicalizeValuesForUi(canonicalizer, requests) {
  const values = parseAdapterResult(canonicalizer(requests)) || [];
  return values.map((value) => {
    const issues = rankIssues(value.parsed);
    return {
      ...value,
      issues,
      message: value.message ?? (issues.length > 0 ? formatIssue(issues[0]) : null),
    };
  });
}

export function applyParseState(element, state) {
  const best = state.best || {};
  const topIssue = state.issues[0] || null;

  element.dataset.unravelOk = state.ok ? "true" : "false";
  setDatasetValue(element, "unravelKind", best.kind);
  setDatasetValue(element, "unravelUnit", best.unit);
  setDatasetValue(element, "unravelValue", best.value);
  setDatasetValue(element, "unravelDate", best.date);
  setDatasetValue(element, "unravelRecurrence", best.recurrence);
  setDatasetValue(element, "unravelIssueCode", topIssue && topIssue.code);
  setDatasetValue(element, "unravelIssueSeverity", topIssue && topIssue.severity);

  if (state.ok) {
    element.removeAttribute("aria-invalid");
    element.removeAttribute("title");
  } else {
    element.setAttribute("aria-invalid", "true");
    if (state.message) {
      element.setAttribute("title", state.message);
    }
  }

  return state;
}

export function parseInputElement(element, parser, ctx = undefined) {
  return applyParseState(element, parseForUi(parser, element.value || "", ctx));
}

export function createUnravelFieldController(element, parser, options = {}) {
  const ctx = options.ctx;
  const delayMs = options.delayMs ?? DEFAULT_DELAY_MS;
  let timer = null;
  let state = parseInputElement(element, parser, ctx);

  const run = () => {
    state = parseInputElement(element, parser, ctx);
    if (typeof options.onChange === "function") {
      options.onChange(state);
    }
  };

  const schedule = () => {
    if (timer !== null) {
      clearTimeout(timer);
    }
    if (delayMs > 0) {
      timer = setTimeout(run, delayMs);
    } else {
      run();
    }
  };

  element.addEventListener("input", schedule);

  return {
    get state() {
      return state;
    },
    parse: run,
    disconnect() {
      if (timer !== null) {
        clearTimeout(timer);
      }
      element.removeEventListener("input", schedule);
    },
  };
}

export function createUnravelReactAdapter(React, parser) {
  function useUnravelValue(initialValue = "", options = {}) {
    const [value, setValue] = React.useState(initialValue);
    const [state, setState] = React.useState(() => parseForUi(parser, initialValue, options.ctx));

    const onChange = React.useCallback(
      (event) => {
        const nextValue = event.target.value;
        setValue(nextValue);
        setState(parseForUi(parser, nextValue, options.ctx));
      },
      [options.ctx],
    );

    return { value, setValue, state, onChange };
  }

  function UnravelInput(props) {
    const { ctx, onParsed, defaultValue = "", ...inputProps } = props;
    const model = useUnravelValue(defaultValue, { ctx });

    React.useEffect(() => {
      if (typeof onParsed === "function") {
        onParsed(model.state);
      }
    }, [model.state, onParsed]);

    return React.createElement("input", {
      ...inputProps,
      value: model.value,
      onChange: model.onChange,
      "aria-invalid": model.state.ok ? undefined : true,
      "data-unravel-ok": model.state.ok ? "true" : "false",
      "data-unravel-kind": model.state.best && model.state.best.kind,
      title: model.state.message || undefined,
    });
  }

  return { useUnravelValue, UnravelInput };
}

export function defineUnravelElement(parser, options = {}) {
  const registry = options.customElements || globalThis.customElements;
  const BaseHTMLElement = options.HTMLElement || globalThis.HTMLElement;
  const tagName = options.tagName || "unravel-input";

  if (!registry || !BaseHTMLElement) {
    return null;
  }
  if (registry.get(tagName)) {
    return registry.get(tagName);
  }

  class UnravelInputElement extends BaseHTMLElement {
    connectedCallback() {
      if (!this.input) {
        this.input = document.createElement("input");
        this.appendChild(this.input);
      }
      this.controller = createUnravelFieldController(this.input, parser, {
        ctx: options.ctx,
        onChange: (state) => {
          this.dispatchEvent(new CustomEvent("unravel-parse", { detail: state }));
        },
      });
    }

    disconnectedCallback() {
      if (this.controller) {
        this.controller.disconnect();
      }
    }

    get value() {
      return this.input ? this.input.value : "";
    }

    set value(nextValue) {
      if (!this.input) {
        this.input = document.createElement("input");
        this.appendChild(this.input);
      }
      this.input.value = nextValue;
      if (this.controller) {
        this.controller.parse();
      }
    }
  }

  registry.define(tagName, UnravelInputElement);
  return UnravelInputElement;
}

/**
 * Orders two reference texts the way the Rust core does.
 *
 * The core breaks a rank tie with `String::cmp`, which compares UTF-8 bytes and
 * therefore code points. `localeCompare` does not: it puts `"a"` before `"B"`
 * where the core puts `"B"` first, so the two sides disagreed on which issue is
 * `issues[0]` — and that is the one a UI shows.
 */
function compareRefText(left, right) {
  const a = Array.from(String(left ?? ""));
  const b = Array.from(String(right ?? ""));
  const shared = Math.min(a.length, b.length);
  for (let index = 0; index < shared; index += 1) {
    const diff = a[index].codePointAt(0) - b[index].codePointAt(0);
    if (diff !== 0) {
      return diff < 0 ? -1 : 1;
    }
  }
  return a.length === b.length ? 0 : a.length < b.length ? -1 : 1;
}

export function rankIssues(parsed) {
  if (parsed && Array.isArray(parsed.issues)) {
    return parsed.issues
      .map((issue) => ({
        code: issue.code,
        severity: issue.severity ?? issueSeverity(issue.code),
        rank: issue.rank ?? issueRank(issue.code),
        recoverable: issue.recoverable ?? issueRecoverable(issue.code),
        ref_text: issue.ref_text,
        reason: issue.reason,
        span: issue.span,
      }))
      .sort((a, b) => b.rank - a.rank || compareRefText(a.ref_text, b.ref_text));
  }
  const findings = (parsed && parsed.findings) || {};
  const issues = [
    ...mapIssues(findings.skipped || []),
    ...mapIssues(findings.ambiguities || []),
    ...mapIssues(findings.approximations || []),
  ];
  return issues.sort((a, b) => b.rank - a.rank || compareRefText(a.ref_text, b.ref_text));
}

function mapIssues(issues) {
  return issues.map((issue) => ({
    code: issue.code,
    severity: issueSeverity(issue.code),
    rank: issueRank(issue.code),
    recoverable: issueRecoverable(issue.code),
    ref_text: issue.ref_text,
    reason: issue.reason,
    span: issue.span,
  }));
}

function issueSeverity(code) {
  switch (code) {
    case "EMPTY":
    case "NO_VALUE":
    case "UNKNOWN_UNIT":
    case "TIMEZONE_UNSUPPORTED":
    case "RECURRENCE_UNSUPPORTED":
    case "REJECTED_BY_POLICY":
      return "error";
    case "UNIT_ASSUMED":
      return "info";
    default:
      return "warning";
  }
}

function issueRank(code) {
  switch (code) {
    case "EMPTY":
    case "NO_VALUE":
      return 100;
    case "TIMEZONE_UNSUPPORTED":
    case "RECURRENCE_UNSUPPORTED":
    case "REJECTED_BY_POLICY":
      return 90;
    case "UNKNOWN_UNIT":
      return 80;
    case "TYPO_CORRECTED":
      return 65;
    case "AMBIGUOUS_NUMBER":
    case "AMBIGUOUS_DATE":
    case "AMBIGUOUS_UNIT":
    case "AMBIGUOUS_CURRENCY":
      return 55;
    case "UNIT_ASSUMED":
      return 40;
    case "APPROXIMATION":
      return 30;
    default:
      return 10;
  }
}

function issueRecoverable(code) {
  return code !== "EMPTY" && code !== "NO_VALUE";
}

function setDatasetValue(element, key, value) {
  if (value === undefined || value === null) {
    delete element.dataset[key];
  } else {
    element.dataset[key] = String(value);
  }
}

function formatIssue(issue) {
  return `[${issue.code}] ${issue.reason}`;
}

function parseAdapterResult(value) {
  if (typeof value === "string") {
    return JSON.parse(value);
  }
  return value;
}

function normalizeMatchSpan(match, sourceText, searchFrom) {
  if (!match || typeof match.text !== "string") {
    return match;
  }
  let codeUnitStart = sourceText.indexOf(match.text, searchFrom);
  if (codeUnitStart < 0) {
    codeUnitStart = sourceText.indexOf(match.text);
  }
  if (codeUnitStart < 0) {
    return match;
  }
  return {
    ...match,
    codeUnitStart,
    codeUnitEnd: codeUnitStart + match.text.length,
  };
}
