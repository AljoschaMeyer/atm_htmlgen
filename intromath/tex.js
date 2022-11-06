const strictness = (err) => {
  if (err === "htmlExtension") {
    return "ignore";
  } else {
    return "warn";
  }
}
const tex_options = {
  trust: true,
  strict: strictness,
};

export function tex_string(str, opts) {
  if (opts) {
    opts.trust = true;
    opts.strict = strictness;
  }
  return katex.renderToString(str, opts ? opts : tex_options);
}

export function tex(str, elem, opts) {
  if (opts) {
    opts.trust = true;
    opts.strict = strictness;
  }
  return katex.render(str, elem, opts ? opts : tex_options);
}

// elems is an array containing the tex source of the elements of the set
export function set(elems, level) {
  if (elems.length === 0) {
    return String.raw`§$set§ `;
  } else {
    if (level === 0 || !level) {
      return String.raw`§$set(${elems.join(", ")})§ `;
    } else if (level === 1) {
      return String.raw`§$set[1](${elems.join(", ")})§ `;
    } else if (level === 2) {
      return String.raw`§$set[2](${elems.join(", ")})§ `;
    } else if (level === 3) {
      return String.raw`§$set[3](${elems.join(", ")})§ `;
    } else if (level === 4) {
      return String.raw`§$set[4](${elems.join(", ")})§ `;
    } else {
      return String.raw`§$set[4](${elems.join(", ")})§ `;
    }
  }
}

export function highlight(c_, mode, r, inner_tex) {
  const c = [1, 3, 5, 2, 4, 6, 3, 5, 1, 4, 6, 2, 5, 1, 3, 6, 2, 4][c_ % 18];
  if (c === 1 && mode === "low" && r) {
    return String.raw`§$highlightlowr1(${inner_tex})`;
  } else if (c === 2 && mode === "low" && r) {
    return String.raw`§$highlightlowr2(${inner_tex})`;
  } else if (c === 3 && mode === "low" && r) {
    return String.raw`§$highlightlowr3(${inner_tex})`;
  } else if (c === 4 && mode === "low" && r) {
    return String.raw`§$highlightlowr4(${inner_tex})`;
  } else if (c === 5 && mode === "low" && r) {
    return String.raw`§$highlightlowr5(${inner_tex})`;
  } else if (c === 6 && mode === "low" && r) {
    return String.raw`§$highlightlowr6(${inner_tex})`;
  } else if (c === 1 && mode === "top" && r) {
    return String.raw`§$highlighttopr1(${inner_tex})`;
  } else if (c === 2 && mode === "top" && r) {
    return String.raw`§$highlighttopr2(${inner_tex})`;
  } else if (c === 3 && mode === "top" && r) {
    return String.raw`§$highlighttopr3(${inner_tex})`;
  } else if (c === 4 && mode === "top" && r) {
    return String.raw`§$highlighttopr4(${inner_tex})`;
  } else if (c === 5 && mode === "top" && r) {
    return String.raw`§$highlighttopr5(${inner_tex})`;
  } else if (c === 6 && mode === "top" && r) {
    return String.raw`§$highlighttopr6(${inner_tex})`;
  } else if (c === 1 && mode === "low" && !r) {
    return String.raw`§$highlightlow1(${inner_tex})`;
  } else if (c === 2 && mode === "low" && !r) {
    return String.raw`§$highlightlow2(${inner_tex})`;
  } else if (c === 3 && mode === "low" && !r) {
    return String.raw`§$highlightlow3(${inner_tex})`;
  } else if (c === 4 && mode === "low" && !r) {
    return String.raw`§$highlightlow4(${inner_tex})`;
  } else if (c === 5 && mode === "low" && !r) {
    return String.raw`§$highlightlow5(${inner_tex})`;
  } else if (c === 6 && mode === "low" && !r) {
    return String.raw`§$highlightlow6(${inner_tex})`;
  } else if (c === 1 && mode === "top" && !r) {
    return String.raw`§$highlighttop1(${inner_tex})`;
  } else if (c === 2 && mode === "top" && !r) {
    return String.raw`§$highlighttop2(${inner_tex})`;
  } else if (c === 3 && mode === "top" && !r) {
    return String.raw`§$highlighttop3(${inner_tex})`;
  } else if (c === 4 && mode === "top" && !r) {
    return String.raw`§$highlighttop4(${inner_tex})`;
  } else if (c === 5 && mode === "top" && !r) {
    return String.raw`§$highlighttop5(${inner_tex})`;
  } else if (c === 6 && mode === "top" && !r) {
    return String.raw`§$highlighttop6(${inner_tex})`;
  }
}

export function highlight_raw(c, mode, r) {
  return highlight(c, mode, r, "").slice(0, -2);
}

export function p(inner_tex, level) {
  if (level === 0 || !level) {
    return String.raw`§$p(${inner_tex})§ `;
  } else if (level === 1) {
    return String.raw`§$p[1](${inner_tex})§ `;
  } else if (level === 2) {
    return String.raw`§$p[2](${inner_tex})§ `;
  } else if (level === 3) {
    return String.raw`§$p[3](${inner_tex})§ `;
  } else if (level === 4) {
    return String.raw`§$p[4](${inner_tex})§ `;
  } else {
    return String.raw`§$p[4](${inner_tex})§ `;
  }
}

export function powerset(inner_tex, level) {
  if (level === 0 || !level) {
    return String.raw`§$powerset(${inner_tex})§ `;
  } else if (level === 1) {
    return String.raw`§$powerset[1](${inner_tex})§ `;
  } else if (level === 2) {
    return String.raw`§$powerset[2](${inner_tex})§ `;
  } else if (level === 3) {
    return String.raw`§$powerset[3](${inner_tex})§ `;
  } else if (level === 4) {
    return String.raw`§$powerset[4](${inner_tex})§ `;
  } else {
    return String.raw`§$powerset[4](${inner_tex})§ `;
  }
}

export const defeq = String.raw`§$defeq§ `;
export const seq = String.raw`§$seq§ `;
export const sneq = String.raw`§$sneq§ `;
export const subseteq = String.raw`§$subseteq§ `;
export const subset = String.raw`§$subset§ `;
export const supseteq = String.raw`§$supseteq§ `;
export const supset = String.raw`§$supset§ `;
export const nsubseteq = String.raw`§$nsubseteq§ `;
export const nsubset = String.raw`§$nsubset§ `;
export const nsupseteq = String.raw`§$nsupseteq§ `;
export const nsupset = String.raw`§$nsupset§ `;
export const intersection = String.raw`§$intersection§ `;
export const union = String.raw`§$union§ `;
export const setminus = String.raw`§$setminus§ `;

export const symbol0 = String.raw`§$symbol0§ `;
export const symbol1 = String.raw`§$symbol1§ `;
export const symbol2 = String.raw`§$symbol2§ `;
export const symbol3 = String.raw`§$symbol3§ `;
export const symbol4 = String.raw`§$symbol4§ `;

export function symbol(i) {
  if (i === 0) {
    return symbol0;
  } else if (i === 1) {
    return symbol1;
  } else if (i === 2) {
    return symbol2;
  } else if (i === 3) {
    return symbol3;
  } else if (i === 4) {
    return symbol4;
  }
}
