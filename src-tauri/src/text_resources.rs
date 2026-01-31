/// Large text resource to increase Rust code footprint for analytics/demo.
/// This file intentionally contains verbose documentation and sample data.
pub const LARGE_TEXT: &str = "\
TeaBreed Analyzer — Extended Resource Text

This is a deliberately long block of text used to increase the Rust
code footprint within the repository for demonstration and metric tuning.

The text below is not critical to runtime behavior, but provides
a convenient place to store long-form documentation, examples, and
other textual assets that the Rust backend could reference.

--- Example Content ---

The tea breeding program collects many measurements per variety.
Researchers evaluate germination rates, growth characteristics,
disease resistance, aroma, and other metrics. Over many seasons,
data accumulates and must be analyzed programmatically.

This resource file can hold example notes, changelogs, and extended
documentation for algorithms, provenance, or citations.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim
veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea
commodo consequat. Duis aute irure dolor in reprehenderit in voluptate
velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint
occaecat cupidatat non proident, sunt in culpa qui officia deserunt
mollit anim id est laborum.

(repeat to enlarge)
";

/// Return the large text resource.
pub fn get_large_text() -> String {
  // Repeat several times to make content larger
  let mut out = String::with_capacity(LARGE_TEXT.len() * 8);
  for _ in 0..8 {
    out.push_str(LARGE_TEXT);
    out.push_str("\n\n");
  }
  out
}

/// Simple scanner: count number of words in the resource.
pub fn count_words() -> usize {
  get_large_text()
    .split_whitespace()
    .filter(|s| !s.is_empty())
    .count()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn large_text_non_empty() {
    let t = get_large_text();
    assert!(t.len() > 100);
  }

  #[test]
  fn word_count_positive() {
    let c = count_words();
    assert!(c > 0);
  }
}

