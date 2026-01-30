use crate::{CsvIssue, TeaVariety};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorRule {
  pub field: String,
  pub op: String,    // "lte", "gte", "eq", "in"
  pub value: String, // compared as string, parsed as needed
}

fn check_rule(variety: &TeaVariety, rule: &ValidatorRule) -> Option<String> {
  match rule.field.as_str() {
    "germination_rate" => {
      let v = variety.germination_rate;
      let target = rule.value.trim().trim_end_matches('%').parse::<f32>().ok()?;
      match rule.op.as_str() {
        "lte" => {
          if v <= target { None } else { Some(format!("{} <= {}", v, target)) }
        }
        "gte" => {
          if v >= target { None } else { Some(format!("{} >= {}", v, target)) }
        }
        _ => None,
      }
    }
    "growth_score" => {
      let v = variety.growth_score as i32;
      let t = rule.value.parse::<i32>().ok()?;
      match rule.op.as_str() {
        "eq" => if v == t { None } else { Some(format!("{} == {}", v, t)) },
        "gte" => if v >= t { None } else { Some(format!("{} >= {}", v, t)) },
        "lte" => if v <= t { None } else { Some(format!("{} <= {}", v, t)) },
        _ => None,
      }
    }
    _ => None,
  }
}

/// Apply validator rules to a list of varieties, producing CsvIssue entries.
pub fn apply_rules_to_varieties(
  varieties: &[TeaVariety],
  rules: &[ValidatorRule],
) -> Vec<CsvIssue> {
  let mut issues = Vec::new();
  for (idx, v) in varieties.iter().enumerate() {
    for r in rules {
      if let Some(msg) = check_rule(v, r) {
        issues.push(CsvIssue {
          row: idx + 1,
          line: idx + 2,
          id: Some(v.id.clone()),
          message: format!("rule {} {} {}", r.field, r.op, r.value),
          expected: Some(r.value.clone()),
        });
      }
    }
  }
  issues
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::TeaVariety;

  #[test]
  fn rule_germination_rate_lte() {
    let t = TeaVariety {
      id: "a".into(),
      name: "n".into(),
      generation: "F1".into(),
      location: "x".into(),
      year: 2024,
      germination_rate: 85.0,
      growth_score: 3,
      disease_resistance: 3,
      aroma_score: 3,
    };
    let rules = vec![ValidatorRule {
      field: "germination_rate".into(),
      op: "lte".into(),
      value: "80".into(),
    }];
    let issues = apply_rules_to_varieties(&[t], &rules);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("germination_rate"));
  }
}

