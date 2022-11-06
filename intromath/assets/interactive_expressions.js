import { tex } from "./tex.js";
import { animate, ease_in_out_cubic } from "./animation.js";
import { register_tooltip_handler } from "./tooltips.js";
import { lerp, angle_to_x, distance } from "./geometry.js";
import { dfs, height, is_rightmost_child, is_root, leftmost_child, push_child, rightmost_child, set_children, tree_for_each } from "./trees.js";
import { expr_to_tex, new_expr } from "./expressions.js";

// id: String // prefix for dom ids unique to this interactive expression
// ex_: Expression (from ./expressions.js) // the initial value of this interactive expression
// opts: {
//     editable: Option<([Node], [Node], [Node])>, // not editable if undefined, else contains the possible `node` values of arities 0, 1, 2 respectively
//     render_tex: DomNode, // renders tex into this, or does not render tex if falsey
//     render_node_to_tex, // see the expression.js expr_to_tex
//     render_tex_opts: {}, // same options as in expressions.js expr_to_tex 
//     render_hierarchy: DomNode, // renders a tree drawing into this, or does not render a tree if falsey
//     render_node_label_tex: Node -> String, // Render node label as tex
//     render_hierarchy_opts: {
//         layout: {
//             minsep: f64, // minimum x distance between two nodes in em units
//             y_factor: f64, // distance between y layers in em units
//             min_height: Option<f64>,
//             max_height: Option<f64>,
//         },
//         animation_duration: f64, // duration of x positioning in ms units
//         magnify: Option<f64>, // set font size to this many rem units, default 1
//     },
// }
export function new_interactive_expression(id, ex_, opts) {
    let ex = ex_;

    let node_no = 0;
    dfs(
        ex => {
            ex.interactive_expression_no = node_no;
            node_no += 1;
        },
        null,
        ex,
    );

    if (opts.render_tex) {
        if (!opts.render_tex_opts) {
            opts.render_tex_opts = {};
        }
        opts.render_tex_opts.highlight_steps = false;
        opts.render_tex_opts.interactive_id = id;

        opts.render_tex.classList.add("interactive_expression_tex");

        opts.render_tex.addEventListener("mouseover", evt => {
            const iex = evt.target.parentNode.interactive_expression;
            if (iex) {
                on_mouseenter(iex);
                evt.target.addEventListener("mouseleave", () => on_mouseleave(iex), { once: true });
            }
        });

        render_tex();
    }

    const container_drawing = document.createElement("div");
    const drawing_edges = document.createElement("div");
    container_drawing.appendChild(drawing_edges);
    const drawing_vertices = document.createElement("div");
    container_drawing.appendChild(drawing_vertices);

    if (opts.render_hierarchy) {
        ex.x = 0;

        opts.render_hierarchy.appendChild(container_drawing);
        opts.render_hierarchy.style.fontSize = `${opts.render_hierarchy_opts.magnify ? opts.render_hierarchy_opts.magnify : 1}rem`;

        tree_for_each(ex, init_drawing_of_node);

        determine_threads(ex);
        determine_x_offset(ex, opts);
        petrify(ex, opts);
        compute_drawing_width(ex);
        update_dimensions(ex, opts);
    }

    function init_drawing_of_node(t) {
        const drawing = document.createElement("span");
        drawing.classList.add("induction_tree_node");
        drawing.interactive_expression = t;
        drawing.id = `${id}_hierarchy_${t.interactive_expression_no}`;

        if (is_root(t)) {
          drawing.classList.add("root");
        } else {
          const drawing_edge = document.createElement("div");
          drawing_edge.classList.add("induction_edge");
          t.drawing_edge = drawing_edge;
          drawing_edges.appendChild(drawing_edge);
        }

        render_node_label(drawing, t.node);
        t.drawing = drawing;

        drawing_vertices.appendChild(drawing);

        drawing.addEventListener("mouseover", () => {
            on_mouseenter(t);
            drawing.addEventListener("mouseleave", () => on_mouseleave(t), { once: true });
        });

        if (opts.editable) {
            drawing.menu = make_render_menu(t);
        }
    }

    function render_node_label(elem, node) {
        tex(opts.render_node_label_tex(node), elem);
    }

    function make_render_menu(t) {
        return remove_tooltip => {
            const menu = document.createElement("div");
            menu.classList.add("induction_menu");

            for (let i = 0; i < opts.editable.length; i++) {
                for (let j = 0; j < opts.editable[i].length; j++) {
                    const case_btn = document.createElement("button");
                    render_node_label(case_btn, opts.editable[i][j]);
                    case_btn.addEventListener("click", n => {
                        modify_logical_tree(t, opts.editable[i][j], i);
                        remove_tooltip();
                    });
                    menu.appendChild(case_btn);
                }
            }
            
            return menu;
        };
    }

    function modify_logical_tree(t, node, arity) {
        t.node = node;

        t.children.forEach(c => tree_for_each(c, descendent => {
            descendent.drawing.remove();
            descendent.drawing_edge.remove();
        }));

        set_children(t, []);
        for (let i = 0; i < arity; i++) {
            const child = new_expr(opts.editable[0][0]);            
            push_child(t, child);

            child.interactive_expression_no = node_no;
            node_no += 1;

            if (opts.render_hierarchy) {
                init_drawing_of_node(child);
            }            
        }

        if (opts.render_tex) {
            render_tex();
        }

        if (opts.render_hierarchy) {
            render_node_label(t.drawing, node);

            determine_threads(ex);
            determine_x_offset(ex, opts);
            petrify(ex, opts);
            compute_drawing_width(ex);
            update_dimensions(ex, opts);
        }
    }

    function render_tex() {
        tex(expr_to_tex(
            ex,
            opts.render_node_to_tex,
            opts.render_tex_opts,
        ), opts.render_tex);

        dfs(
            ex => {
                const node = document.getElementById(`${id}_tex_${ex.interactive_expression_no}`);
                node.interactive_expression = ex;
                node.childNodes[0].menu = make_render_menu(ex);
            },
            null,
            ex,
        );
    }

    function on_mouseenter(iex) {
        if (opts.render_tex) {
            const node_tex = document.getElementById(`${id}_tex_${iex.interactive_expression_no}`);
            const expression_tex = document.getElementById(`${id}_tex_subexpression_${iex.interactive_expression_no}`);
            add_classes(highlighting_classes(iex, "low"), expression_tex);
            add_classes(highlighting_classes(iex, "full"), node_tex);
        }

        if (opts.render_hierarchy) {
            const node_vertex = document.getElementById(`${id}_hierarchy_${iex.interactive_expression_no}`);
            tree_for_each(iex, t => t.drawing.style.backgroundColor = "var(--color-bg3)");
            tree_for_each(iex, t => {
                if (t != iex) {
                    t.drawing_edge.style.backgroundColor = "var(--color-bg3)";
                }
            });
        }
    }

    function on_mouseleave(iex) {
        if (opts.render_tex) {
            const node_tex = document.getElementById(`${id}_tex_${iex.interactive_expression_no}`);
            const expression_tex = document.getElementById(`${id}_tex_subexpression_${iex.interactive_expression_no}`);
            remove_classes(highlighting_classes(iex, "low"), expression_tex);
            remove_classes(highlighting_classes(iex, "full"), node_tex);
        }

        if (opts.render_hierarchy) {
            const node_vertex = document.getElementById(`${id}_hierarchy_${iex.interactive_expression_no}`);
            tree_for_each(iex, t => t.drawing.style.backgroundColor = "");
            tree_for_each(iex, t => {
                if (t != iex) {
                    t.drawing_edge.style.backgroundColor = "var(--color-bg2)"
                }
            });
   ;     }
    }
}

