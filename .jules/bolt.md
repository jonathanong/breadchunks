## 2024-05-18 - `captures_iter` vs `find_iter` in Regex

**Learning:** `Regex::captures_iter` has significant overhead compared to `Regex::find_iter` when you only need the matched text and can derive sub-captures via small string slicing.

In `breadchunks`, switching the header scanning loop in `crate/src/split.rs` from `captures_iter` to `find_iter` reduced matching overhead (~3.5x observed in a focused benchmark). The header regex now avoids capture groups and instead strips a leading newline and optional `\r` suffix when normalizing the header text.

**Action:** When a capture group only removes a predictable prefix/suffix (like an optional leading newline), prefer `find_iter` and explicit slicing over full capture extraction.
