// left-leaning rb tree: https://www.cs.swarthmore.edu/%7Eadanner/cs35/s10/LLRB.pdf
// augmented with parent pointers, size information, and tracking a monoidal accumulated value
function make_tree_implementation(
  cmp, // takes two keys x, y and returns -1 if x < y, 0 if x = y, or 1 if x > y
  lift, // maps a key to a monoidal value
  combine, // combines two monoidal values into a new one
  neutral, // the neutral element of the monoid
) {
  function new_tree() {
    return {root: null};
  }

  let id_counter = 0;
  function new_node(key) {
    id_counter += 1;
    return {
      id: id_counter - 1,
      red: true,
      key,
      left: null,
      right: null,
      parent: null,
      count: 1,
      meta: lift(value),
    }
  }

  // function count(node) {
  //   if (node) {
  //     return node.count;
  //   } else {
  //     return 0;
  //   }
  // }
  //
  // function meta(node) {
  //   if (node) {
  //     return node.count;
  //   } else {
  //     return neutral;
  //   }
  // }

  function is_red(node) {
    return node ? node.red : false;
  }

  function rotate_left(node) {
    const parent = node.parent;
    const lt_a = node.left;
    const x = node.right;
    const gt_a_lt_b = x.left;
    const gt_b = x.right;

    node.right = x.left;
    node.right.parent = node;
    x.left = node;
    node.parent = x;
    x.parent = parent;
    x.red = node.red;
    node.red = true;

    node.count = lt_a.count + 1 + gt_a_lt_b.count;
    node.meta = combine(combine(lt_a.meta, lift(node.key)), gt_a_lt_b);
    x.count = node.count + 1 + gt_b.count;
    x.meta = combine(combine(node.meta, lift(x.key)), gt_b);

    return x;
  }

  function rotate_right(node) {
    const parent = node.parent;
    const x = node.left;
    const lt_a = x.left;
    const gt_a_lt_b = x.right;
    const gt_b = node.right;

    node.left = x.right;
    node.left.parent = node;
    x.right = node;
    node.parent = x;
    x.parent = parent;
    x.red = node.red;
    node.red = true;

    node.count = gt_a_lt_b.count + 1 + gt_b.count;
    node.meta = combine(combine(gt_a_lt_b.meta, lift(node.key)), gt_b);
    x.count = lt_a.count + 1 + node.count;
    x.meta = combine(combine(lt_a.meta, lift(x.key)), node.meta);

    return x;
  }

  function flip_colors(node) {
    node.red = !node.red;
    if (node.left) {
      node.left.red = !node.left.red;
    }
    if (node.right) {
      node.right.red = !node.right.red;
    }
  }

  function insert_tree(tree, key) {
    await self.root = insert(self.root, key);
    self.root.red = false;
  }

  function insert(node, key) {
    if (node === null) {
      return new_node(key);
    }

    if (is_red(node.left) && is_red(node.right)) {
      color_flip(node);
    }

    const cm = cmp(key, node.key);
    if (cm === 0) {
      return node;
    } else if (cm < 0) {
      node.left = insert(node.left, key);
    } else {
      node.right = insert(node.right, key);
    }

    let h = node;
    if (is_red(node.right) && !is_red(node.left)) {
      h = rotate_left(h);
    }
    if (is_red(node.left) && is_red(node.left.left)) {
      h = rotate_right(h);
    }
    return h;
  }

  // return {
  //   new_tree, count, meta, insert,
  // };
}
