use crate::{ExportRow, GenerationAvg, YearAvg};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedGeneration {
  pub generation: String,
  pub avg_score: f32,
  pub median_score: f32,
  pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedYear {
  pub year: u16,
  pub avg: f32,
  pub stddev: f32,
  pub count: u32,
}

pub fn compute_advanced_generation(rows: &[ExportRow]) -> Vec<AdvancedGeneration> {
  let mut map: BTreeMap<String, Vec<f32>> = BTreeMap::new();
  for r in rows {
    map.entry(r.generation.clone()).or_default().push(r.total_score);
  }
  let mut out = Vec::new();
  for (g, mut vals) in map {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let sum: f32 = vals.iter().copied().sum();
    let avg = if vals.is_empty() { 0.0 } else { sum / (vals.len() as f32) };
    let median = if vals.is_empty() {
      0.0
    } else {
      let m = vals.len() / 2;
      if vals.len() % 2 == 1 { vals[m] } else { (vals[m-1] + vals[m]) / 2.0 }
    };
    out.push(AdvancedGeneration {
      generation: g,
      avg_score: avg,
      median_score: median,
      count: vals.len() as u32,
    });
  }
  out
}

pub fn compute_advanced_year(rows: &[ExportRow]) -> Vec<AdvancedYear> {
  let mut map: BTreeMap<u16, Vec<f32>> = BTreeMap::new();
  for r in rows {
    map.entry(r.year).or_default().push(r.total_score);
  }
  let mut out = Vec::new();
  for (year, vals) in map {
    let n = vals.len() as f32;
    let sum: f32 = vals.iter().copied().sum();
    let avg = if n == 0.0 { 0.0 } else { sum / n };
    let var = if n <= 1.0 {
      0.0
    } else {
      vals.iter().map(|v| (v - avg)*(v - avg)).sum::<f32>() / (n - 1.0)
    };
    let stddev = var.sqrt();
    out.push(AdvancedYear {
      year,
      avg,
      stddev,
      count: vals.len() as u32,
    });
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ExportRow;

  #[test]
  fn advanced_generation_metrics() {
    let rows = vec![
      ExportRow {
        id: "a".into(), name:"a".into(), generation:"F1".into(), location:"x".into(),
        year:2023, germination_rate:90.0, growth_score:4, disease_resistance:4, aroma_score:3,
        total_score:80.0, decision:crate::Decision::Keep
      },
      ExportRow {
        id: "b".into(), name:"b".into(), generation:"F1".into(), location:"x".into(),
        year:2023, germination_rate:85.0, growth_score:4, disease_resistance:4, aroma_score:3,
        total_score:70.0, decision:crate::Decision::Review
      },
    ];
    let ag = compute_advanced_generation(&rows);
    assert_eq!(ag.len(), 1);
    assert!((ag[0].avg_score - 75.0).abs() < 1e-6);
    assert!((ag[0].median_score - 75.0).abs() < 1e-6);
  }

  #[test]
  fn advanced_year_metrics() {
    let rows = vec![
      ExportRow {
        id: "a".into(), name:"a".into(), generation:"F1".into(), location:"x".into(),
        year:2022, germination_rate:90.0, growth_score:4, disease_resistance:4, aroma_score:3,
        total_score:80.0, decision:crate::Decision::Keep
      },
      ExportRow {
        id: "b".into(), name:"b".into(), generation:"F2".into(), location:"x".into(),
        year:2023, germination_rate:85.0, growth_score:4, disease_resistance:4, aroma_score:3,
        total_score:60.0, decision:crate::Decision::Review
      },
    ];
    let ay = compute_advanced_year(&rows);
    assert_eq!(ay.len(), 2);
  }
}

