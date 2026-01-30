use crate::validators::ValidatorRule;

/// Parse a simple rule string like "germination_rate <= 80" or "growth_score == 4"
/// Returns ValidatorRule or error string.
pub fn parse_simple_rule(s: &str) -> Result<ValidatorRule, String> {
  let pieces: Vec<&str> = s.split_whitespace().collect();
  if pieces.len() != 3 {
    return Err("expected format: <field> <op> <value>".to_string());
  }
  let field = pieces[0].to_string();
  let op = match pieces[1] {
    "<=" => "lte",
    ">=" => "gte",
    "==" | "=" => "eq",
    "<" => "lt",
    ">" => "gt",
    other => return Err(format!("unsupported operator: {}", other)),
  }
  .to_string();
  let value = pieces[2].to_string();
  Ok(ValidatorRule { field, op, value })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_lte_rule() {
    let r = parse_simple_rule("germination_rate <= 80").unwrap();
    assert_eq!(r.field, "germination_rate");
    assert_eq!(r.op, "lte");
    assert_eq!(r.value, "80");
  }

  #[test]
  fn parse_eq_rule() {
    let r = parse_simple_rule("growth_score == 4").unwrap();
    assert_eq!(r.field, "growth_score");
    assert_eq!(r.op, "eq");
    assert_eq!(r.value, "4");
  }

  #[test]
  fn parse_invalid_operator() {
    let err = parse_simple_rule("a <> 1").unwrap_err();
    assert!(err.contains("unsupported operator"));
  }
}

