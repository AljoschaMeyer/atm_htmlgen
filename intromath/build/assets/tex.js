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
    return String.raw`\htmlData{preview=http://localhost:1234/previews/set.html}{\href{http://localhost:1234/sets.html#set}{\emptyset}}`;
  } else {
    if (level === 0 || !level) {
      return String.raw` \left\lbrace ${elems.join(", ")} \right\rbrace `;
    } else if (level === 1) {
      return String.raw` \big\lbrace ${elems.join(", ")} \big\rbrace `;
    } else if (level === 2) {
      return String.raw` \Big\lbrace ${elems.join(", ")} \Big\rbrace `;
    } else if (level === 3) {
      return String.raw` \bigg\lbrace ${elems.join(", ")} \bigg\rbrace `;
    } else if (level === 4) {
      return String.raw` \Bigg\lbrace ${elems.join(", ")} \Bigg\rbrace `;
    } else {
      throw "unimplemented level of paren sizing";
    }
  }
}

export function highlight(c_, mode, r, inner_tex) {
  const c = [1, 3, 5, 2, 4, 6, 3, 5, 1, 4, 6, 2, 5, 1, 3, 6, 2, 4][c_ % 18];
  if (c === 1 && mode === "low" && r) {
    return String.raw`\htmlClass{bgclll1 highlight rightspace low}{${inner_tex}}`;
  } else if (c === 2 && mode === "low" && r) {
    return String.raw`\htmlClass{bgclll2 highlight rightspace low}{${inner_tex}}`;
  } else if (c === 3 && mode === "low" && r) {
    return String.raw`\htmlClass{bgclll3 highlight rightspace low}{${inner_tex}}`;
  } else if (c === 4 && mode === "low" && r) {
    return String.raw`\htmlClass{bgclll4 highlight rightspace low}{${inner_tex}}`;
  } else if (c === 5 && mode === "low" && r) {
    return String.raw`\htmlClass{bgclll5 highlight rightspace low}{${inner_tex}}`;
  } else if (c === 6 && mode === "low" && r) {
    return String.raw`\htmlClass{bgclll6 highlight rightspace low}{${inner_tex}}`;
  } else if (c === 1 && mode === "top" && r) {
    return String.raw`\htmlClass{bgclll1 highlight rightspace top}{${inner_tex}}`;
  } else if (c === 2 && mode === "top" && r) {
    return String.raw`\htmlClass{bgclll2 highlight rightspace top}{${inner_tex}}`;
  } else if (c === 3 && mode === "top" && r) {
    return String.raw`\htmlClass{bgclll3 highlight rightspace top}{${inner_tex}}`;
  } else if (c === 4 && mode === "top" && r) {
    return String.raw`\htmlClass{bgclll4 highlight rightspace top}{${inner_tex}}`;
  } else if (c === 5 && mode === "top" && r) {
    return String.raw`\htmlClass{bgclll5 highlight rightspace top}{${inner_tex}}`;
  } else if (c === 6 && mode === "top" && r) {
    return String.raw`\htmlClass{bgclll6 highlight rightspace top}{${inner_tex}}`;
  } else if (c === 1 && mode === "low" && !r) {
    return String.raw`\htmlClass{bgclll1 highlight low}{${inner_tex}}`;
  } else if (c === 2 && mode === "low" && !r) {
    return String.raw`\htmlClass{bgclll2 highlight low}{${inner_tex}}`;
  } else if (c === 3 && mode === "low" && !r) {
    return String.raw`\htmlClass{bgclll3 highlight low}{${inner_tex}}`;
  } else if (c === 4 && mode === "low" && !r) {
    return String.raw`\htmlClass{bgclll4 highlight low}{${inner_tex}}`;
  } else if (c === 5 && mode === "low" && !r) {
    return String.raw`\htmlClass{bgclll5 highlight low}{${inner_tex}}`;
  } else if (c === 6 && mode === "low" && !r) {
    return String.raw`\htmlClass{bgclll6 highlight low}{${inner_tex}}`;
  } else if (c === 1 && mode === "top" && !r) {
    return String.raw`\htmlClass{bgclll1 highlight top}{${inner_tex}}`;
  } else if (c === 2 && mode === "top" && !r) {
    return String.raw`\htmlClass{bgclll2 highlight top}{${inner_tex}}`;
  } else if (c === 3 && mode === "top" && !r) {
    return String.raw`\htmlClass{bgclll3 highlight top}{${inner_tex}}`;
  } else if (c === 4 && mode === "top" && !r) {
    return String.raw`\htmlClass{bgclll4 highlight top}{${inner_tex}}`;
  } else if (c === 5 && mode === "top" && !r) {
    return String.raw`\htmlClass{bgclll5 highlight top}{${inner_tex}}`;
  } else if (c === 6 && mode === "top" && !r) {
    return String.raw`\htmlClass{bgclll6 highlight top}{${inner_tex}}`;
  }
}

