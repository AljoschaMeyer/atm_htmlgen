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
    return String.raw`§$set§ `;
  } else {
    return String.raw`§$set(${elems.join(", ")})§ `;
  }
}

export const defeq = String.raw`§$defeq§ `;
