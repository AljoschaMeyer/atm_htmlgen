import { tex, tex_string, defeq, set, seq, sneq, subseteq, subset, supseteq, supset, nsubseteq, nsubset, nsupseteq, nsupset, intersection, union, setminus, p } from './tex.js';

const svgns = "http://www.w3.org/2000/svg";
const PI = Math.PI;
const R = 10;

function euler(container, compute_s3, render_results, prefix) {
  const s1 = [false, true, true, false, true];
  const s2 = [false, true, false, true, false];

  const svg = container.children[0];

  const p1 = svg.children[0];
  const p2 = svg.children[1];

  const svg_elements = [];
  for (let i = 5; i > 0; i--) {
    svg_elements.push(svg.children[svg.children.length - i]);
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

  const clip1 = prefix ? document.querySelector(`#${prefix}_clip1_euler_path`) : null;
  const clip2 = prefix ? document.querySelector(`#${prefix}_clip2_euler_path`) : null;
  const mask1 = prefix ? document.querySelector(`#${prefix}_mask1_euler_path`) : null;
  const mask2 = prefix ? document.querySelector(`#${prefix}_mask2_euler_path`) : null;

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
      buttons1.children[i].classList.toggle("yes", s1[i]);
      buttons1.children[i].classList.toggle("no", !s1[i]);
      buttons1.children[i].innerHTML = button_text(i, 1, s1[i]);
      buttons2.children[i].classList.toggle("yes", s2[i]);
      buttons2.children[i].classList.toggle("no", !s2[i]);
      buttons2.children[i].innerHTML = button_text(i, 2, s2[i]);
    }

    set1.innerHTML = render_set_def(1, s1);
    set2.innerHTML = render_set_def(2, s2);

    if (render_results) {
      render_results(results, s1, s2, s3);
    }

    if (clip1) {
      draw_set(s1, clip1);
    }
    if (clip2) {
      draw_set(s2, clip2);
    }
    if (mask1) {
      draw_set(s1, mask1);
    }
    if (mask2) {
      draw_set(s2, mask2);
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

function set_tex_vanilla(s) {
  const elements = s.reduce((acc, element, i) => {
    if (element) {
      acc.push(`${i}`);
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

function set_intersection(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] && s2[i]);
  }

  return s3;
}

function set_union(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] || s2[i]);
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

const container_subseteq = document.querySelector("#container_euler_subseteq");
euler(container_subseteq, () => [false, false, false, false, false], (container, s1, s2, s3) => {
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

  tex(`${s1_name} ${seq} ${set1_2} ${is_subseteq ? subseteq : nsubseteq} ${set2} ${seq} ${s2_name}`, container.children[0]);
  tex(`${s1_name} ${seq} ${set1} ${is_supseteq ? supseteq : nsupseteq} ${set2_2} ${seq} ${s2_name}`, container.children[1]);
});

const container_intersection = document.querySelector("#container_euler_intersection");
euler(container_intersection, set_intersection, (container, s1, s2, s3) => {
  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, s3);
  const set2 = set_tex(s2, s3);
  const set3 = set_tex(s3, s3);

  return tex(`${s1_name} ${intersection} ${s2_name} ${seq} ${set1} ${intersection} ${set2} ${seq} ${set3}`, container);
}, "intersection");

const container_union = document.querySelector("#container_euler_union");
euler(container_union, set_union, (container, s1, s2, s3) => {
  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, s3);
  const set2 = set_tex(s2, s3);
  const set3 = set_tex(s3, s3);

  return tex(`${s1_name} ${union} ${s2_name} ${seq} ${set1} ${union} ${set2} ${seq} ${set3}`, container);
}, "union");

const container_setminus = document.querySelector("#container_euler_setminus");
euler(container_setminus, set_difference, (container, s1, s2, s3) => {
  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, s3);
  const set2 = set_tex(s2, s3);
  const set3 = set_tex(s3, s3);

  return tex(`${s1_name} ${setminus} ${s2_name} ${seq} ${set1} ${setminus} ${set2} ${seq} ${set3}`, container);
}, "setminus");

