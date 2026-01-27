use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod state;
mod report;

use state::{load_app_state, save_app_state};
use report::{save_analysis_json, save_report_markdown};

/* Decision categories for selection outcomes */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
  Keep,
  Review,
  Discard,
}

/* Tea variety data model for CSV/JSON interop */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeaVariety {
  #[serde(deserialize_with = "deserialize_trim_string", alias = "ID")]
  pub id: String,
  #[serde(deserialize_with = "deserialize_trim_string", alias = "Name")]
  pub name: String,
  #[serde(deserialize_with = "deserialize_trim_string", alias = "Generation")]
  pub generation: String,
  #[serde(deserialize_with = "deserialize_trim_string", alias = "Location")]
  pub location: String,
  #[serde(deserialize_with = "deserialize_u16", alias = "Year")]
  pub year: u16,
  #[serde(
    deserialize_with = "deserialize_percent_f32",
    alias = "germinationRate",
    alias = "germination",
    alias = "GerminationRate"
  )]
  pub germination_rate: f32,
  #[serde(
    deserialize_with = "deserialize_u8",
    alias = "growthScore",
    alias = "growth",
    alias = "GrowthScore"
  )]
  pub growth_score: u8,
  #[serde(
    deserialize_with = "deserialize_u8",
    alias = "diseaseResistance",
    alias = "disease",
    alias = "DiseaseResistance"
  )]
  pub disease_resistance: u8,
  #[serde(
    deserialize_with = "deserialize_u8",
    alias = "aromaScore",
    alias = "aroma",
    alias = "AromaScore"
  )]
  pub aroma_score: u8,
}

/* Analysis result returned to the UI */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
  pub id: String,
  pub total_score: f32,
  pub decision: Decision,
}

/* Export row structure for CSV output */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRow {
  pub id: String,
  pub name: String,
  pub generation: String,
  pub location: String,
  pub year: u16,
  pub germination_rate: f32,
  pub growth_score: u8,
  pub disease_resistance: u8,
  pub aroma_score: u8,
  pub total_score: f32,
  pub decision: Decision,
}

/* Sort key for table/export ordering */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
  TotalScore,
  Year,
  Name,
  Generation,
  Decision,
  Id,
}

/* Sort direction */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDir {
  Asc,
  Desc,
}

/* Request parameters for building the UI view model */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewParams {
  pub query: String,
  pub decision: Option<Decision>,
  pub generation: Option<String>,
  pub year_from: Option<u16>,
  pub year_to: Option<u16>,
  pub sort_key: SortKey,
  pub sort_dir: SortDir,
  pub top_n: usize,
}

/* Summary counts by decision */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SummaryCounts {
  pub keep: u32,
  pub review: u32,
  pub discard: u32,
}

/* Summary row used in top/bottom lists */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryRow {
  pub id: String,
  pub name: String,
  pub total_score: f32,
}

/* Summary data returned to the UI */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
  pub total: usize,
  pub counts: SummaryCounts,
  pub avg_score: f32,
  pub top: Vec<SummaryRow>,
  pub bottom: Vec<SummaryRow>,
}

/* Generation average for charts */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationAvg {
  pub generation: String,
  pub avg_score: f32,
  pub count: u32,
}

/* Yearly average for charts */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearAvg {
  pub year: u16,
  pub avg_score: f32,
  pub count: u32,
}

/* View model returned to the UI */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewModel {
  pub generations: Vec<String>,
  pub filtered_rows: Vec<ExportRow>,
  pub summary: Summary,
  pub generation_averages: Vec<GenerationAvg>,
  pub yearly_trend: Vec<YearAvg>,
}

/* Map decision to a stable rank for sorting */
fn decision_rank(decision: Decision) -> u8 {
  match decision {
    Decision::Keep => 0,
    Decision::Review => 1,
    Decision::Discard => 2,
  }
}

/* Compute unique generations (for filter dropdown) */
fn compute_generations(rows: &[ExportRow]) -> Vec<String> {
  let mut set: HashSet<String> = HashSet::new();
  for r in rows {
    set.insert(r.generation.clone());
  }
  let mut out = set.into_iter().collect::<Vec<_>>();
  out.sort_by(|a, b| a.cmp(b));
  out
}

