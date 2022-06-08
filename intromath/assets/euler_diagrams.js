import { reduce_motion } from "./accessibility.js";

import { animate, ease_in_out_cubic } from "./animation.js";

import { tex, tex_string, defeq, set, seq, sneq, subseteq, subset, supseteq, supset, nsubseteq, nsubset, nsupseteq, nsupset, intersection, union, setminus, powerset, p, highlight, highlight_raw } from "./tex.js";

import {
  bitvec_singleton,
  bitvec_count,
  bitvec_first,
  bitvec_previous,
  bitvec_next,
  bitvec_without,
  bitvec_and,
  bitvec_or,
  bitvec_xor,
  bitvec_eq,
} from "./bitvec.js";

import {
  convex_path_description,
  PI,
  polar_to_cartesian,
  lerp,
  lerp_poly,
  closest_point_on_line,
} from "./geometry.js";

import {
  convex_path_string
} from "./svg.js";

const svgns = "http://www.w3.org/2000/svg";
const R = 10;
const DIAGRAM_R = 70;
const SET_ANIMATION_DURATION = 700;

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
      render_state(i, true);
    });

    buttons2.children[i].addEventListener("click", () => {
      s2[i] = !s2[i];
      render_state(i, false);
    });
  }

  const clip1 = prefix ? document.querySelector(`#${prefix}_clip1_euler_path`) : null;
  const clip2 = prefix ? document.querySelector(`#${prefix}_clip2_euler_path`) : null;
  const mask1 = prefix ? document.querySelector(`#${prefix}_mask1_euler_path`) : null;
  const mask2 = prefix ? document.querySelector(`#${prefix}_mask2_euler_path`) : null;

  const set1 = container.children[2].children[0];
  const set2 = container.children[2].children[1];

  const results = container.children[3];

  initialize_set_path(p1, s1);
  initialize_set_path(p2, s2);

  function render_state(change, set1_changed) {
    const s3 = compute_s3 ? compute_s3(s1, s2) : empty_s();

    const old1 = s1.map(x => x);
    const old2 = s2.map(x => x);
    if (set1_changed) {
      old1[change] = !s1[change];
      draw_set(old1, s1, p1);

      if (clip1) {
        draw_set(old1, s1, clip1);
      }
      if (mask1) {
        draw_set(old1, s1, mask1);
      }
    } else {
      old2[change] = !s2[change];
      draw_set(old2, s2, p2);

      if (clip2) {
        draw_set(old2, s2, clip2);
      }
      if (mask2) {
        draw_set(old2, s2, mask2);
      }
    }

    for (let i = 0; i < 5; i++) {
      svg_elements[i].classList.toggle("s3", s3[i]);
    }

    for (let i = 0; i < 5; i++) {
      buttons1.children[i].classList.toggle("yes", s1[i]);
      buttons1.children[i].classList.toggle("no", !s1[i]);
      buttons1.children[i].innerHTML = button_text(i, s1[i], 1);
      buttons2.children[i].classList.toggle("yes", s2[i]);
      buttons2.children[i].classList.toggle("no", !s2[i]);
      buttons2.children[i].innerHTML = button_text(i, s2[i], 2);
    }

    set1.innerHTML = render_set_def(s1, 1);
    set2.innerHTML = render_set_def(s2, 2);

    if (render_results) {
      render_results(results, s1, s2, s3);
    }
  }
}

function empty_s() {
  return [false, false, false, false, false];
}

function name_set(set) {
  if (set) {
    return set_tex_class(set, set === 1 ? "A" : "B");
  } else {
    return "A";
  }
}

function set_tex_class(set, tex) {
  return `\\htmlClass{cd${set === 1 ? 1 : (set === 3 ? 6 : 3)}}{${tex}}`;
}

function set_tex_class_bg(set, tex) {
  if (set === 3) {
    return `\\htmlClass{bgmc6}{${tex}}`;
  } else {
    return `\\htmlClass{bgmclll${set === 1 ? 1 : 3}}{${tex}}`;
  }
}

function button_text(element, is_in, set) {
  return tex_string(`${tex_symbol(element)} ${is_in ? "\\in" : "\\notin"} ${name_set(set)}`);
}

function render_set_def(s, set, s3) {
  if (set) {
    return tex_string(`${name_set(set)} ${defeq} ${set_tex_class(set, set_tex_class_bg(set, set_tex(s, s3)))}`);
  } else {
    return tex_string(`A ${defeq} ${set_tex(s, s3)}`);
  }
}