function polar_to_cartesian([x, y], r, t) {
  return [r * Math.cos(t) + x, r * Math.sin(t) + y];
}

// let [xfoo, yfoo] = polar_to_cartesian([0, 0], 20, (PI + PI * 1.5) + ((2 * PI * 2) / 3));
// console.log(`x="${xfoo - 15}" y="${yfoo - 15}"`);

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
  const path = document.createElementNS(svgns, "path");
  path.setAttribute("class", clazz);
  return path;
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

function random_bin_tree(target_inner, gen_leaf, gen_inner, state) {
  if (target_inner === 0) {
    return gen_leaf(state);
  } else {
    const left = random_int(target_inner, state);
    return {
      "inner": [
        random_bin_tree(left, gen_leaf, gen_inner, state),
        gen_inner(state),
        random_bin_tree(target_inner - (left + 1), gen_leaf, gen_inner, state),
      ],
    };
  }
}

function random_int(max_exclusive) {
  return Math.floor(Math.random() * max_exclusive);
}

function coin_flip() {
  return Math.random() < 0.5;
}

function random_set_5(state) {
  return [coin_flip(state), coin_flip(state), coin_flip(state), coin_flip(state), coin_flip(state)];
}

function random_from_array(arr, state) {
  return arr[random_int(arr.length, state)];
}

function dfs(inner_pre, inner_post, leaf, node, parent_pre) {
  if (node["inner"]) {
    const pre = inner_pre ? inner_pre(node.inner[1], parent_pre) : undefined;
    const left = dfs(inner_pre, inner_post, leaf, node.inner[0], pre);
    const right = dfs(inner_pre, inner_post, leaf, node.inner[2], pre);
    return inner_post ? inner_post(node.inner[0], node.inner[1], node.inner[2], left, pre, right) : undefined;
  } else {
    return leaf ? leaf(node) : node;
  }
}

function eval_op(op, left, right) {
  switch (op) {
    case "intersection": return set_intersection(left, right);
    case "union": return set_union(left, right);
    case "difference": return set_difference(left, right);
    case "symmetric_difference": return set_symmetric_difference(left, right);

    default: throw "unknown operator";
  }
}

function eval_node(node) {
  return dfs(null, (_l, op, _r, l, _p, r) => {return eval_op(op, l, r)}, null, node);
}

function has_all_operators(node, ops) {
  const found_ops = {};

  dfs(op => {found_ops[op] = true}, null, null, node);

  let all = true;
  ops.forEach(op => {
    all = all && !!found_ops[op];
  });

  return all;
}

function has_two_differences(node) {
  let differences = 0;

  dfs(op => {
    if (op === "difference") {
      differences += 1;
    }
  }, null, null, node);

  return differences >= 2;
}

function height(node) {
  return dfs(null, (_l, _op, _r, l, _p, r) => {
    return 1 + Math.max(l, r);
  }, () => 0, node);
}

function leaves(node) {
  let n = 0;
  dfs(null, null, () => {n += 1;}, node);
  return n;
}

function eval_one_step(node) {
  return dfs(null, (lnode, op, rnode, l, _p, r) => {
    if (!lnode["inner"] && !rnode["inner"]) {
      return eval_op(op, l, r);
    } else {
      return {
        "inner": [l, op, r],
      };
    }
  }, null, node);
}

function practice_intersection_union_tree() {
  while (true) {
    const expr = random_bin_tree(3, random_set_5, () => {return random_from_array(["intersection", "union"]);});
    if (!has_all_operators(expr, ["intersection", "union"])) {
      continue
    }

    let interesting = true;

    dfs(null, (_l, op, _r, left, _p, right) => {
      const result = eval_op(op, left, right);
      if (set_eq(result, left) && Math.random() <= 0.7) {
        interesting = false;
      }
      if (set_eq(result, right) && Math.random() <= 0.7) {
        interesting = false;
      }
      return result;
    }, null, expr);

    if (interesting) {
      return expr;
    }
  }
}

