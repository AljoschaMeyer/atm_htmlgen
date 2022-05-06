export const PI = Math.PI;
export const EPSILON = 0.0001;

export function eq_point([x1, y1], [x2, y2], epsilon_) {
  const epsilon = epsilon_ ? epsilon_ : EPSILON;
  return (Math.abs(x1 - x2) <= epsilon) && (Math.abs(y1 - y2) <= epsilon);
}

export function polar_to_cartesian([x, y], r, t) {
  return [r * Math.cos(t) + x, r * Math.sin(t) + y];
}

export function angle_to_x([[x1, y1], [x2, y2]]) {
  return Math.atan2(y2 - y1, x2 - x1);
}

export function angle_to_y(line) {
  return angle_to_x(line) + (PI * 0.5);
}

function convex_line_in(from, to, r) {
  const angle = angle_to_x([from, to]) + 3*(PI / 2);
  return polar_to_cartesian(to, r, angle);
}

function convex_line_out(from, to, r) {
  const angle = angle_to_x([from, to]) + -1*(PI / 2);
  return polar_to_cartesian(from, r, angle);
}

export function convex_path_description(poly, r) {
  const ret = [];
  const k = poly.length;


  for (let i = 0; i < k; i++) {
    const previous = poly[(i + (k - 1)) % k];
    const current = poly[i];
    const next = poly[(i + 1) % k];

    let next_other = current;
    for (let j = i; j < i + k; j++) {
      if (!eq_point(current, poly[j % k])) {
        next_other = poly[j % k];
        break;
      }
    }

    let previous_other = current;
    for (let j = i + k; j > i; j--) {
      if (!eq_point(current, poly[j % k])) {
        previous_other = poly[j % k];
        break;
      }
    }

    const angle_in = angle_to_x([previous_other, current]) + 3*(PI / 2);
    const angle_out = angle_to_x([current, next_other]) - (PI / 2);
    const angle_in_next = angle_to_x([current, next_other]) + 3*(PI / 2);

    const in_ = polar_to_cartesian(current, r, angle_in);
    const out = polar_to_cartesian(current, r, angle_out);
    const in_next = polar_to_cartesian(next, r, angle_in_next);

    if (eq_point(current, previous)) {
      ret.push({
        in: out,
        out,
        center: current,
        r,
        large_arc: 0,
        sweep: 1,
        in_next: eq_point(current, next) ? out : in_next,
      });
    } else {
      ret.push({
        in: in_,
        out,
        center: current,
        r,
        large_arc: 0,
        sweep: 1,
        in_next,
      });
    }
  }

  return ret;
}

export function lerp(x0, x1, t) {
  return ((1 - t) * x0) + (t * x1);
}

export function lerp_n(xs0, xs1, t) {
  const ret = [];

  for (let i = 0; i < xs0.length; i++) {
    ret.push(lerp(xs0[i], xs1[i], t));
  }

  return ret;
}

export function lerp_poly(p0, p1, t) {
  const ret = [];

  for (let i = 0; i < p0.length; i++) {
    ret.push(lerp_n(p0[i], p1[i], t));
  }
  return ret;
}
