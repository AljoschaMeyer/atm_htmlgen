// An expression is a Tree with a `node` field, representing the syntax tree of a mathematic expression.

import { highlight, highlight_raw, seq, p } from "./tex.js";
import { dfs, is_leaf, set_children, is_leftmost_child, is_leaf_as_are_siblings, is_rightmost_child, is_root, new_tree } from "./trees.js";

export function new_expr(node) {
    const t = new_tree();
    t.node = node;
    return t;
}

export function has_all_operators(t, ops) {
    const found_ops = {};

    dfs(t => { found_ops[t.node] = true }, null, t);

    let all = true;
    ops.forEach(op => {
        all = all && !!found_ops[op];
    });

    return all;
}

export function count_node_kind(t, node) {
    let count = 0;

    dfs(
        t => {
            if (t.node === node) {
                count += 1;
            }
        },
        null,
        t,
    );

    return count;
}

// export function eval_expr(ex, ops) {
//     return dfs(
//         null,
//         (ex, cs) => {
//             if (is_leaf(ex)) {
//                 return ex.node;
//             } else {
//                 return ops(ex.node, cs);
//             }
//         },
//         ex,
//     );
// }

export function eval_expr_one_step(t, ops) {
    return dfs(
        null,
        (t, cs) => {
            if (is_leaf(t)) {
                return t;
            } else {
                const r = new_tree();
                if (t.children.reduce((acc, c) => acc && is_leaf(c), true)) {
                    r.node = ops(t.node, cs.map(c => c.node));
                    r.render_step_fresh = true;
                } else {
                    set_children(r, cs);
                    r.node = t.node;
                }
                return r;
            }
        },
        t,
    );
}

function needs_parens(t, opts) {
    if (is_leaf(t) || is_root(t)) {
        return false;
    } else if (is_rightmost_child(t) && (t.children.length === 1)) {
        return false;
    } else if (opts.associativity && (t.node === t.parent.node) && opts.associativity(t.node)) {
        return false;
    } else {
        return true;
    }
}

// render_node_to_tex: (Node, [RenderedChildren]) -> RenderedExpr
// opts: { // all keys are optional
//     highlight_steps: bool, // whether to highlight the last and next possible computation steps
//     base_level: uint, // starting size level for parens
//     associativity: (Node) => bool, // whether the given node is associative. Does not take associativity into account if this key is missing
// }
export function expr_to_tex(ex, render_node_to_tex, opts) {
    return dfs(
        null,
        (t, cs) => {
            if (is_leaf(t)) {
                let expr_tex = render_node_to_tex(t.node, []);

                if (opts.highlight_steps && t.render_step_fresh) {
                    opts.color_top += 1;
                    expr_tex = highlight(opts.color_top, "top", is_leftmost_child(t), expr_tex);
                }

                if (opts.highlight_steps && is_leaf_as_are_siblings(t)) {
                    if (is_leftmost_child(t)) {
                        opts.color_low += 1;
                        expr_tex = `${highlight_raw(opts.color_low, "low", is_rightmost_child(t))}{${expr_tex}`;
                    }
                    if (is_rightmost_child(t)) {
                        expr_tex = `${expr_tex}}`;
                    }
                }

                return {
                    expr_tex,
                    level: 0,
                };
            } else {
                let expr_tex = `${render_node_to_tex(t.node, cs.map(c => c.expr_tex))}`;
                let level = Math.max(...(cs.map(c => c.level)));
                if (needs_parens(t, opts)) {
                    level += 1;
                    expr_tex = p(expr_tex, level - (opts.base_level ? opts.base_level : 0));
                }
                return {
                    expr_tex,
                    level,
                };
            }
        },
        ex,
    ).expr_tex;
}

// args see expr_to_tex
export function expr_to_solution_tex(ex_, render_node_to_tex, ops, opts) {
    let ex = ex_;
    opts.color_top = -1;
    opts.color_low = -1;
    const lines = [`&${expr_to_tex(ex, render_node_to_tex, opts)}\\\\`];

    while (true) {
        ex = eval_expr_one_step(ex, ops);
        lines.push(`${seq} {} &${expr_to_tex(ex, render_node_to_tex, opts)}\\\\`);
        if (is_leaf(ex)) {
            break
        }
    }

    return `\\begin{align*}${lines.join("\n")}\\end{align*}`;
}
