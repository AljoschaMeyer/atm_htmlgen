const tex_options = {trust: true};

export function tex_string(str, opts) {
  if (opts) {
    opts.trust = true;
  }
  return katex.renderToString(str, opts ? opts : tex_options);
}

export function tex(str, elem, opts) {
  if (opts) {
    opts.trust = true;
  }
  return katex.render(str, elem, opts ? opts : tex_options);
}

// elems is an array containing the tex source of the elements of the set
export function set(elems) {
  if (elems.length === 0) {
    return String.raw`§$set§ `;
  } else {
    return String.raw`§$set(${elems.join(", ")})§ `;
  }
}

export function p(inner_tex, level) {
  if (level === 0) {
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
    throw "unimplemented level of paren sizing";
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
