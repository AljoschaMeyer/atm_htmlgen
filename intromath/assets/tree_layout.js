import { tex } from "./tex.js";
import { animate, ease_in_out_cubic } from "./animation.js";
import { register_tooltip_handler } from "./tooltips.js";
import { lerp, angle_to_x, distance } from "./geometry.js";

export function node(children) {
  return {
    children,
  };
}

const TREE_ANIMATION_DURATION = 400;

export function make_tree(id, cases, layout, bottom_up_function) {
  const container = document.querySelector(`#${id}`);

  let node_id = 0;
  const logical_tree = default_leaf(true);
  logical_tree.x = 0; // ?
  let root = logical_tree;

  const container_drawing = document.createElement("div");
  container_drawing.style.transform = "translateX(calc(50% - 0.75em))"
  const drawing_edges = document.createElement("div");
  container_drawing.appendChild(drawing_edges);
  const drawing_vertices = document.createElement("div");
  container_drawing.appendChild(drawing_vertices);

  if (layout) {
    container.appendChild(container_drawing);
    drawing_vertices.appendChild(root.drawing); // ?
  }

  if (bottom_up_function) {
    bottom_up(root, bottom_up_function); // ?
  }

  function modify_logical_tree(c, t) {
    const old_children = t.children;

    const children = [];
    for (let i = 0; i < cases[c].arity; i++) {
      const child = default_leaf(false);
      children.push(child);
    }

    t.children = children;
    t.c = c; // ?

    children.forEach(child => {
      child.parent = t;
    });

    if (layout) {
      function remove_children(t) {
        t.children.forEach(c => {
          c.drawing.remove();
          c.drawing_edge.remove();
          remove_children(c);
        });
      }
      old_children.forEach(old_child => {
        old_child.drawing.remove();
        old_child.drawing_edge.remove();
        remove_children(old_child);
      });

      children.forEach(child => {
        drawing_vertices.appendChild(child.drawing);
      });

      render_case_label(t.drawing, c);

      determine_threads(root);
      determine_x_offset(root);
      petrify(root);
    }

    if (bottom_up_function) {
      bottom_up(root, bottom_up_function);
    }
  }

  // https://www.reingold.co/tidier-drawings.pdf

  function determine_threads(t) {
    const l = leftmost_child(t);
    const r = rightmost_child(t);

    if (l === null) {
      return [null, null];
    }

    const [ll, lr] = determine_threads(l);
    const [rl, rr] = determine_threads(r);

    if (lr === null && l != null) {
      l.leftmost_thread = rl;
    }
    if (rl === null && r != null) {
      r.rightmost_thread = lr;
    }

    return [l, r];
  }

  function leftmost_child(t) {
    if (t === null) {
      return null;
    } else {
      return t.children.length > 0 ? t.children[0] : null;
    }
  }

  function rightmost_child(t) {
    if (t === null) {
      return null;
    } else {
      return t.children.length > 0 ? t.children[t.children.length - 1] : null;
    }
  }

  function determine_x_offset(t) {
    const l = leftmost_child(t);
    const r = rightmost_child(t);
    if (l === null) {
      t.x_offset = 0;
      return;
    }

    determine_x_offset(l);
    determine_x_offset(r);

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

  function petrify(t) {
    petrify_(t, 0, 0);
  }

  function petrify_(t, x, y) {
    const x_ = x + (t.x_offset ? t.x_offset : 0);

    t.old_x = (t.x || (t.x === 0)) ? t.x : t.parent.old_x + (t.x_offset ? t.x_offset : 0);

    t.x = x_;
    animate(
      t.drawing,
      (drawing, time) => {
        const factor = ease_in_out_cubic(time);
        const node_x = lerp(t.old_x, t.x, factor);
        const node_y = y * 2.5;
        t.drawing_x = node_x;
        t.drawing_y = node_y;

        t.drawing.style.transform = `translate(${node_x}em, ${node_y}em)`;

        if (t.parent) {
          const edge_line = [[node_x, node_y], [t.parent.drawing_x, t.parent.drawing_y]];
          t.drawing_edge.style.width = `${distance(edge_line[0], edge_line[1])}em`;
          t.drawing_edge.style.transform = `translate(${node_x + 0.75}em, ${node_y + 0.75 - 0.125}em) rotate(${angle_to_x(edge_line)}rad)`;
        }
      },
      TREE_ANIMATION_DURATION,
    );

    for (const child of t.children) {
      petrify_(child, x_, y + 1);
    }
  }

  function bottom_up(t, f) {
    return f(t, t.children.map(child => bottom_up(child, f)));
  }

  function default_leaf(is_root) {
    let logical_tree = node([]);
    logical_tree.parent = null;
    logical_tree.c = 0;
    logical_tree.id = node_id;
    node_id += 1;

    if (layout) {
      create_default_drawing_node(logical_tree, is_root);
    }

    return logical_tree;
  }

  function create_drawing_node(logical_tree, c, is_root) {
    const drawing = document.createElement("span");
    drawing.induction_tree = [
      logical_tree,
      (remove_tooltip) => {
        const menu = document.createElement("div");
        menu.classList.add("induction_menu");

        for (let i = 0; i < cases.length; i++) {
          const case_btn = document.createElement("button");
          render_case_label(case_btn, i);
          case_btn.addEventListener("click", n => {
            modify_logical_tree(i, logical_tree);
            remove_tooltip();
          });
          menu.appendChild(case_btn);
        }
        return menu;
      },
    ];

    drawing.classList.add("induction_tree_node");
    if (is_root) {
      drawing.classList.add("root");
    } else {
      const drawing_edge = document.createElement("div");
      drawing_edge.classList.add("induction_edge");
      logical_tree.drawing_edge = drawing_edge;
      drawing_edges.appendChild(drawing_edge);
    }

    render_case_label(drawing, c);

    // drawing.children[0].textContent = `${logical_tree.id}`; // TODO remove, tmp for debugging

    for (let i = 0; i < cases[c].arity; i++) {
      create_default_drawing_node(logical_tree.children[i], false)
      drawing_vertices.appendChild(logical_tree.children[i].drawing);
    }

    logical_tree.drawing = drawing;
  }

  function create_default_drawing_node(logical_tree, is_root) {
    return create_drawing_node(logical_tree, 0, is_root);
  }

  function render_case_label(elem, c) {
    if (cases[c].tex) {
      tex(cases[c].content, elem);
    } else {
      elem.textContent = cases[c].content;
    }
  }
}

register_tooltip_handler({
  selector: find_induction_tree,
  start_delay: 200,
  render: ([_logical_tree, make_menu], _evt, remove_tooltip) => {
    return make_menu(remove_tooltip);
  },
});

function find_induction_tree(elem) {
  if (elem.induction_tree) {
    return elem.induction_tree;
  }

  return undefined;
}