function practice_set_difference_tree() {
  while (true) {
    const expr = random_bin_tree(3, random_set_5, () => {return random_from_array(["intersection", "union", "difference"]);});
    if (!has_two_differences(expr)) {
      continue
    }

    let interesting = true;

    dfs(null, (_l, op, _r, left, _p, right) => {
      const result = eval_op(op, left, right);
      if (set_eq(result, left) && Math.random() <= 0.7) {
        interesting = false;
      }
      if (set_eq(result, right) && Math.random() <= 0.7) {
        interesting = false;
      }
      if (set_eq(result, empty_s()) && Math.random() <= 0.8) {
        interesting = false;
      }
      return result;
    }, null, expr);

    if (interesting) {
      return expr;
    }
  }
}

function tex_op(op) {
  switch (op) {
    case "intersection": return intersection;
    case "union": return union;
    case "difference": return setminus;
    case "symmetric_difference": return "TODO";

    default: throw "unknown operator";
  }
}

function expr_to_tex(expr, not_a_set) {
  const h = height(expr);
  return dfs((_, level) => {return level + 1;}, (lnode, op, rnode, left, pre, right) => {
    if (pre === -1) {
      return `${left} ${tex_op(op)} ${right}`;
    } else {
      return p(`${left} ${tex_op(op)} ${right}`, h - (pre + 1));
    }
  }, not_a_set ? x => x : set_tex_vanilla, expr, -2);
}

function expr_to_solution_tex(expr_) {
  let expr = expr_;
  const lines = [`&${expr_to_tex(expr)}\\\\`];

  while (true) {
    expr = eval_one_step(expr);
    lines.push(`${seq} {} &${expr_to_tex(expr)}\\\\`);
    if (!expr["inner"]) {
      break
    }
  }

  return `\\begin{align*}${lines.join("\n")}\\end{align*}`;
}

const practice_intersection_union_direct_text = document.querySelector("#practice_intersection_union_direct_text");
const practice_intersection_union_direct_solution = document.querySelector("#practice_intersection_union_direct_solution");
const practice_intersection_union_direct_new = document.querySelector("#practice_intersection_union_direct_new");

function new_practice_intersection_union_direct() {
  const expr = practice_intersection_union_tree();
  tex(`${expr_to_tex(expr)}.`, practice_intersection_union_direct_text);
  tex(`${expr_to_solution_tex(expr)}`, practice_intersection_union_direct_solution, {displayMode: true});

  const btn_toggle_practice_intersection_union_direct = document.querySelector("#btn_toggle_practice_intersection_union_direct");
  if (btn_toggle_practice_intersection_union_direct.classList.contains("yes")) {
    btn_toggle_practice_intersection_union_direct.click()
  }
}

new_practice_intersection_union_direct();
practice_intersection_union_direct_new.addEventListener("click", new_practice_intersection_union_direct);



const practice_set_difference_direct_text = document.querySelector("#practice_set_difference_direct_text");
const practice_set_difference_direct_solution = document.querySelector("#practice_set_difference_direct_solution");
const practice_set_difference_direct_new = document.querySelector("#practice_set_difference_direct_new");

function new_practice_set_difference_direct() {
  const expr = practice_set_difference_tree();
  tex(`${expr_to_tex(expr)}.`, practice_set_difference_direct_text);
  tex(`${expr_to_solution_tex(expr)}`, practice_set_difference_direct_solution, {displayMode: true});

  const btn_toggle_practice_set_difference_direct = document.querySelector("#btn_toggle_practice_set_difference_direct");
  if (btn_toggle_practice_set_difference_direct.classList.contains("yes")) {
    btn_toggle_practice_set_difference_direct.click()
  }
}

new_practice_set_difference_direct();
practice_set_difference_direct_new.addEventListener("click", new_practice_set_difference_direct);

const exercise_arbitrary_venn_text = document.querySelector("#exercise_arbitrary_venn_text");
const exercise_arbitrary_venn_solution = document.querySelector("#exercise_arbitrary_venn_solution");
const exercise_arbitrary_venn_new = document.querySelector("#exercise_arbitrary_venn_new");