function highlighting_classes(ex, area) {
    const classes = ["bgcbg3", "highlight", area];
    // Add this class if the glyph(s) to highlight have whitespace to their right.
    if (ex.children.length != 1 && (ex.children.length === 2 || /*leaf*/ (!is_rightmost_child(ex) && !is_root(ex)))) {
        classes.push("rightspace");
    }
    return classes;
}

function add_classes(classes, elem) {
    classes.forEach(c => elem.classList.add(c));
}

function remove_classes(classes, elem) {
    classes.forEach(c => elem.classList.remove(c));
}

function determine_threads(t) {
    const l = leftmost_child(t);
    const r = rightmost_child(t);

    if (l === null) {
        return [null, null];
    }

    const [_ll, lr] = determine_threads(l);
    const [rl, _rr] = determine_threads(r);

    if (lr === null && l != null) {
        l.leftmost_thread = rl;
    }
    if (rl === null && r != null) {
        r.rightmost_thread = lr;
    }

    return [l, r];
}

function determine_x_offset(t, opts) {
    const layout = opts.render_hierarchy_opts.layout;

    const l = leftmost_child(t);
    const r = rightmost_child(t);
    if (l === null) {
        t.x_offset = 0;
        return;
    }

    determine_x_offset(l, opts);
    determine_x_offset(r, opts);

    let distance = 0;
    if (l != r) {
        distance = layout.minsep;
        let distance_at_current_depth = distance;

        let lp = l;
        let l_ = rightmost_child_or_thread(l);
        let rp = r;
        let r_ = leftmost_child_or_thread(r);

        while (l_ != null && r_ != null && l_ != r_) {
            let l_offset = l_.x_offset;
            let l_predecessor = l_.parent;
            let l_least_common_ancestor = lp;
            while (l_predecessor != l_least_common_ancestor) {
                l_offset += l_predecessor.x_offset;
                l_predecessor = l_predecessor.parent;
                l_offset -= l_least_common_ancestor.x_offset;
                l_least_common_ancestor = l_least_common_ancestor.parent;
            }
            distance_at_current_depth -= l_offset;

            let r_offset = r_.x_offset;
            let r_predecessor = r_.parent;
            let r_least_common_ancestor = rp;
            while (r_predecessor != r_least_common_ancestor) {
                r_offset += r_predecessor.x_offset;
                r_predecessor = r_predecessor.parent;
                r_offset -= r_least_common_ancestor.x_offset;
                r_least_common_ancestor = r_least_common_ancestor.parent;
            }
            distance_at_current_depth += r_offset;

            distance += Math.max(0, layout.minsep - distance_at_current_depth);
            distance_at_current_depth = Math.max(distance_at_current_depth, layout.minsep);

            lp = l_;
            l_ = rightmost_child_or_thread(l_);
            rp = r_;
            r_ = leftmost_child_or_thread(r_);
        }
    }

    l.x_offset = distance / -2;
    r.x_offset = distance / 2;
}

