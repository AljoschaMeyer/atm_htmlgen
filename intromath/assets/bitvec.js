import {
  PI,
  polar_to_cartesian,
} from "./geometry.js";

export function bitvec_singleton(i, n) {
  const ret = [];
  for (let j = 0; j < n; j++) {
    ret.push(j === i);
  }
  return ret;
}

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

export function bitvec_xor(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] != s2[i]);
  }

  return s3;
}

export function bitvec_without(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] && !s2[i]);
  }

  return s3;
}

export function bitvec_and(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] && s2[i]);
  }

  return s3;
}

export function bitvec_or(s1, s2) {
  const s3 = [];

  for (let i = 0; i < 5; i++) {
    s3.push(s1[i] || s2[i]);
  }

  return s3;
}

export function bitvec_eq(a1, a2) {
  if (a1.length != a2.length) {
    return false;
  }

  for (let i = 0; i < a1.length; i++) {
    if (a1[i] != a2[i]) {
      return false;
    }
  }

  return true;
}
