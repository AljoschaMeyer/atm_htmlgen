export const PI = Math.PI;
export const EPSILON = 0.0001;

export function eq_point([x1, y1], [x2, y2], epsilon_) {
  const epsilon = epsilon_ ? epsilon_ : EPSILON;
  return (Math.abs(x1 - x2) <= epsilon) && (Math.abs(y1 - y2) <= epsilon);
}

export function distance([x1, y1], [x2, y2]) {
  return Math.sqrt((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1));
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

    if (eq_point(current, next_other)) {
      const start = polar_to_cartesian(current, r, 0);
      const end = polar_to_cartesian(current, r, PI);
      ret.push({
        in: start,
        out: end,
        center: current,
        r,
        large_arc: 0,
        sweep: 1,
        in_next: end,
      });
      ret.push({
        in: end,
        out: start,
        center: current,
        r,
        large_arc: 0,
        sweep: 1,
        in_next: start,
      });
      for (let j = 0; j < 3; j++) {
        ret.push({
          in: start,
          out: start,
          center: current,
          r,
          large_arc: 0,
          sweep: 1,
          in_next: start,
        });
      }
      return ret;
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

export function vec_combine(xs0, xs1, fun) {
  const ret = [];

  for (let i = 0; i < xs0.length; i++) {
    ret.push(fun(xs0[i], xs1[i]));
  }

  return ret;
}

// export function vec_scale(xs, c) {
//   return xs.map(x => x * c);
// }
//
// export function vec_add(xs0, xs1) {
//   return vec_combine(xs0, xs1, (x0, x1) => x0 + x1);
// }

export function vec_sub(xs0, xs1) {
  return vec_combine(xs0, xs1, (x0, x1) => x0 - x1);
}

// https://stackoverflow.com/a/3122532
export function closest_point_on_line([a, b], p) {
  if (eq_point(a, b)) {
    return a;
  }

  const a_to_p = vec_sub(p, a);
  const a_to_b = vec_sub(b, a);

  const atb2 = (a_to_b[0] * a_to_b[0]) + (a_to_b[1] * a_to_b[1]);
  const atp_dot_atb = (a_to_p[0] * a_to_b[0]) + (a_to_p[1] * a_to_b[1]);
  const t = Math.max(0, Math.min(1, atp_dot_atb / atb2));

  return [a[0] + (a_to_b[0] * t), a[1] + (a_to_b[1] * t)];
}

export function lerp(x0, x1, t) {
  return ((1 - t) * x0) + (t * x1);
}

export function lerp_n(xs0, xs1, t) {
  return vec_combine(xs0, xs1, (x0, x1) => lerp(x0, x1, t));
}

export function lerp_poly(p0, p1, t) {
  return vec_combine(p0, p1, (xs0, xs1) => lerp_n(xs0, xs1, t));
}
