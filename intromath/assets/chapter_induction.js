import {
  make_tree,
} from "./tree_layout.js";

import { new_interactive_expression } from "./interactive_expressions.js";
import { new_expr } from "./expressions.js";
import { is_leaf, push_child } from "./trees.js";

const layout = {
  minsep: 4,
};

const TREE_ANIMATION_DURATION = 400;

make_tree(
  "first_induction_tree",
  [
    {
      tex: true,
      content: "\\top",
      arity: 0,
    },
    {
      tex: true,
      content: "\\bot",
      arity: 0,
    },
    {
      tex: true,
      content: "\\lnot",
      arity: 1,
    },
    {
      tex: true,
      content: "\\land",
      arity: 2,
    },
  ],
  layout,
  (t, children_eval) => {
    function handle(t, bool) {
      t.drawing.classList.add(bool ? "true" : "false");
      t.drawing.classList.remove(!bool ? "true" : "false");
      if (t.drawing_edge) {
        t.drawing_edge.classList.add(bool ? "true" : "false");
        t.drawing_edge.classList.remove(!bool ? "true" : "false");
      }
      return bool;
    }

    if (t.c === 0) {
      return handle(t, true);
    } else if (t.c === 1) {
      return handle(t, false);
    } else if (t.c === 2) {
      return handle(t, !children_eval[0]);
    } else {
      return handle(t, children_eval[0] && children_eval[1]);
    }
  }
);

const t_0 = new_expr(0);
const t_1 = new_expr(1);
const t_2 = new_expr(2);
const t_m = new_expr("+");
push_child(t_m, t_0);
push_child(t_m, t_1);
const ttt_root = new_expr("-");
push_child(ttt_root, t_m);
push_child(ttt_root, t_2);

new_interactive_expression(
  "ttttttt",
  ttt_root,
  {
    render_tex: document.getElementById("bbbbb"),
    render_node_to_tex: (ex) => {
      return `${ex.node}`;
    },
    render_tex_opts: {},
    render_hierarchy: document.getElementById("bbbbbdraw"),
    render_node_label_tex: x => `${x}`,
    render_hierarchy_opts: {
      layout: {
        minsep: 4,
        y_factor: 2.5,
      },
      animation_duration: TREE_ANIMATION_DURATION,
    },
    editable: [[0, 1], [], ["+", "-"]],
  },
);
// opts: {
//     editable: bool,
//     render_tex: DomNode, // renders tex into this, or does not render tex if falsey
//     render_node_to_tex, // see the expression.js expr_to_tex
//     render_tex_opts: {}, // same options as in expressions.js expr_to_tex 
//     render_hierarchy: DomNode, // renders a tree drawing into this, or does not render a tree if falsey
//     render_node_label_tex: Node -> String, // Render node label as tex
//     render_hierarchy_opts: {
//         layout: {
//             minsep: f64, // minimum x distance between two nodes in em units
//             y_factor: f64, // distance between y layers in em units
//         }
//     },