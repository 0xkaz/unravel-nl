const DEFAULT_DELAY_MS = 0;

export function parseForUi(parser, text, ctx = undefined) {
  const parsed = parseAdapterResult(parser(text, ctx));
  const issues = rankIssues(parsed);
  const best = parsed && parsed.best ? parsed.best : null;

  return {
    ok: acceptsParsed(parsed),
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
    const ok = acceptsParsed(parsed);
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

/**
 * Whether a parse is acceptable.
 *
 * The Rust core owns this decision — `accepts` in `findings.rs` — and puts the
 * answer in `ok` on every result it serializes. This module deliberately does
 * not re-derive it: deriving it
 * here from `error` severity alone, with no view of the strictness, is what made
 * a `confirm` field show green on an ambiguity the Rust adapter had refused.
 * A hand-built object without the core's decision is therefore not accepted.
 */
export function acceptsParsed(parsed) {
  return Boolean(parsed && parsed.ok === true);
}

export function rankIssues(parsed) {
  return parsed && Array.isArray(parsed.issues) ? parsed.issues : [];
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
