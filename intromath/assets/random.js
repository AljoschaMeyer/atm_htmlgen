export function random_int(max_exclusive) {
    return Math.floor(Math.random() * max_exclusive);
}

export function random_bool() {
    return Math.random() < 0.5;
}

export function random_from_array(arr, state) {
    return arr[random_int(arr.length, state)];
}

export function gen_interesting(gen, test, state) {
    while (true) {
        const x = gen(state);
        if (test(x)) {
            return x;
        }
    }
}