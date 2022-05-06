export const svgns = "http://www.w3.org/2000/svg";

export function svg_label(label, [x, y]) {
  const e = document.createElementNS(svgns, "text");
  e.setAttribute("x", x);
  e.setAttribute("y", y);
  e.setAttribute("class", "label");
  e.setAttribute("text-anchor", "middle");
  e.setAttribute("dominant-baseline", "middle");
  e.setAttribute("font-size", "20px");
  e.textContent = label;
  return e;
};

export function svg_path(clazz) {
  const path = document.createElementNS(svgns, "path");
  path.setAttribute("class", clazz);
  return path;
}

export function convex_path_string(description) {
  const segments = [];

  if (description.length > 0) {
    segments.push(`M ${description[0].in[0]} ${description[0].in[1]}`);
  }

  for (let i = 0; i < description.length; i++) {
    const x = description[i];
    segments.push(describe_arc(x.center, x.r, x.in, x.out, x.large_arc, x.sweep));
    segments.push(`L ${x.in_next[0]} ${x.in_next[1]}`);
  }

  return segments.join(" ");
}

function describe_arc([x, y], r, [start_x, start_y], [end_x, end_y], large_arc, sweep){
  return ["A", r, r, 0, large_arc, sweep, end_x, end_y].join(" ");
}