function set_tex(s, s3_, set_n) {
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

  if (set_n) {
    return set_tex_class(set_n, set_tex_class_bg(set_n, set(elements)));
  } else {
    return set(elements);
  }
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

const container_vanilla = document.querySelector("#container_euler_vanilla");
euler(container_vanilla, () => [false, false, false, false, false], () => {});

const container_equality = document.querySelector("#container_euler_equality");
euler(container_equality, bitvec_xor, (container, s1, s2, s3) => {
  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, s3, 1);
  const set2 = set_tex(s2, s3, 2);
  const rel = bitvec_count(s3) === 0 ? seq : sneq;

  return tex(`${s1_name} ${seq} ${set1} ${rel} ${set2} ${seq} ${s2_name}`, container);
});

const container_subseteq = document.querySelector("#container_euler_subseteq");
euler(container_subseteq, () => [false, false, false, false, false], (container, s1, s2, s3) => {
  const empty = empty_s();

  const s1_name = name_set(1);
  const s2_name = name_set(2);
  const set1 = set_tex(s1, empty, 1);
  const set2 = set_tex(s2, empty, 2);
  const set1_2 = set_tex(s1, bitvec_without(s1, s2), 1);
  const set2_2 = set_tex(s2, bitvec_without(s2, s1), 2);

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

function handle_binop(op_name, op_tex, op_bitvec) {
  const container = document.querySelector(`#container_euler_${op_name}`);
  euler(container, op_bitvec, (container, s1, s2, s3) => {
    const s1_name = name_set(1);
    const s2_name = name_set(2);
    const set1 = set_tex_class(1, set_tex_class_bg(1, set_tex(s1)));
    const set2 = set_tex_class(2, set_tex_class_bg(2, set_tex(s2)));
    const set3 = set_tex_class_bg(3, set_tex(s3));

    return tex(`${s1_name} ${op_tex} ${s2_name} ${seq} ${set1} ${op_tex} ${set2} ${seq} ${set3}`, container);
  }, op_name);
}

handle_binop("intersection", intersection, bitvec_and);
handle_binop("union", union, bitvec_or);
handle_binop("setminus", setminus, bitvec_without);

function powerset_color(set_as_int) {
  return `hsl\\(${(set_as_int / 31) * 360}, 100%, 50%\\)`;
}

const container_powerset = document.querySelector("#container_euler_powerset");
(() => {
  const container = container_powerset;
  const s = [false, false, false, true, true];

  const svg = container.children[0];

  const svg_elements = [];
  for (let i = 5; i > 0; i--) {
    svg_elements.push(svg.children[svg.children.length - i]);
  }

  const buttons = container.children[1];
  const buttons1 = buttons.children[0];

  for (let i = 0; i < 5; i++) {
    buttons1.children[i].addEventListener("click", () => {
      s[i] = !s[i];
      render_state(i);
    });
  }

  const set1 = container.children[2].children[0];

  const results = container.children[3];

  function render_state(j) {
    const grew = s[j];
    const car = bitvec_count(s);
    const subs = subsets(s);

    for (let i = 0; i < 5; i++) {
      buttons1.children[i].classList.toggle("yes", s[i]);
      buttons1.children[i].classList.toggle("no", !s[i]);
      buttons1.children[i].innerHTML = button_text(i, s[i]);
    }

    set1.innerHTML = render_set_def(s);

    const tex_prefix = `${powerset("A")} ${seq} `;
    const result_set_texs = [];
    const lines = [];

    const delay_round = reduce_motion ? 0 : 450;
    const delay_set = reduce_motion ? 0 : 250;
    let delay = -delay_round;

    for (const row of subs) {
      lines.push([]);
      delay += delay_round;

      for (const b of row) {
        const set_as_int = (
          (b[0] ? 1 : 0) |
          (b[1] ? 2 : 0) |
          (b[2] ? 4 : 0) |
          (b[3] ? 8 : 0) |
          (b[4] ? 16 : 0)) - 1;

        const t = `\\htmlClass{powerset${set_as_int}}{${set_tex(b)}}`;
        if (car <= 1) {
          result_set_texs.push(t);
        } else {
          lines[lines.length - 1].push(t);
        }

        if (grew && b[j]) {
          setTimeout(() => animate_power_set_grow(b, j, svg.children[set_as_int]), delay);
          delay += delay_set;
        }
        if (!grew) {
          const old = b.map(x => x);
          old[j] = true;
          setTimeout(() => animate_power_set_shrink(old, j, svg.children[((set_as_int + 1) | Math.pow(2, j)) - 1]), delay);
          delay += delay_set;
        }
      }
    }

    if (car <= 1) {
      tex(`${tex_prefix}${set(result_set_texs)}`, results);
    } else {
      tex(`${tex_prefix}\\big\\{\\\\ \\hspace{2em}
  ${lines.map(sets => sets.join(", ")).join(", \\\\ \\hspace{2em}")}
\\\\\\big\\}`, results, {display: true, fleqn: true});
    }

  }
})();

function subsets(b) {
  const unsorted = [];
  for (let i4 = 0; i4 <= b[4] ? 1 : 0; i4++) {
    for (let i3 = 0; i3 <= b[3] ? 1 : 0; i3++) {
      for (let i2 = 0; i2 <= b[2] ? 1 : 0; i2++) {
        for (let i1 = 0; i1 <= b[1] ? 1 : 0; i1++) {
          for (let i0 = 0; i0 <= b[0] ? 1 : 0; i0++) {
            unsorted.push([i0, i1, i2, i3, i4]);
          }
        }
      }
    }
  }

  const sorted = [];
  for (let i = 0; i < 6; i++) {
    const arr = [];
    unsorted.forEach(sb => {
      if (bitvec_count(sb) === i) {
        arr.push(sb);
      }
    });
    if (arr.length > 0) {
      sorted.push(arr);
    }
  }

  return sorted;
}

function animate_power_set_grow(b, i, p) {
  const c_to = bitvec_count(b);
  const r_to = set_margin(c_to);
  const [poly_from, poly_to] = morpheable_polys(bitvec_singleton(i, b.length), b);
  animate(p, make_set_morph(poly_from, 0, poly_to, r_to), SET_ANIMATION_DURATION);
}

function animate_power_set_shrink(b, i, p) {
  const c_from = bitvec_count(b);
  const r_from = set_margin(c_from);
  const [poly_from, poly_to] = morpheable_polys(b, bitvec_singleton(i, b.length));
  animate(p, make_set_morph(poly_from, r_from, poly_to, 0), SET_ANIMATION_DURATION);
}

function set_margin(cardinality) {
  return cardinality === 0 ? 0 : R + (2.5 * cardinality);
}

function initialize_set_path(p, s) {
  const r = set_margin(bitvec_count(s));
  p.atm_poly_r = r;
  p.atm_poly = bitvec_to_poly(s, r);
}

function regular_polygon_point(center, r, i, n) {
  return polar_to_cartesian(center, r, (PI * 1.5) + ((2 * PI * i) / n));
}

function draw_set(s_from, s_to, p) {
  const c_from = bitvec_count(s_from);
  const r_from = set_margin(c_from);
  const c_to = bitvec_count(s_to);
  const r_to = set_margin(c_to);

  let poly_from = null;
  let poly_to = null;

  if (c_from === 0) {
    const poly_from = set_to_poly(s_to);
    const poly_to = set_to_poly(s_to);
    animate(p, make_set_morph(poly_from, r_from, poly_to, r_to), SET_ANIMATION_DURATION);
  } else if (c_to === 0) {
    const poly_from = set_to_poly(s_from);
    const poly_to = set_to_poly(s_from);
    animate(p, make_set_morph(poly_from, r_from, poly_to, r_to), SET_ANIMATION_DURATION);
  } else {
    const [poly_from, poly_to] = morpheable_polys(s_from, s_to);
    animate(p, make_set_morph(poly_from, r_from, poly_to, r_to), SET_ANIMATION_DURATION);
  }
}

function set_to_poly(bs) {
  const poly = [];
  bs.forEach((b, i) => {
    if (b) {
      poly.push(regular_polygon_point([0, 0], DIAGRAM_R, i, bs.length));
    }
  });
  return poly;
}

function morpheable_polys(bs1, bs2) {
  const poly_bs1 = [];
  bs1.forEach((b, i) => {
    const p = regular_polygon_point([0, 0], DIAGRAM_R, i, bs1.length);
    if (b) {
      poly_bs1.push(p);
    } else if (bs2[i]) {
      const previous = regular_polygon_point([0, 0], DIAGRAM_R, bitvec_previous(bs1, i), bs1.length);
      const next = regular_polygon_point([0, 0], DIAGRAM_R, bitvec_next(bs1, i), bs1.length);
      poly_bs1.push(closest_point_on_line([previous, next], p));
    }
  });

  const poly_bs2 = [];
  bs2.forEach((b, i) => {
    const p = regular_polygon_point([0, 0], DIAGRAM_R, i, bs2.length);
    if (b) {
      poly_bs2.push(p);
    } else if (bs1[i]) {
      const previous = regular_polygon_point([0, 0], DIAGRAM_R, bitvec_previous(bs2, i), bs2.length);
      const next = regular_polygon_point([0, 0], DIAGRAM_R, bitvec_next(bs2, i), bs2.length);
      poly_bs2.push(closest_point_on_line([previous, next], p));
    }
  });

  return [poly_bs1, poly_bs2];
}

function bitvec_to_poly(bs, r, center_) {
  const c = bitvec_count(bs);
  if (c === 0) {
    return [];
  } else {
    const center = center_ ? center_ : [0, 0];
    const poly = [];

    const first = bitvec_first(bs);
    if (c === 1) {
      for (let i = 0; i < bs.length; i++) {
        poly.push(regular_polygon_point(center, DIAGRAM_R, first, bs.length));
      }
    } else {
      for (let i = 0; i < bs.length; i++) {
        const p = regular_polygon_point(center, DIAGRAM_R, i, bs.length);
        if (bs[i]) {
          poly.push(p);
        } else {
          const previous = regular_polygon_point(center, DIAGRAM_R, bitvec_previous(bs, i), bs.length);
          const next = regular_polygon_point(center, DIAGRAM_R, bitvec_next(bs, i), bs.length);
          poly.push(closest_point_on_line([previous, next], p));
        }
      }
    }

    return poly;
  }
}

function make_set_morph(poly_from, r_from, poly_to, r_to) {
  const tween = ease_in_out_cubic;
  const poly = (t) => lerp_poly(poly_from, poly_to, tween(t));
  const r = (t) => lerp(r_from, r_to, tween(t));

  return (elem, t) => {
    const path_string = convex_path_string(convex_path_description(poly(t), r(t)));
    elem.setAttribute("d", path_string);
  };
}

function random_bin_tree(target_inner, gen_leaf, gen_inner, state) {
  if (target_inner === 0) {
    return gen_leaf(state);
  } else {
    const left = random_int(target_inner, state);
    return {
      "inner": {
        l: random_bin_tree(left, gen_leaf, gen_inner, state),
        node: gen_inner(state),
        r: random_bin_tree(target_inner - (left + 1), gen_leaf, gen_inner, state),
      },
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
    const pre = inner_pre ? inner_pre(node.inner, parent_pre) : undefined;
    const left = dfs(inner_pre, inner_post, leaf, node.inner.l, pre);
    const right = dfs(inner_pre, inner_post, leaf, node.inner.r, pre);
    return inner_post ? inner_post(node.inner, left, pre, right) : undefined;
  } else {
    return leaf ? leaf(node.leaf) : node;
  }
}

function eval_op(op, left, right) {
  switch (op) {
    case "intersection": return bitvec_and(left, right);
    case "union": return bitvec_or(left, right);
    case "difference": return bitvec_without(left, right);
    case "symmetric_difference": return bitvec_xor(left, right);

    default: throw "unknown operator";
  }
}

function eval_node(node) {
  return dfs(null, (n, l, _p, r) => {return eval_op(n.node, l, r)}, (x) => x.leaf.set, node);
}

function has_all_operators(node, ops) {
  const found_ops = {};

  dfs(n => {found_ops[n.node] = true}, null, null, node);

  let all = true;
  ops.forEach(op => {
    all = all && !!found_ops[op];
  });

  return all;
}

function has_two_differences(node) {
  let differences = 0;

  dfs(n => {
    if (n.node === "difference") {
      differences += 1;
    }
  }, null, null, node);

  return differences >= 2;
}

function height(node) {
  return dfs(null, (_n, l, _p, r) => {
    return 1 + Math.max(l, r);
  }, () => 0, node);
}

function leaves(node) {
  let n = 0;
  dfs(null, null, () => {n += 1;}, node);
  return n;
}

function eval_one_step(node) {
  return dfs(null, (n, l, _p, r) => {
    if (!n.l["inner"] && !n.r["inner"]) {
      return {
        leaf: {
          set: eval_op(n.node, l.leaf.set, r.leaf.set),
          fresh: true,
        }
      };
    } else {
      return {
        "inner": {l, node: n.node, r},
      };
    }
  }, null, node);
}

function random_leaf() {
  return {leaf: {set: random_set_5()}};
}

function practice_intersection_union_tree() {
  while (true) {
    const expr = random_bin_tree(3, random_leaf, () => {return random_from_array(["intersection", "union"]);});
    if (!has_all_operators(expr, ["intersection", "union"])) {
      continue
    }

    let interesting = true;

    dfs(null, (n, l, _p, r) => {
      const result = eval_op(n.node, l, r);
      if (bitvec_eq(result, l) && Math.random() <= 0.7) {
        interesting = false;
      }
      if (bitvec_eq(result, r) && Math.random() <= 0.7) {
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
    const expr = random_bin_tree(3, random_leaf, () => {return random_from_array(["intersection", "union", "difference"]);});
    if (!has_two_differences(expr)) {
      continue
    }

    let interesting = true;

    dfs(null, (n, l, _p, r) => {
      const result = eval_op(n.node, l, r);
      if (bitvec_eq(result, l) && Math.random() <= 0.7) {
        interesting = false;
      }
      if (bitvec_eq(result, r) && Math.random() <= 0.7) {
        interesting = false;
      }
      if (bitvec_eq(result, empty_s()) && Math.random() <= 0.8) {
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

function associative_op(op) {
  return (op === "intersection") || (op === "union") || (op === "symmetric_difference");
}

let color_top = -1;
let color_low = -1;
function expr_to_tex(expr, not_a_set, associative, do_highlight) {
  const h = height(expr);
  let outermost = true;
  return dfs(
    (n, [parent_op, _, _2]) => {
      if (n.l["inner"]) {
        n.l["inner"].is_left = true;
      } else {
        n.l["leaf"].is_left = true;
      }
      if (n.l["leaf"] && n.r["leaf"]) {
        n.l["leaf"].highlight_bot = true;
        n.r["leaf"].highlight_bot = true;
      }
      if (outermost) {
        outermost = false;
        return [n.node, true, associative && (n.node === parent_op) && associative_op(n.node)];
      } else {
        return [n.node, false, associative && (n.node === parent_op) && associative_op(n.node)];
      }
    }, (n, [left_tex, left_level], pre, [right_tex, right_level]) => {
      if (pre[1] || pre[2]) {
        return [`${left_tex} ${tex_op(n.node)} ${right_tex}`, Math.max(left_level, right_level)];
      } else {
        const level = Math.max(left_level, right_level) + 1;
        return [p(`${left_tex} ${tex_op(n.node)} ${right_tex}`, level - (not_a_set ? 1 : 0)), level];
      }
    }, (x) => {
      let set_t = not_a_set ? x.set : set_tex_vanilla(x.set);
      if (do_highlight && x.fresh) {
        color_top += 1;
        set_t = highlight(color_top, "top", x.is_left, set_t);
      }
      if (do_highlight && x.highlight_bot) {
        if (x.is_left) {
          color_low += 1;
          set_t = `${highlight_raw(color_low, "low", x.is_left)}{${set_t}`;
        } else {
          set_t = `${set_t}}`;
        }
      }
      return [set_t, 0];
    },
    expr, [null, null, null]
  )[0];
}

function expr_to_solution_tex(expr_) {
  let expr = expr_;
  color_top = -1;
  color_low = -1;
  const lines = [`&${expr_to_tex(expr, false, false, true)}\\\\`];

  while (true) {
    expr = eval_one_step(expr);
    lines.push(`${seq} {} &${expr_to_tex(expr, false, false, true)}\\\\`);
    if (!expr["inner"]) {
      break
    }
  }

  return `\\begin{align*}${lines.join("\n")}\\end{align*}`;
}

function generated_exercise(id_prefix, new_exercise, new_text, new_solution) {
  const tag_text = document.querySelector(`#${id_prefix}_text`);
  const tag_solution = document.querySelector(`#${id_prefix}_solution`);
  const tag_new = document.querySelector(`#${id_prefix}_new`);
  const btn = document.querySelector(`#btn_toggle_${id_prefix}`);

  function on_click(first) {
    const ex = new_exercise(first);
    new_text(ex, tag_text);
    new_solution(ex, tag_solution);
    if (btn.classList.contains("yes")) {
      btn.click();
    }
  }

  on_click(true);
  tag_new.addEventListener("click", () => on_click(false));
}

generated_exercise(
  "practice_intersection_union_direct",
  practice_intersection_union_tree,
  (ex, tag) => tex(`${expr_to_tex(ex)}.`, tag),
  (ex, tag) => tex(expr_to_solution_tex(ex), tag, {displayMode: true}),
);

generated_exercise(
  "practice_set_difference_direct",
  practice_set_difference_tree,
  (ex, tag) => tex(`${expr_to_tex(ex)}.`, tag),
  (ex, tag) => tex(expr_to_solution_tex(ex), tag, {displayMode: true}),
);

const exercise_arbitrary_venn_sections = [];
for (let i = 0; i < 7; i++) {
  exercise_arbitrary_venn_sections.push(document.querySelector(`#arbitrary_venn${i}`));
}

const arbitrary_venn_solutions = [[120,2,120],[27,2,126],[46,0,3],[27,2,120],[46,2,123],[63,2,122],[46,2,120],[63,2,120],[120,0,10],[27,2,86],[46,0,27],[27,2,80],[46,2,99],[125,2,112],[46,2,96],[63,2,112],[120,0,17],[27,2,46],[126,0,19],[27,2,40],[126,2,106],[63,2,42],[126,2,104],[63,2,40],[120,0,27],[27,2,6],[126,0,27],null,[124,2,96],[31,2,2],[126,2,96],[27,1,6],[120,0,36],[123,2,90],[123,0,38],[123,2,88],[46,2,27],[63,2,26],[46,2,24],[63,2,24],[120,0,46],[121,2,80],[123,0,46],[123,2,80],[46,2,3],[47,2,2],null,[46,1,3],[120,0,53],[63,2,14],[126,0,51],[59,2,8],[63,2,11],[63,2,10],[62,2,8],[63,2,8],[120,0,63],[63,2,6],[126,0,59],[40,1,27],[63,2,3],[63,2,2],[46,1,24],[46,1,27],[120,2,63],[123,2,62],[126,2,60],[123,2,56],[126,2,59],[127,2,58],[126,2,56],[127,2,56],[120,2,53],[123,2,54],[110,2,36],[91,2,16],[126,2,51],[125,2,48],[110,2,32],[127,2,48],[120,2,46],[123,2,46],[126,2,44],[123,2,40],[126,2,42],[127,2,42],[126,2,40],[127,2,40],[120,2,36],[123,2,38],[126,2,36],[80,1,27],[124,2,32],[127,2,34],[126,2,32],[91,1,6],[120,2,27],[123,2,26],[123,2,25],[123,2,24],[126,2,27],[127,2,26],[126,2,24],[127,2,24],[120,2,17],[121,2,16],[123,2,17],[123,2,16],[126,2,19],[127,2,18],[96,1,46],[110,1,3],[120,2,10],[123,2,10],[122,2,8],[123,2,8],[126,2,10],[127,2,10],[126,2,8],[127,2,8],null,[120,1,17],[120,1,10],[120,1,27],[120,1,36],[124,1,17],[120,1,46],[126,1,27]];

function arbitrary_venn_index_to_term(i) {
  if (i === 27) {
    return {leaf: {set: "C"}};
  } else if (i === 46) {
    return {leaf: {set: "B"}};
  } else if (i === 120) {
    return {leaf: {set: "A"}};
  } else {
    const op = arbitrary_venn_solutions[i][1];
    return {
      "inner": {
        l: arbitrary_venn_index_to_term(arbitrary_venn_solutions[i][0]),
        node: op === 0 ? "intersection" : (op === 1 ? "union" : "difference"),
        r: arbitrary_venn_index_to_term(arbitrary_venn_solutions[i][2]),
      },
    };
  }
}

generated_exercise(
  "exercise_arbitrary_venn",
  (first) => {
    while (true) {
      const i = random_int(128);
      const expr = arbitrary_venn_index_to_term(i);
      const size = leaves(expr);

      if ((size >= 4) && ((!first) || (size === 4))) {
        return [i, expr];
      }
    }
  },
  ([i, expr], tag) => {
    const size = leaves(expr);
    tag.textContent = `${size - 1}`;

    for (let j = 0; j < 7; j++) {
      exercise_arbitrary_venn_sections[j].classList.toggle("venn_yay", (i & (1 << j)) != 0);
    }
  },
  ([_, expr], tag) => tex(`${expr_to_tex(expr, true, true)}.`, tag),
);

const duck_container = document.querySelector("#duck_container");
const duck_region = document.querySelector("#duck_region");
duck_region.addEventListener("click", () => {
  if (!reduce_motion) {
    duck_container.classList.toggle("active_duck");
    setTimeout(() => {
      duck_container.classList.toggle("active_duck");
    }, 8000);
  }
});
