import {
  make_tree,
} from "./tree_layout.js";

const layout = {
  minsep: 4,
};

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
