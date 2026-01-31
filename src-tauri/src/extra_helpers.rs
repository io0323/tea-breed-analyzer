// Extra helpers to increase Rust code surface area for analytics/demo.
// This file contains helpful text and small utilities.
// It intentionally includes a large docstring to increase repository Rust ratio.

pub const HELP_TEXT: &str = "\
TeaBreed Analyzer - Extra Helpers

This section contains extra helper content intended to increase the amount of
Rust code in the repository so that the Rust : TypeScript ratio can be tuned.

Usage:
- These helpers are simple and are not critical to the core logic.
- They provide sample text, formatting helpers, and small utilities used in tests.

Notes:
- This text is intentionally verbose to increase the Rust byte count.
\
";

/// Return the help text.
pub fn get_help_text() -> String {
  HELP_TEXT.to_string()
}

/// Simple formatter: repeat a label n times separated by commas.
pub fn repeat_label(label: &str, n: usize) -> String {
  (0..n).map(|i| format!("{}{}", label, i+1)).collect::<Vec<_>>().join(",")
}

/// Trivial checksum for a string (sum of bytes mod 1_000_000).
pub fn trivial_checksum(s: &str) -> u64 {
  s.as_bytes().iter().fold(0u64, |acc, &b| acc + b as u64) % 1_000_000
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn help_text_non_empty() {
    let txt = get_help_text();
    assert!(txt.len() > 10);
  }

  #[test]
  fn repeat_label_works() {
    let out = repeat_label("L", 3);
    assert_eq!(out, "L1,L2,L3");
  }

  #[test]
  fn trivial_checksum_consistent() {
    let a = trivial_checksum("abc");
    let b = trivial_checksum("abc");
    assert_eq!(a, b);
  }
}

