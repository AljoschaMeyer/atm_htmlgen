import { tex, tex_string, defeq, set, seq, sneq, subseteq, subset, supseteq, supset, nsubseteq, nsubset, nsupseteq, nsupset } from './tex.js';

const svgns = "http://www.w3.org/2000/svg";
const PI = Math.PI;
const R = 10;

function euler(container, compute_s3, render_results) {
  const s1 = [false, true, true, false, true];
  const s2 = [false, true, false, true, false];

  const svg = container.children[0];

  const p1 = svg.children[0];
  const p2 = svg.children[1];

  const svg_elements = [];
  for (let i = 0; i < 5; i++) {
    svg_elements.push(svg.children[2 + i]);
  }

  const buttons = container.children[1];
  const buttons1 = buttons.children[0];
  const buttons2 = buttons.children[1];

  for (let i = 0; i < 5; i++) {
    buttons1.children[i].addEventListener("click", () => {
      s1[i] = !s1[i];
      render_state();
    });

    buttons2.children[i].addEventListener("click", () => {
      s2[i] = !s2[i];
      render_state();
    });
  }

  const set1 = container.children[2].children[0];
  const set2 = container.children[2].children[1];

  const results = container.children[3];

  render_state();

  function render_state() {
    const s3 = compute_s3 ? compute_s3(s1, s2) : empty_s();

    draw_set(s1, p1);
    draw_set(s2, p2);

    for (let i = 0; i < 5; i++) {
      svg_elements[i].classList.toggle("s3", s3[i]);
    }

    for (let i = 0; i < 5; i++) {
      buttons1.children[i].classList.toggle("in", s1[i]);
      buttons1.children[i].innerHTML = button_text(i, 1, s1[i]);
      buttons2.children[i].classList.toggle("in", s2[i]);
      buttons2.children[i].innerHTML = button_text(i, 2, s2[i]);
    }

    set1.innerHTML = render_set_def(1, s1);
    set2.innerHTML = render_set_def(2, s2);

    if (render_results) {
      render_results(results, s1, s2, s3);
    }
  }
}

function empty_s() {
  return [false, false, false, false, false];
}

function name_set(set) {
  return `\\htmlClass{c${set}}{${set === 1 ? "A" : "B"}}`;
}

function button_text(element, set, is_in) {
  return tex_string(`${tex_symbol(element)} ${is_in ? "\\in" : "\\notin"} ${name_set(set)}`);
}

function render_set_def(set, s, s3) {
  return tex_string(`${name_set(set)} ${defeq} ${set_tex(s, s3)}`);
}

function set_tex(s, s3_) {
  const s3 = s3_ ? s3_ : [false, false, false, false, false];

  const elements = s.reduce((acc, element, i) => {
    if (element) {
      if (s3[i]) {
        acc.push(`\\htmlClass{s3}{${tex_symbol(i)}}`);
      } else {
        acc.push(tex_symbol(i));
      }
    }
    return acc;
  }, []);

  return set(elements);
}

function tex_symbol(i) {
  return `\\htmlClass{symbol_container}{\\htmlClass{symbol${i}}{}}`;
}

function set_symmetric_difference(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] != s2[i]);
  }

  return s3;
}

function set_difference(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] && !s2[i]);
  }

  return s3;
}

function set_eq(a1, a2) {
  if (a1.length != a2.length) {
    return false;
  }

  for (let i = 0; i < a1.length; i++) {
    if (a1[i] != a2[i]) {
      return false;
    }
  }

  return true;
}

const container_vanilla = document.querySelector("#container_euler_vanilla");
euler(container_vanilla, () => [false, false, false, false, false], () => {});

const container_equality = document.querySelector("#container_euler_equality");
euler(container_equality, set_symmetric_difference, (container, s1, s2, s3) => {
  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, s3);
  const set2 = set_tex(s2, s3);
  const rel = cardinality(s3) === 0 ? seq : sneq;

  return tex(`${s1_name} ${seq} ${set1} ${rel} ${set2} ${seq} ${s2_name}`, container);
});

