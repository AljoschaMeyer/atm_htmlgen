// A tree is an object with a `children` array of trees, and a `parent` tree/null.

export function new_tree() {
    return {
        parent: null,
        children: [],
    };
}

export function is_root(t) {
    return t.parent === null;
}

export function is_leaf(t) {
    return t.children.length === 0;
}

export function is_leftmost_child(t) {
    if (is_root(t)) {
        return false;
    } else {
        return t === t.parent.children[0];
    }
}

export function is_rightmost_child(t) {
    if (is_root(t)) {
        return false;
    } else {
        return t === t.parent.children[t.parent.children.length - 1];
    }
}

export function is_leaf_as_are_siblings(t) {
    if (is_root(t)) {
        return is_leaf(t);
    } else {
        return t.parent.children.reduce((acc, c) => acc && is_leaf(c), true);
    }
}

export function leftmost_child(t) {
    if (t === null) {
        return null;
    } else {
        return t.children.length > 0 ? t.children[0] : null;
    }
}

export function rightmost_child(t) {
    if (t === null) {
        return null;
    } else {
        return t.children.length > 0 ? t.children[t.children.length - 1] : null;
    }
}

export function tree_for_each(t, fun, start) {
  dfs(
    fun,
    null,
    t,
    start,
);
}

export function push_child(t, c) {
    t.children.push(c);
    c.parent = t;
}

export function set_children(t, cs) {
    t.children = cs;
    t.children.forEach(c => {c.parent = t});
}

// pre: <T>(current: Tree, parent_pre: T) -> T
// post: <S>(current: Tree, childv: [S], pre_of_current: T)
// t: root of Tree
// prev_: T (parent_pre argument for calling pre(t))
export function dfs(pre, post, t, prev_) {
    const prev = pre ? pre(t, prev_) : undefined;
    const childv = t.children.map(c => dfs(pre, post, c, prev));
    return post ? post(t, childv, prev) : undefined;
}

export function height(t) {
  return dfs(
    null,
    (t, cs) => {
      if (is_leaf(t)) {
        return 0;
      } else {
        return 1 + Math.max(cs[0], cs[1]);
      }
    },
    t,
  );
}

export function count_leaves(t) {
  let n = 0;
  dfs(
    null,
    t => {
      if (is_leaf(t)) {
        n += 1;
      }
    },
    t,
  );
  return n;
}