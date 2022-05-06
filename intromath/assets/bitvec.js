import {
  PI,
  polar_to_cartesian,
} from "./geometry.js";

export function bitvec_count(s) {
  let count = 0;
  s.forEach(x => {
    if (x) {
      count++;
    }
  });
  return count;
}

export function bitvec_first(s) {
  return bitvec_next(s, 0);
}

export function bitvec_next(s, i) {
  for (let j = 1; j <= s.length; j++) {
    if (s[(i + j) % s.length]) {
      return (i + j) % s.length;
    }
  }

  throw "empty bitvec has no next element";
}

export function bitvec_previous(s, i) {
  for (let j = s.length - 1; j >= 0; j--) {
    if (s[(i + j) % s.length]) {
      return (i + j) % s.length;
    }
  }

  throw "empty bitvec has no previous element";
}
