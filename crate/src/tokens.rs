/// Count characters in `text` after collapsing all runs of whitespace to a
/// single space and trimming leading/trailing whitespace.
///
/// Single-pass, zero-allocation implementation.
pub fn default_length_counter(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_ws = false;
    let mut started = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if started {
                in_ws = true;
            }
        } else {
            if in_ws {
                count += 1;
            }
            count += 1;
            in_ws = false;
            started = true;
        }
    }
    count
}