/* Filter rows based on view params */
fn filter_rows(rows: &[ExportRow], params: &ViewParams) -> Vec<ExportRow> {
  let q = params.query.trim().to_lowercase();

  rows
    .iter()
    .filter(|r| {
      if let Some(d) = params.decision {
        if r.decision != d {
          return false;
        }
      }
      if let Some(gen) = params.generation.as_ref() {
        if r.generation != *gen {
          return false;
        }
      }
      if let Some(from) = params.year_from {
        if r.year < from {
          return false;
        }
      }
      if let Some(to) = params.year_to {
        if r.year > to {
          return false;
        }
      }
      if !q.is_empty() {
        let hay = format!("{} {} {} {}", r.id, r.name, r.location, r.generation)
          .to_lowercase();
        if !hay.contains(&q) {
          return false;
        }
      }
      true
    })
    .cloned()
    .collect::<Vec<_>>()
}

/* Sort rows for display/export */
fn sort_rows(rows: &mut [ExportRow], sort_key: SortKey, sort_dir: SortDir) {
  rows.sort_by(|a, b| {
    let cmp = match sort_key {
      SortKey::TotalScore => a.total_score.total_cmp(&b.total_score),
      SortKey::Year => a.year.cmp(&b.year),
      SortKey::Name => a.name.cmp(&b.name),
      SortKey::Generation => a.generation.cmp(&b.generation),
      SortKey::Decision => decision_rank(a.decision).cmp(&decision_rank(b.decision)),
      SortKey::Id => a.id.cmp(&b.id),
    };

    let cmp = if cmp == std::cmp::Ordering::Equal {
      a.id.cmp(&b.id)
    } else {
      cmp
    };

    match sort_dir {
      SortDir::Asc => cmp,
      SortDir::Desc => cmp.reverse(),
    }
  });
}

/* Build summary (counts/avg/top/bottom) from already-filtered rows */
fn compute_summary(rows: &[ExportRow], top_n: usize) -> Summary {
  let mut counts = SummaryCounts {
    keep: 0,
    review: 0,
    discard: 0,
  };
  let mut sum: f32 = 0.0;

  for r in rows {
    match r.decision {
      Decision::Keep => counts.keep += 1,
      Decision::Review => counts.review += 1,
      Decision::Discard => counts.discard += 1,
    }
    sum += r.total_score;
  }

  let avg_score = if rows.is_empty() {
    0.0
  } else {
    sum / (rows.len() as f32)
  };

  let n = if top_n == 0 { 3 } else { top_n };
  let mut by_score = rows.to_vec();
  by_score.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));

  let top = by_score
    .iter()
    .take(n)
    .map(|r| SummaryRow {
      id: r.id.clone(),
      name: r.name.clone(),
      total_score: r.total_score,
    })
    .collect::<Vec<_>>();

  let mut bottom = by_score
    .iter()
    .rev()
    .take(n)
    .map(|r| SummaryRow {
      id: r.id.clone(),
      name: r.name.clone(),
      total_score: r.total_score,
    })
    .collect::<Vec<_>>();
  bottom.reverse();

  Summary {
    total: rows.len(),
    counts,
    avg_score,
    top,
    bottom,
  }
}

/* Compute generation average scores from filtered rows */
fn compute_generation_averages(rows: &[ExportRow]) -> Vec<GenerationAvg> {
  let mut acc: HashMap<String, (f32, u32)> = HashMap::new();

  for r in rows {
    let entry = acc.entry(r.generation.clone()).or_insert((0.0, 0));
    entry.0 += r.total_score;
    entry.1 += 1;
  }

  let mut out = acc
    .into_iter()
    .map(|(generation, (sum, count))| GenerationAvg {
      generation,
      avg_score: if count == 0 { 0.0 } else { sum / (count as f32) },
      count,
    })
    .collect::<Vec<_>>();
  out.sort_by(|a, b| a.generation.cmp(&b.generation));
  out
}

