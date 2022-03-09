#![forbid(unsafe_code)]

pub fn longest_common_prefix(strs: Vec<&str>) -> String {
    if strs.is_empty() {
        return String::new();
    }

    let mut max_len = strs[0].len();

    for str in strs.iter() {
        if max_len == 0 {
            return String::new();
        }

        let mut begin = strs[0].chars();
        let mut cur_len = 0usize;

        for c in str.chars() {
            if begin.next() != Some(c) {
                break;
            }
            cur_len += 1;

            if cur_len == max_len {
                break;
            }
        }

        max_len = cur_len;
    }

    strs[0].chars().take(max_len).collect()
}
