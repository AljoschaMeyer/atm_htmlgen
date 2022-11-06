import { reduce_motion } from "./accessibility.js";

import { animate, ease_in_out_cubic } from "./animation.js";

import { tex, tex_string, defeq, set, seq, sneq, subseteq, subset, supseteq, supset, nsubseteq, nsubset, nsupseteq, nsupset, intersection, union, setminus, powerset, p, highlight, highlight_raw, symbol as tex_symbol } from "./tex.js";

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
    convex_path_string,
} from "./svg.js";

import {
    random_bool, random_int, random_from_array, gen_interesting
} from "./random.js";

import {
    dfs, is_leaf, push_child, count_leaves,
} from "./trees.js";

import {
    count_node_kind,
    expr_to_solution_tex,
    expr_to_tex,
    has_all_operators,
    new_expr
} from "./expressions.js";

////////////////////
// Euler Diagrams //
////////////////////

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

const container_vanilla = document.querySelector("#container_euler_vanilla");
euler(container_vanilla, () => [false, false, false, false, false], () => { });

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
\\\\\\big\\}`, results, { display: true, fleqn: true });
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

//////////////////////////
// Generative exercises //
//////////////////////////

function random_bin_tree(target_inner, gen_leaf, gen_inner, state) {
    if (target_inner === 0) {
        return gen_leaf(state);
    } else {
        const left = random_int(target_inner, state);
        const t = new_expr(gen_inner(state));
        push_child(t, random_bin_tree(left, gen_leaf, gen_inner, state));
        push_child(t, random_bin_tree(target_inner - (left + 1), gen_leaf, gen_inner, state));
        return t;
    }
}

function random_set_5(state) {
    return [random_bool(state), random_bool(state), random_bool(state), random_bool(state), random_bool(state)];
}

function random_leaf() {
    return new_expr(random_set_5());
}

function set_ops(node, cs) {
    switch (node) {
        case "intersection": return bitvec_and(...cs);
        case "union": return bitvec_or(...cs);
        case "difference": return bitvec_without(...cs);
        case "symmetric_difference": return bitvec_xor(...cs);
    }
}

function tex_op(ex) {
    switch (ex.node) {
        case "intersection":
            return intersection;
        case "union":
            return union;
        case "difference":
            return setminus;
        default: throw "unknown operator";
    }
}

function render_node_to_tex(node, rendered_children) {
    if (rendered_children.length === 0) {
        if (Array.isArray(node)) {
            return set_tex_vanilla(node);
        } else {
            return node;
        }
    } else {
        return tex_op(node, rendered_children);
    }
}

function associative_op(op) {
    return (op === "intersection") || (op === "union") || (op === "symmetric_difference");
}

function has_two_differences(t) {
    return count_node_kind(t, "difference") >= 2;
}

function practice_intersection_union_tree() {
    return gen_interesting(
        () => random_bin_tree(3, random_leaf, () => { return random_from_array(["intersection", "union"]); }),
        expr => {
            if (!has_all_operators(expr, ["intersection", "union"])) {
                return false
            }

            let interesting = true;

            dfs(
                null,
                (t, cs) => {
                    if (!is_leaf(t)) {
                        const result = set_ops(t.node, cs);
                        if (bitvec_eq(result, cs[0]) && Math.random() <= 0.7) {
                            interesting = false;
                        }
                        if (bitvec_eq(result, cs[1]) && Math.random() <= 0.7) {
                            interesting = false;
                        }
                        return result;
                    } else {
                        return t.node;
                    }
                },
                expr,
            );

            return interesting;
        },
    );
}

function practice_set_difference_tree() {
    return gen_interesting(
        () => random_bin_tree(3, random_leaf, () => { return random_from_array(["intersection", "union", "difference"]); }),
        expr => {
            if (!has_two_differences(expr)) {
                return false
            }

            let interesting = true;

            dfs(
                null,
                (t, cs) => {
                    if (!is_leaf(t)) {
                        const result = set_ops(t.node, cs);
                        if (bitvec_eq(result, cs[0]) && Math.random() <= 0.7) {
                            interesting = false;
                        }
                        if (bitvec_eq(result, cs[1]) && Math.random() <= 0.7) {
                            interesting = false;
                        }
                        if (bitvec_eq(result, empty_s()) && Math.random() <= 0.8) {
                            interesting = false;
                        }
                        return result;
                    } else {
                        return t.node;
                    }
                },
                expr,
            );

            return interesting;
        }
    );
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
    (ex, tag) => tex(`${expr_to_tex(ex, render_node_to_tex, { base_level: 1 })}.`, tag),
    (ex, tag) => tex(expr_to_solution_tex(ex, render_node_to_tex, set_ops, {
        highlight_steps: true,
        base_level: 1
    }), tag, { displayMode: true }),
);

generated_exercise(
    "practice_set_difference_direct",
    practice_set_difference_tree,
    (ex, tag) => tex(`${expr_to_tex(ex, render_node_to_tex, { base_level: 1 })}.`, tag),
    (ex, tag) => tex(expr_to_solution_tex(ex, render_node_to_tex, set_ops, {
        highlight_steps: true,
        base_level: 1
    }), tag, { displayMode: true }),
);

const exercise_arbitrary_venn_sections = [];
for (let i = 0; i < 7; i++) {
    exercise_arbitrary_venn_sections.push(document.querySelector(`#arbitrary_venn${i}`));
}

