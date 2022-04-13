const tex_options = {trust: true};

export function tex_string(str) {
  return katex.renderToString(str, tex_options);
}

export function tex(str, elem) {
  return katex.render(str, elem, tex_options);
}

// elems is an array containing the tex source of the elements of the set
export function set(elems) {
  if (elems.length === 0) {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/set.html}{\href{http://localhost:1234/sets.html#set}{\emptyset}}`;
  } else {
    return String.raw`\htmlData{preview=http://localhost:1234/previews/set.html}{\href{http://localhost:1234/sets.html#set}{\lbrace}}${elems.join(", ")}\htmlData{preview=http://localhost:1234/previews/set.html}{\href{http://localhost:1234/sets.html#set}{\rbrace}}`;
  }
}

export const defeq = String.raw`\htmlData{preview=http://localhost:1234/previews/defeq.html}{\href{http://localhost:1234/deductive_reasoning.html#defeq}{\coloneqq}}`;
export const seq = String.raw`\htmlData{preview=http://localhost:1234/previews/set_eq.html}{\href{http://localhost:1234/sets.html#set_eq}{=}}`;
export const sneq = String.raw`\htmlData{preview=http://localhost:1234/previews/set_eq.html}{\href{http://localhost:1234/sets.html#set_eq}{\neq}}`;
export const subseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\subseteq}}`;
export const subset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\subset}}`;
export const supseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\supseteq}}`;
export const supset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\supset}}`;
export const nsubseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\nsubseteq}}`;
export const nsubset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\not\subset}}`;
export const nsupseteq = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\nsupseteq}}`;
export const nsupset = String.raw`\htmlData{preview=http://localhost:1234/previews/subset.html}{\href{http://localhost:1234/sets.html#subset}{\not\supset}}`;