/* Compute yearly trend (average total_score by year) from filtered rows */
fn compute_yearly_trend(rows: &[ExportRow]) -> Vec<YearAvg> {
  let mut acc: HashMap<u16, (f32, u32)> = HashMap::new();

  for r in rows {
    let entry = acc.entry(r.year).or_insert((0.0, 0));
    entry.0 += r.total_score;
    entry.1 += 1;
  }

  let mut out = acc
    .into_iter()
    .map(|(year, (sum, count))| YearAvg {
      year,
      avg_score: if count == 0 { 0.0 } else { sum / (count as f32) },
      count,
    })
    .collect::<Vec<_>>();
  out.sort_by(|a, b| a.year.cmp(&b.year));
  out
}

/* Validate TeaVariety domain constraints */
fn validate_variety(variety: &TeaVariety) -> Result<(), String> {
  if !(0.0..=100.0).contains(&variety.germination_rate) {
    return Err(format!(
      "germination_rate={} は 0..=100 である必要があります",
      variety.germination_rate
    ));
  }
  if !(1..=5).contains(&variety.growth_score) {
    return Err(format!(
      "growth_score={} は 1..=5 である必要があります",
      variety.growth_score
    ));
  }
  if !(1..=5).contains(&variety.disease_resistance) {
    return Err(format!(
      "disease_resistance={} は 1..=5 である必要があります",
      variety.disease_resistance
    ));
  }
  if !(1..=5).contains(&variety.aroma_score) {
    return Err(format!(
      "aroma_score={} は 1..=5 である必要があります",
      variety.aroma_score
    ));
  }
  Ok(())
}

/* Custom deserializer for trimmed strings */
fn deserialize_trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct TrimStringVisitor;

  impl<'de> serde::de::Visitor<'de> for TrimStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(v.trim().to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(v.trim().to_string())
    }
  }

  deserializer.deserialize_any(TrimStringVisitor)
}

/* Custom deserializer for u16 values from strings or numbers */
fn deserialize_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct U16Visitor;

  impl<'de> serde::de::Visitor<'de> for U16Visitor {
    type Value = u16;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a u16 number or numeric string")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      u16::try_from(v).map_err(|_| E::custom("out of range for u16"))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      if v < 0 {
        return Err(E::custom("negative value for u16"));
      }
      u16::try_from(v as u64).map_err(|_| E::custom("out of range for u16"))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      if !v.is_finite() {
        return Err(E::custom("non-finite value for u16"));
      }
      if v.fract() != 0.0 {
        return Err(E::custom("non-integer value for u16"));
      }
      if v < 0.0 {
        return Err(E::custom("negative value for u16"));
      }
      u16::try_from(v as u64).map_err(|_| E::custom("out of range for u16"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      let trimmed = v.trim();
      trimmed
        .parse::<u16>()
        .map_err(|_| E::custom(format!("invalid u16 string: {trimmed:?}")))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_str::<E>(&v)
    }
  }

  deserializer.deserialize_any(U16Visitor)
}

/* Custom deserializer for u8 values from strings or numbers */
fn deserialize_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct U8Visitor;

  impl<'de> serde::de::Visitor<'de> for U8Visitor {
    type Value = u8;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a u8 number or numeric string")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      u8::try_from(v).map_err(|_| E::custom("out of range for u8"))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      if v < 0 {
        return Err(E::custom("negative value for u8"));
      }
      u8::try_from(v as u64).map_err(|_| E::custom("out of range for u8"))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      if !v.is_finite() {
        return Err(E::custom("non-finite value for u8"));
      }
      if v.fract() != 0.0 {
        return Err(E::custom("non-integer value for u8"));
      }
      if v < 0.0 {
        return Err(E::custom("negative value for u8"));
      }
      u8::try_from(v as u64).map_err(|_| E::custom("out of range for u8"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      let trimmed = v.trim();
      trimmed
        .parse::<u8>()
        .map_err(|_| E::custom(format!("invalid u8 string: {trimmed:?}")))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_str::<E>(&v)
    }
  }

  deserializer.deserialize_any(U8Visitor)
}

