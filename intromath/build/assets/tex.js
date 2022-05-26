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