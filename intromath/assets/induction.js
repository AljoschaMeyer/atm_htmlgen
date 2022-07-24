import {
  make_tree,
} from "./tree_layout.js";

const layout = {
  minsep: 4,
};

make_tree(
  "tree_test",
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
);