export function highlight_raw(c, mode, r) {
  return highlight(c, mode, r, "").slice(0, -2);
}

export function p(inner_tex, level) {
  if (level === 0 || !level) {
    return String.raw` \left( ${inner_tex} \right) `;
  } else if (level === 1) {
    return String.raw` \big( ${inner_tex} \big) `;
  } else if (level === 2) {
    return String.raw` \Big( ${inner_tex} \Big) `;
  } else if (level === 3) {
    return String.raw` \bigg( ${inner_tex} \bigg) `;
  } else if (level === 4) {
    return String.raw` \Bigg( ${inner_tex} \Bigg) `;
  } else {
    throw "unimplemented level of paren sizing";
  }
}

export function powerset(inner_tex, level) {
  if (level === 0 || !level) {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/powerset.html}{\href{http://localhost:1234/sets.html#powerset}{\operatorname{\mathcal{P}}}} \left( ${inner_tex} \right) `;
  } else if (level === 1) {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/powerset.html}{\href{http://localhost:1234/sets.html#powerset}{\operatorname{\mathcal{P}}}} \big( ${inner_tex} \big) `;
  } else if (level === 2) {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/powerset.html}{\href{http://localhost:1234/sets.html#powerset}{\operatorname{\mathcal{P}}}} \Big( ${inner_tex} \Big) `;
  } else if (level === 3) {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/powerset.html}{\href{http://localhost:1234/sets.html#powerset}{\operatorname{\mathcal{P}}}} \bigg( ${inner_tex} \bigg) `;
  } else if (level === 4) {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/powerset.html}{\href{http://localhost:1234/sets.html#powerset}{\operatorname{\mathcal{P}}}} \Bigg( ${inner_tex} \Bigg) `;
  } else {
    throw "unimplemented level of paren sizing";
  }
}

export const defeq = String.raw`\htmlData{preview=http://localhost:1234/previews/defeq.html}{\href{http://localhost:1234/deductive_reasoning.html#defeq}{\coloneqq}}`;
export const seq = String.raw`\htmlData{preview=http://localhost:1234/previews/set_eq.html}{\href{http://localhost:1234/sets.html#set_eq}{=}}`;
export const sneq = String.raw`\htmlData{preview=http://localhost:1234/previews/set_eq.html}{\href{http://localhost:1234/sets.html#set_eq}{\neq}}`;
export const subseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subseteq.html}{\href{http://localhost:1234/sets.html#subseteq}{\subseteq}}`;
export const subset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\subset}}`;
export const supseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subseteq.html}{\href{http://localhost:1234/sets.html#subseteq}{\supseteq}}`;
export const supset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\supset}}`;
export const nsubseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subseteq.html}{\href{http://localhost:1234/sets.html#subseteq}{\nsubseteq}}`;
export const nsubset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\not\subset}}`;
export const nsupseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subseteq.html}{\href{http://localhost:1234/sets.html#subseteq}{\nsupseteq}}`;
export const nsupset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\not\supset}}`;
export const intersection = String.raw`\htmlData{preview=http://localhost:1234/previews/intersection.html}{\href{http://localhost:1234/sets.html#intersection}{\cap}}`;
export const union = String.raw`\htmlData{preview=http://localhost:1234/previews/union.html}{\href{http://localhost:1234/sets.html#union}{\cup}}`;
export const setminus = String.raw`\htmlData{preview=http://localhost:1234/previews/set_difference.html}{\href{http://localhost:1234/sets.html#set_difference}{\setminus}}`;

export const symbol0 = String.raw`\htmlClass{symbol_container}{\char"e904}`;
export const symbol1 = String.raw`\htmlClass{symbol_container}{\char"e903}`;
export const symbol2 = String.raw`\htmlClass{symbol_container}{\char"e902}`;
export const symbol3 = String.raw`\htmlClass{symbol_container}{\char"e901}`;
export const symbol4 = String.raw`\htmlClass{symbol_container}{\char"e900}`;

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