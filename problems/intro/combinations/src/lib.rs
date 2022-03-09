#![forbid(unsafe_code)]

pub fn combine(arr: &[i32], k: usize, cur: &mut Vec<i32>, result: &mut Vec<Vec<i32>>) {
    if k == 0 {
        result.push(cur.clone());
        return;
    }

    for i in 0..arr.len() {
        cur.push(arr[i]);
        combine(&arr[i + 1..], k - 1, cur, result);
        cur.pop();
    }
}

pub fn combinations(arr: &[i32], k: usize) -> Vec<Vec<i32>> {
    let mut result = Vec::<Vec<i32>>::new();
    let mut cur = Vec::<i32>::with_capacity(k);

    combine(arr, k, &mut cur, &mut result);

    result
}