const exercise_arbitrary_venn_sections = [];
for (let i = 0; i < 7; i++) {
  exercise_arbitrary_venn_sections.push(document.querySelector(`#arbitrary_venn${i}`));
}

const arbitrary_venn_solutions = [[120,2,120],[27,2,126],[46,0,3],[27,2,120],[46,2,123],[63,2,122],[46,2,120],[63,2,120],[120,0,10],[27,2,86],[46,0,27],[27,2,80],[46,2,99],[125,2,112],[46,2,96],[63,2,112],[120,0,17],[27,2,46],[126,0,19],[27,2,40],[126,2,106],[63,2,42],[126,2,104],[63,2,40],[120,0,27],[27,2,6],[126,0,27],null,[124,2,96],[31,2,2],[126,2,96],[27,1,6],[120,0,36],[123,2,90],[123,0,38],[123,2,88],[46,2,27],[63,2,26],[46,2,24],[63,2,24],[120,0,46],[121,2,80],[123,0,46],[123,2,80],[46,2,3],[47,2,2],null,[46,1,3],[120,0,53],[63,2,14],[126,0,51],[59,2,8],[63,2,11],[63,2,10],[62,2,8],[63,2,8],[120,0,63],[63,2,6],[126,0,59],[40,1,27],[63,2,3],[63,2,2],[46,1,24],[46,1,27],[120,2,63],[123,2,62],[126,2,60],[123,2,56],[126,2,59],[127,2,58],[126,2,56],[127,2,56],[120,2,53],[123,2,54],[110,2,36],[91,2,16],[126,2,51],[125,2,48],[110,2,32],[127,2,48],[120,2,46],[123,2,46],[126,2,44],[123,2,40],[126,2,42],[127,2,42],[126,2,40],[127,2,40],[120,2,36],[123,2,38],[126,2,36],[80,1,27],[124,2,32],[127,2,34],[126,2,32],[91,1,6],[120,2,27],[123,2,26],[123,2,25],[123,2,24],[126,2,27],[127,2,26],[126,2,24],[127,2,24],[120,2,17],[121,2,16],[123,2,17],[123,2,16],[126,2,19],[127,2,18],[96,1,46],[110,1,3],[120,2,10],[123,2,10],[122,2,8],[123,2,8],[126,2,10],[127,2,10],[126,2,8],[127,2,8],null,[120,1,17],[120,1,10],[120,1,27],[120,1,36],[124,1,17],[120,1,46],[126,1,27]];

function arbitrary_venn_index_to_term(i) {
  if (i === 27) {
    return "C";
  } else if (i === 46) {
    return "B";
  } else if (i === 120) {
    return "A";
  } else {
    const op = arbitrary_venn_solutions[i][1];
    return {
      "inner": [
        arbitrary_venn_index_to_term(arbitrary_venn_solutions[i][0]),
        op === 0 ? "intersection" : (op === 1 ? "union" : "difference"),
        arbitrary_venn_index_to_term(arbitrary_venn_solutions[i][2]),
      ],
    };
  }
}

function new_arbitrary_venn() {
  while (true) {
    const i = random_int(128);
    const expr = arbitrary_venn_index_to_term(i);
    const size = leaves(expr);

    if (size >= 4) {
      exercise_arbitrary_venn_text.textContent = `${size - 1}`;
      tex(`${expr_to_tex(expr, true)}.`, exercise_arbitrary_venn_solution);

      const btn_toggle_exercise_arbitrary_venn = document.querySelector("#btn_toggle_exercise_arbitrary_venn");
      if (btn_toggle_exercise_arbitrary_venn.classList.contains("yes")) {
        btn_toggle_exercise_arbitrary_venn.click()
      }

      for (let j = 0; j < 7; j++) {
        exercise_arbitrary_venn_sections[j].classList.toggle("venn_yay", (i & (1 << j)) != 0);
      }

      return;
    }
  }
}

new_arbitrary_venn();
exercise_arbitrary_venn_new.addEventListener("click", new_arbitrary_venn);