function leftmost_child_or_thread(t) {
    if (t === null) {
        return null;
    } else {
        if (t.children.length > 0) {
            return t.children[0];
        } else {
            return t.leftmost_thread ? t.leftmost_thread : null;
        }
    }
}

function rightmost_child_or_thread(t) {
    if (t === null) {
        return null;
    } else {
        if (t.children.length > 0) {
            return t.children[t.children.length - 1];
        } else {
            return t.rightmost_thread ? t.rightmost_thread : null;
        }
    }
}

function petrify(t, opts) {
    petrify_(t, 0, 0, opts);
}

function petrify_(t, x, y, opts) {
    const x_ = x + (t.x_offset ? t.x_offset : 0);

    t.old_x = (t.x || (t.x === 0)) ? t.x : t.parent.old_x + (t.x_offset ? t.x_offset : 0);

    t.x = x_;
    animate(
        t.drawing,
        (_drawing, time) => {
            const factor = ease_in_out_cubic(time);
            const node_x = lerp(t.old_x, t.x, factor);
            const node_y = y * opts.render_hierarchy_opts.layout.y_factor;
            t.drawing_x = node_x;
            t.drawing_y = node_y;

            t.drawing.style.transform = `translate(${node_x + t.drawing_width/2}em, ${node_y}em)`;

            if (t.parent) {
                const edge_line = [[node_x + t.drawing_width/2, node_y], [t.parent.drawing_x + t.drawing_width/2, t.parent.drawing_y]];
                t.drawing_edge.style.width = `${distance(edge_line[0], edge_line[1])}em`;
                t.drawing_edge.style.transform = `translate(${node_x + 0.75 + t.drawing_width/2}em, ${node_y + 0.75 - 0.125}em) rotate(${angle_to_x(edge_line)}rad)`;
            }
        },
        opts.render_hierarchy_opts.animation_duration,
    );

    for (const child of t.children) {
        petrify_(child, x_, y + 1, opts);
    }
}

function compute_drawing_width(ex) {
    let min = 0;
    let max = 0;

    tree_for_each(ex, t => {
        min = Math.min(min, t.x);
        max = Math.max(max, t.x);
    });

    tree_for_each(ex, t => t.drawing_width = -min + max + 3);
}

function update_dimensions(ex, opts) {
    const layout = opts.render_hierarchy_opts.layout;
    
    const h = height(ex);
    let drawing_height = (layout.y_factor) * h + 3;
    drawing_height = Math.max(drawing_height, layout.min_height ? layout.min_height : 0);
    drawing_height = Math.min(drawing_height, layout.max_height ? layout.max_height : drawing_height);
    opts.render_hierarchy.style.height = `${drawing_height}em`;

    opts.render_hierarchy.style.width = `${ex.drawing_width}em`;
}

register_tooltip_handler({
    selector: elem => elem.menu,
    start_delay: 200,
    render: (menu, _evt, remove_tooltip) => {
        return menu(remove_tooltip);
    },
});