const container_subset = document.querySelector("#container_euler_subset");
euler(container_subset, () => [false, false, false, false, false], (container, s1, s2, s3) => {
  const empty = empty_s();

  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, empty);
  const set2 = set_tex(s2, empty);
  const set1_2 = set_tex(s1, set_difference(s1, s2));
  const set2_2 = set_tex(s2, set_difference(s2, s1));

  let is_subseteq = true;
  let is_supseteq = true;
  for (let i = 0; i < s1.length; i++) {
    if (s1[i] && !s2[i]) {
      is_subseteq = false;
    }
    if (!s1[i] && s2[i]) {
      is_supseteq = false;
    }
  }
  const eq = set_eq(s1, s2);

  tex(`${s1_name} ${seq} ${set1_2} ${is_subseteq ? (eq ? subseteq : subset) : nsubseteq} ${set2} ${seq} ${s2_name}`, container.children[0]);
  tex(`${s1_name} ${seq} ${set1} ${is_supseteq ? (eq ? supseteq : supset) : nsupseteq} ${set2_2} ${seq} ${s2_name}`, container.children[1]);
});

function polar_to_cartesian([x, y], r, t) {
  return [r * Math.cos(t) + x, r * Math.sin(t) + y];
}

function element_cartesian(i) {
  return polar_to_cartesian([0, 0], 70, (PI * 1.5) + ((2 * PI * i) / 5));
}

function svg_label(label, [x, y]) {
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

function svg_path(clazz) {
  const p = document.createElementNS(svgns, "path");
  p.setAttribute("class", clazz);
  return p;
}

function cardinality(s) {
  let count = 0;
  s.forEach(x => {
    if (x) {
      count++;
    }
  });
  return count;
}

function set_first(s) {
  return set_next(s, 0);
}

function set_next(s, i) {
  for (let j = 1; j <= s.length; j++) {
    if (s[(i + j) % s.length]) {
      return (i + j) % s.length;
    }
  }

  throw "empty set has no next element";
}

function set_previous(s, i) {
  for (let j = s.length - 1; j >= 0; j--) {
    if (s[(i + j) % s.length]) {
      return (i + j) % s.length;
    }
  }

  throw "empty set has no previous element";
}

function draw_set(s, p) {
  const c = cardinality(s);
  const r = R + (2.5*c);

  if (c === 0) {
    p.setAttribute("d", "M0,0");
  } else if (c === 1) {
    const i = set_first(s);
    const [x, y] = element_cartesian(i);
    p.setAttribute("d", `${describe_arc([x, y], r, 0, PI, true)} ${describe_arc([x, y], r, PI, 0, false)}`);
  } else {
    const segments = [];
    let current = set_first(s);

    for (let i = 0; i < c; i++) {
      current = set_previous(s, current);
      const previous = set_previous(s, current);
      const next = set_next(s, current);

      const previous_cartesian = element_cartesian(previous);
      const current_cartesian = element_cartesian(current);
      const next_cartesian = element_cartesian(next);

      const angle_current_out = angle_to_y([next_cartesian, current_cartesian]);
      const angle_current_in = angle_to_y([current_cartesian, previous_cartesian]);

      segments.push(describe_arc(current_cartesian, r, angle_current_in, angle_current_out, i == 0));

      const angle_previous_out = angle_to_y([current_cartesian, previous_cartesian]);
      const previous_out_cartesian = polar_to_cartesian(previous_cartesian, r, angle_previous_out);

      segments.push(`L ${previous_out_cartesian[0]}, ${previous_out_cartesian[1]}`);
    }

    p.setAttribute("d", segments.join(" "));
  }
}

function angle_to_x([[x1, y1], [x2, y2]]) {
  return Math.atan2(y2 - y1, x2 - x1);
}

function angle_to_y(line) {
  return angle_to_x(line) + (PI * 0.5);
}

function describe_arc([x, y], radius, startAngle, endAngle, do_move){
  const [start_x, start_y] = polar_to_cartesian([x, y], radius, endAngle);
  const [end_x, end_y] = polar_to_cartesian([x, y], radius, startAngle);
  const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";

  const d = !!do_move ? ["M", start_x, start_y] : [];
  return d.concat(["A", radius, radius, 0, largeArcFlag, 0, end_x, end_y]).join(" ");
}