const arbitrary_venn_solutions = [[120, 2, 120], [27, 2, 126], [46, 0, 3], [27, 2, 120], [46, 2, 123], [63, 2, 122], [46, 2, 120], [63, 2, 120], [120, 0, 10], [27, 2, 86], [46, 0, 27], [27, 2, 80], [46, 2, 99], [125, 2, 112], [46, 2, 96], [63, 2, 112], [120, 0, 17], [27, 2, 46], [126, 0, 19], [27, 2, 40], [126, 2, 106], [63, 2, 42], [126, 2, 104], [63, 2, 40], [120, 0, 27], [27, 2, 6], [126, 0, 27], null, [124, 2, 96], [31, 2, 2], [126, 2, 96], [27, 1, 6], [120, 0, 36], [123, 2, 90], [123, 0, 38], [123, 2, 88], [46, 2, 27], [63, 2, 26], [46, 2, 24], [63, 2, 24], [120, 0, 46], [121, 2, 80], [123, 0, 46], [123, 2, 80], [46, 2, 3], [47, 2, 2], null, [46, 1, 3], [120, 0, 53], [63, 2, 14], [126, 0, 51], [59, 2, 8], [63, 2, 11], [63, 2, 10], [62, 2, 8], [63, 2, 8], [120, 0, 63], [63, 2, 6], [126, 0, 59], [40, 1, 27], [63, 2, 3], [63, 2, 2], [46, 1, 24], [46, 1, 27], [120, 2, 63], [123, 2, 62], [126, 2, 60], [123, 2, 56], [126, 2, 59], [127, 2, 58], [126, 2, 56], [127, 2, 56], [120, 2, 53], [123, 2, 54], [110, 2, 36], [91, 2, 16], [126, 2, 51], [125, 2, 48], [110, 2, 32], [127, 2, 48], [120, 2, 46], [123, 2, 46], [126, 2, 44], [123, 2, 40], [126, 2, 42], [127, 2, 42], [126, 2, 40], [127, 2, 40], [120, 2, 36], [123, 2, 38], [126, 2, 36], [80, 1, 27], [124, 2, 32], [127, 2, 34], [126, 2, 32], [91, 1, 6], [120, 2, 27], [123, 2, 26], [123, 2, 25], [123, 2, 24], [126, 2, 27], [127, 2, 26], [126, 2, 24], [127, 2, 24], [120, 2, 17], [121, 2, 16], [123, 2, 17], [123, 2, 16], [126, 2, 19], [127, 2, 18], [96, 1, 46], [110, 1, 3], [120, 2, 10], [123, 2, 10], [122, 2, 8], [123, 2, 8], [126, 2, 10], [127, 2, 10], [126, 2, 8], [127, 2, 8], null, [120, 1, 17], [120, 1, 10], [120, 1, 27], [120, 1, 36], [124, 1, 17], [120, 1, 46], [126, 1, 27]];

function arbitrary_venn_index_to_term(i) {
    if (i === 27) {
        return new_expr("C");
    } else if (i === 46) {
        return new_expr("B");
    } else if (i === 120) {
        return new_expr("A");
    } else {
        const op = arbitrary_venn_solutions[i][1];
        const t = new_expr(op === 0 ? "intersection" : (op === 1 ? "union" : "difference"));
        push_child(t, arbitrary_venn_index_to_term(arbitrary_venn_solutions[i][0]));
        push_child(t, arbitrary_venn_index_to_term(arbitrary_venn_solutions[i][2]));
        return t;
    }
}

generated_exercise(
    "exercise_arbitrary_venn",
    (first) => {
        while (true) {
            const i = random_int(128);
            const expr = arbitrary_venn_index_to_term(i);
            const size = count_leaves(expr);

            if ((size >= 4) && ((!first) || (size === 4))) {
                return [i, expr];
            }
        }
    },
    ([i, expr], tag) => {
        const size = count_leaves(expr);
        tag.textContent = `${size - 1}`;

        for (let j = 0; j < 7; j++) {
            exercise_arbitrary_venn_sections[j].classList.toggle("venn_yay", (i & (1 << j)) != 0);
        }
    },
    ([_, expr], tag) => tex(`${expr_to_tex(expr, render_node_to_tex, {
        associativity: associative_op,
    })}.`, tag),
);

///////////
// Duck! //
///////////

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