/* Custom deserializer for percent values like "85" or "85%" */
fn deserialize_percent_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
  D: serde::Deserializer<'de>,
{
  struct PercentF32Visitor;

  impl<'de> serde::de::Visitor<'de> for PercentF32Visitor {
    type Value = f32;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("a percentage number as string (optional %) or number")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      if !v.is_finite() {
        return Err(E::custom("non-finite percentage"));
      }
      Ok(v as f32)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(v as f32)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      let trimmed = v.trim().trim_end_matches('%').trim();
      trimmed
        .parse::<f32>()
        .map_err(|_| serde::de::Error::custom(format!("invalid percent: {trimmed:?}")))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      self.visit_str::<E>(&v)
    }
  }

  deserializer.deserialize_any(PercentF32Visitor)
}

/* Compute total score and decision for a variety */
fn analyze_one(variety: &TeaVariety) -> AnalysisResult {
  /* Raw score max is 70.0 with current weights; normalize to 0..=100 */
  const RAW_MAX_SCORE: f32 = 70.0;

  let raw_score = variety.germination_rate * 0.4
    + (variety.growth_score as f32) * 10.0 * 0.3
    + (variety.disease_resistance as f32) * 10.0 * 0.2
    + (variety.aroma_score as f32) * 10.0 * 0.1;

  let total_score = (raw_score / RAW_MAX_SCORE) * 100.0;

  let decision = if total_score >= 75.0 {
    Decision::Keep
  } else if total_score >= 50.0 {
    Decision::Review
  } else {
    Decision::Discard
  };

  AnalysisResult {
    id: variety.id.clone(),
    total_score,
    decision,
  }
}

/* Load tea varieties from a CSV file path (internal helper) */
fn load_csv_file(path: &str) -> Result<Vec<TeaVariety>, String> {
  let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let mut reader = csv::Reader::from_reader(file);
  let mut out: Vec<TeaVariety> = Vec::new();

  for (idx, row) in reader.deserialize::<TeaVariety>().enumerate() {
    let row_num = idx + 1;
    let line_num = idx + 2;
    let variety = row.map_err(|e| {
      format!(
        "CSV読み込みエラー\npath: {}\nrow: {} (line {})\n原因: {}",
        path, row_num, line_num, e
      )
    })?;
    validate_variety(&variety).map_err(|e| {
      format!(
        "CSV検証エラー\npath: {}\nrow: {} (line {})\nid: {}\n原因: {}",
        path, row_num, line_num, variety.id, e
      )
    })?;
    out.push(variety);
  }

  Ok(out)
}

/* Load tea varieties from a CSV file path */
#[tauri::command]
fn load_csv(path: String) -> Result<Vec<TeaVariety>, String> {
  load_csv_file(&path)
}

/* Load a CSV and return analyzed rows (merged for UI/export) */
#[tauri::command]
fn load_and_analyze_csv(path: String) -> Result<Vec<ExportRow>, String> {
  let data = load_csv_file(&path)?;

  let mut out = data
    .into_iter()
    .map(|v| {
      let a = analyze_one(&v);
      ExportRow {
        id: v.id,
        name: v.name,
        generation: v.generation,
        location: v.location,
        year: v.year,
        germination_rate: v.germination_rate,
        growth_score: v.growth_score,
        disease_resistance: v.disease_resistance,
        aroma_score: v.aroma_score,
        total_score: a.total_score,
        decision: a.decision,
      }
    })
    .collect::<Vec<_>>();

  out.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));
  Ok(out)
}

/* Analyze tea varieties and return scoring decisions */
#[tauri::command]
fn analyze_varieties(data: Vec<TeaVariety>) -> Result<Vec<AnalysisResult>, String> {
  let results = data.iter().map(analyze_one).collect::<Vec<_>>();
  Ok(results)
}

/* Build UI view model (filter/sort/summary/charts) in Rust */
#[tauri::command]
fn compute_view_model(rows: Vec<ExportRow>, params: ViewParams) -> Result<ViewModel, String> {
  let generations = compute_generations(&rows);
  let mut filtered = filter_rows(&rows, &params);

  sort_rows(&mut filtered, params.sort_key, params.sort_dir);

  let summary = compute_summary(&filtered, params.top_n);
  let generation_averages = compute_generation_averages(&filtered);
  let yearly_trend = compute_yearly_trend(&filtered);

  Ok(ViewModel {
    generations,
    filtered_rows: filtered,
    summary,
    generation_averages,
    yearly_trend,
  })
}

