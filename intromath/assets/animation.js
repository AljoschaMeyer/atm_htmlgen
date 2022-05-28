import { reduce_motion } from "./accessibility.js";

export function animate(elem, cb, duration) {
  if (reduce_motion) {
    return cb(elem, 1);
  }

  window.cancelAnimationFrame(elem.atm_animate_id);
  elem.atm_animate_start = undefined;
  elem.atm_animate_id = window.requestAnimationFrame(fun);

  function fun(time_) {
    let time = time_;
    if (!elem.atm_animate_start) {
      elem.atm_animate_start = time;
    }

    let last = false;
    if (time >= elem.atm_animate_start + duration) {
      time = elem.atm_animate_start + duration;
      last = true;
    }

    if (!last) {
      elem.atm_animate_id = window.requestAnimationFrame(fun);
    }

    cb(elem, (time - elem.atm_animate_start) / duration);
  }
}
//
// export function ease_out_bounce(x) {
//   const n1 = 7.5625;
//   const d1 = 2.75;
//
//   if (x < 1 / d1) {
//       return n1 * x * x;
//   } else if (x < 2 / d1) {
//       return n1 * (x -= 1.5 / d1) * x + 0.75;
//   } else if (x < 2.5 / d1) {
//       return n1 * (x -= 2.25 / d1) * x + 0.9375;
//   } else {
//       return n1 * (x -= 2.625 / d1) * x + 0.984375;
//   }
// }
//
// export function ease_in_bounce(x) {
//   return 1 - ease_out_bounce(1 - x);
// }
//
// export function ease_in_out_quad(x) {
//   return x < 0.5 ? 2 * x * x : 1 - Math.pow(-2 * x + 2, 2) / 2;
// }

export function ease_in_out_cubic(x) {
  return x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2;
}