/* Save analysis rows to CSV file */
#[tauri::command]
fn save_analysis_csv(path: String, rows: Vec<ExportRow>) -> Result<(), String> {
  let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
  let mut writer = csv::Writer::from_writer(file);

  for row in rows {
    writer.serialize(row).map_err(|e| e.to_string())?;
  }

  writer.flush().map_err(|e| e.to_string())?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{
    analyze_one,
    compute_view_model,
    deserialize_percent_f32,
    deserialize_u8,
    Decision,
    ExportRow,
    SortDir,
    SortKey,
    TeaVariety,
    ViewParams,
  };
  use serde::Deserialize;

  /* Helper struct to directly test percent deserializer */
  #[derive(Debug, Deserialize)]
  struct PercentWrap {
    #[serde(deserialize_with = "deserialize_percent_f32")]
    value: f32,
  }

  /* Helper struct to directly test u8 deserializer */
  #[derive(Debug, Deserialize)]
  struct U8Wrap {
    #[serde(deserialize_with = "deserialize_u8")]
    value: u8,
  }

  /* Decision boundary: >= 75 => keep */
  #[test]
  fn decision_is_keep_at_75() {
    let v = TeaVariety {
      id: "TV".to_string(),
      name: "x".to_string(),
      generation: "F1".to_string(),
      location: "loc".to_string(),
      year: 2024,
      germination_rate: 100.0,
      growth_score: 4,
      disease_resistance: 4,
      aroma_score: 3,
    };
    let r = analyze_one(&v);
    assert!(r.total_score >= 75.0);
    assert_eq!(r.decision, Decision::Keep);
  }

  /* Decision boundary: >= 50 and < 75 => review */
  #[test]
  fn decision_is_review_between_50_and_74_9() {
    let v = TeaVariety {
      id: "TV".to_string(),
      name: "x".to_string(),
      generation: "F1".to_string(),
      location: "loc".to_string(),
      year: 2024,
      germination_rate: 50.0,
      growth_score: 3,
      disease_resistance: 3,
      aroma_score: 2,
    };
    let r = analyze_one(&v);
    assert!(r.total_score >= 50.0);
    assert!(r.total_score < 75.0);
    assert_eq!(r.decision, Decision::Review);
  }

  /* Decision boundary: < 50 => discard */
  #[test]
  fn decision_is_discard_below_50() {
    let v = TeaVariety {
      id: "TV".to_string(),
      name: "x".to_string(),
      generation: "F1".to_string(),
      location: "loc".to_string(),
      year: 2024,
      germination_rate: 0.0,
      growth_score: 1,
      disease_resistance: 1,
      aroma_score: 1,
    };
    let r = analyze_one(&v);
    assert!(r.total_score < 50.0);
    assert_eq!(r.decision, Decision::Discard);
  }

  /* Percent deserializer should accept both "85" and "85%" */
  #[test]
  fn deserialize_percent_accepts_percent_sign() {
    let a: PercentWrap = serde_json::from_str("{\"value\":\"85\"}").unwrap();
    let b: PercentWrap = serde_json::from_str("{\"value\":\"85%\"}").unwrap();
    assert_eq!(a.value, 85.0);
    assert_eq!(b.value, 85.0);
  }

  /* u8 deserializer error should include offending value */
  #[test]
  fn deserialize_u8_includes_value_in_error() {
    let err = serde_json::from_str::<U8Wrap>("{\"value\":\"x\"}").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("invalid u8 string"));
    assert!(msg.contains("\"x\""));

    let ok = serde_json::from_str::<U8Wrap>("{\"value\":\"5\"}").unwrap();
    assert_eq!(ok.value, 5);
  }

  /* load_csv error should include row/line/path for parse failures */
  #[test]
  fn load_csv_error_contains_context() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let csv_data = concat!(
      "id,name,generation,location,year,germination_rate,",
      "growth_score,disease_resistance,aroma_score\n",
      "TV-001,Yabukita,F1,Shizuoka,2024,92%,x,4,3\n",
    );

    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    path.push(format!(
      "tea_breed_analyzer_load_csv_error_{}_{}.csv",
      std::process::id(),
      suffix
    ));

    std::fs::write(&path, csv_data).unwrap();
    let res = super::load_csv(path.to_string_lossy().to_string());
    let _ = std::fs::remove_file(&path);

    let err = res.unwrap_err();
    assert!(err.contains("CSV読み込みエラー"));
    assert!(err.contains("path:"));
    assert!(err.contains("row: 1"));
    assert!(err.contains("line 2"));
  }

  /* load_and_analyze_csv should return merged rows for UI */
  #[test]
  fn load_and_analyze_csv_returns_merged_rows() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let csv_data = concat!(
      "id,name,generation,location,year,germination_rate,",
      "growth_score,disease_resistance,aroma_score\n",
      "TV-001,Yabukita,F1,Shizuoka,2024,92%,4,4,3\n",
    );

    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    path.push(format!(
      "tea_breed_analyzer_load_and_analyze_{}_{}.csv",
      std::process::id(),
      suffix
    ));

    std::fs::write(&path, csv_data).unwrap();
    let res = super::load_and_analyze_csv(path.to_string_lossy().to_string());
    let _ = std::fs::remove_file(&path);

    let rows = res.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "TV-001");
    assert!(rows[0].total_score > 0.0);
  }

  /* CSV should accept header aliases and trim values */
  #[test]
  fn csv_aliases_and_trim_work() {
    let csv_data = concat!(
      "ID,Name,Generation,Location,Year,",
      "germinationRate,growthScore,diseaseResistance,aromaScore\n",
      " TV-001 , Yabukita , F1 , Shizuoka , 2024 , 92% , 4 , 4 , 3 \n",
    );

    let mut reader = csv::ReaderBuilder::new()
      .has_headers(true)
      .from_reader(csv_data.as_bytes());

    let row = reader
      .deserialize::<super::TeaVariety>()
      .next()
      .expect("missing row")
      .expect("failed to parse row");

    assert_eq!(row.id, "TV-001");
    assert_eq!(row.name, "Yabukita");
    assert_eq!(row.generation, "F1");
    assert_eq!(row.location, "Shizuoka");
    assert_eq!(row.year, 2024);
    assert_eq!(row.germination_rate, 92.0);
    assert_eq!(row.growth_score, 4);
    assert_eq!(row.disease_resistance, 4);
    assert_eq!(row.aroma_score, 3);
  }

  /* compute_view_model should filter/sort and return consistent view model */
  #[test]
  fn compute_view_model_filters_and_sorts() {
    let rows = vec![
      ExportRow {
        id: "A-001".to_string(),
        name: "Yabukita".to_string(),
        generation: "F1".to_string(),
        location: "Shizuoka".to_string(),
        year: 2024,
        germination_rate: 90.0,
        growth_score: 4,
        disease_resistance: 4,
        aroma_score: 3,
        total_score: 88.0,
        decision: Decision::Keep,
      },
      ExportRow {
        id: "A-002".to_string(),
        name: "Okumidori".to_string(),
        generation: "F1".to_string(),
        location: "Kagoshima".to_string(),
        year: 2023,
        germination_rate: 70.0,
        growth_score: 3,
        disease_resistance: 3,
        aroma_score: 2,
        total_score: 62.0,
        decision: Decision::Review,
      },
      ExportRow {
        id: "B-001".to_string(),
        name: "Saemidori".to_string(),
        generation: "F2".to_string(),
        location: "Shizuoka".to_string(),
        year: 2022,
        germination_rate: 65.0,
        growth_score: 3,
        disease_resistance: 3,
        aroma_score: 2,
        total_score: 60.0,
        decision: Decision::Review,
      },
      ExportRow {
        id: "C-001".to_string(),
        name: "Yabukita Variant".to_string(),
        generation: "F1".to_string(),
        location: "Shizuoka".to_string(),
        year: 2021,
        germination_rate: 85.0,
        growth_score: 4,
        disease_resistance: 4,
        aroma_score: 3,
        total_score: 80.0,
        decision: Decision::Keep,
      },
    ];

    let params = ViewParams {
      query: "yabu".to_string(),
      decision: Some(Decision::Keep),
      generation: Some("F1".to_string()),
      year_from: Some(2022),
      year_to: Some(2024),
      sort_key: SortKey::Year,
      sort_dir: SortDir::Asc,
      top_n: 3,
    };

    let vm = compute_view_model(rows.clone(), params).unwrap();

    /* generations should be derived from all rows, not only filtered */
    assert_eq!(vm.generations, vec!["F1".to_string(), "F2".to_string()]);

    /* Filtered rows: keep + F1 + year 2022..2024 + query 'yabu' => A-001 only */
    assert_eq!(vm.filtered_rows.len(), 1);
    assert_eq!(vm.filtered_rows[0].id, "A-001");

    /* Summary should reflect filtered rows */
    assert_eq!(vm.summary.total, 1);
    assert_eq!(vm.summary.counts.keep, 1);
    assert_eq!(vm.summary.counts.review, 0);
    assert_eq!(vm.summary.counts.discard, 0);
  }

  /* compute_view_model should compute summary and chart series from filtered rows */
  #[test]
  fn compute_view_model_summary_and_charts() {
    let rows = vec![
      ExportRow {
        id: "A".to_string(),
        name: "A".to_string(),
        generation: "F1".to_string(),
        location: "X".to_string(),
        year: 2023,
        germination_rate: 0.0,
        growth_score: 1,
        disease_resistance: 1,
        aroma_score: 1,
        total_score: 80.0,
        decision: Decision::Keep,
      },
      ExportRow {
        id: "B".to_string(),
        name: "B".to_string(),
        generation: "F1".to_string(),
        location: "X".to_string(),
        year: 2023,
        germination_rate: 0.0,
        growth_score: 1,
        disease_resistance: 1,
        aroma_score: 1,
        total_score: 60.0,
        decision: Decision::Review,
      },
      ExportRow {
        id: "C".to_string(),
        name: "C".to_string(),
        generation: "F2".to_string(),
        location: "X".to_string(),
        year: 2024,
        germination_rate: 0.0,
        growth_score: 1,
        disease_resistance: 1,
        aroma_score: 1,
        total_score: 40.0,
        decision: Decision::Discard,
      },
    ];

    let params = ViewParams {
      query: "".to_string(),
      decision: None,
      generation: None,
      year_from: None,
      year_to: None,
      sort_key: SortKey::TotalScore,
      sort_dir: SortDir::Desc,
      top_n: 2,
    };

    let vm = compute_view_model(rows, params).unwrap();

    assert_eq!(vm.summary.total, 3);
    assert_eq!(vm.summary.counts.keep, 1);
    assert_eq!(vm.summary.counts.review, 1);
    assert_eq!(vm.summary.counts.discard, 1);
    assert!((vm.summary.avg_score - (80.0 + 60.0 + 40.0) / 3.0).abs() < 1e-6);

    /* Generation averages */
    assert_eq!(vm.generation_averages.len(), 2);
    assert_eq!(vm.generation_averages[0].generation, "F1");
    assert!((vm.generation_averages[0].avg_score - 70.0).abs() < 1e-6);
    assert_eq!(vm.generation_averages[0].count, 2);
    assert_eq!(vm.generation_averages[1].generation, "F2");
    assert!((vm.generation_averages[1].avg_score - 40.0).abs() < 1e-6);
    assert_eq!(vm.generation_averages[1].count, 1);

    /* Yearly trend */
    assert_eq!(vm.yearly_trend.len(), 2);
    assert_eq!(vm.yearly_trend[0].year, 2023);
    assert!((vm.yearly_trend[0].avg_score - 70.0).abs() < 1e-6);
    assert_eq!(vm.yearly_trend[0].count, 2);
    assert_eq!(vm.yearly_trend[1].year, 2024);
    assert!((vm.yearly_trend[1].avg_score - 40.0).abs() < 1e-6);
    assert_eq!(vm.yearly_trend[1].count, 1);
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      load_csv,
      load_and_analyze_csv,
      analyze_varieties,
      compute_view_model,
      load_app_state,
      save_app_state,
      save_analysis_json,
      save_report_markdown,
      save_analysis_csv
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
