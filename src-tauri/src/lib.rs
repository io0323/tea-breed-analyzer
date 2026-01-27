use serde::{Deserialize, Serialize};

/* Decision categories for selection outcomes */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
  Keep,
  Review,
  Discard,
}

/* Analysis config shared with the UI */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AnalysisWeights {
  pub germination: f32,
  pub growth: f32,
  pub disease: f32,
  pub aroma: f32,
}

/* Analysis configuration (weights + thresholds) */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AnalysisConfig {
  pub weights: AnalysisWeights,
  pub keep_threshold: f32,
  pub review_threshold: f32,
  pub normalize_to_100: bool,
}

/* Default config matching current product logic */
fn default_config() -> AnalysisConfig {
  AnalysisConfig {
    weights: AnalysisWeights {
      germination: 0.4,
      growth: 0.3,
      disease: 0.2,
      aroma: 0.1,
    },
    keep_threshold: 75.0,
    review_threshold: 50.0,
    normalize_to_100: true,
  }
}

/* Validate AnalysisConfig domain constraints */
fn validate_config(config: &AnalysisConfig) -> Result<(), String> {
  let w = &config.weights;
  let weights = [w.germination, w.growth, w.disease, w.aroma];
  if weights.iter().any(|v| !v.is_finite() || *v < 0.0) {
    return Err("weights must be finite and >= 0".to_string());
  }
  let sum = weights.iter().sum::<f32>();
  if sum <= 0.0 {
    return Err("sum(weights) must be > 0".to_string());
  }
  if !config.keep_threshold.is_finite() || !config.review_threshold.is_finite() {
    return Err("thresholds must be finite".to_string());
  }
  if config.keep_threshold < config.review_threshold {
    return Err("keep_threshold must be >= review_threshold".to_string());
  }
  if config.review_threshold < 0.0 {
    return Err("review_threshold must be >= 0".to_string());
  }
  Ok(())
}

/* Tea variety data model for CSV/JSON interop */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeaVariety {
  pub id: String,
  pub name: String,
  pub generation: String,
  pub location: String,
  pub year: u16,
  #[serde(deserialize_with = "deserialize_percent_f32")]
  pub germination_rate: f32,
  pub growth_score: u8,
  pub disease_resistance: u8,
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

/* Validate TeaVariety domain constraints */
fn validate_variety(variety: &TeaVariety) -> Result<(), String> {
  if !(0.0..=100.0).contains(&variety.germination_rate) {
    return Err("germination_rate must be 0..=100".to_string());
  }
  if !(1..=5).contains(&variety.growth_score) {
    return Err("growth_score must be 1..=5".to_string());
  }
  if !(1..=5).contains(&variety.disease_resistance) {
    return Err("disease_resistance must be 1..=5".to_string());
  }
  if !(1..=5).contains(&variety.aroma_score) {
    return Err("aroma_score must be 1..=5".to_string());
  }
  Ok(())
}

/* Custom deserializer for percent values like "85" or "85%" */
fn deserialize_percent_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let raw = String::deserialize(deserializer)?;
  let trimmed = raw.trim().trim_end_matches('%').trim();
  trimmed
    .parse::<f32>()
    .map_err(serde::de::Error::custom)
}

/* Compute raw score with weights */
fn raw_score_with_config(variety: &TeaVariety, config: &AnalysisConfig) -> f32 {
  let w = &config.weights;
  variety.germination_rate * w.germination
    + (variety.growth_score as f32) * 10.0 * w.growth
    + (variety.disease_resistance as f32) * 10.0 * w.disease
    + (variety.aroma_score as f32) * 10.0 * w.aroma
}

/* Compute the theoretical maximum raw score for normalization */
fn raw_max_score(config: &AnalysisConfig) -> f32 {
  let w = &config.weights;
  (100.0 * w.germination)
    + (50.0 * w.growth)
    + (50.0 * w.disease)
    + (50.0 * w.aroma)
}

/* Compute total score and decision for a variety (with config) */
fn analyze_one_with_config(
  variety: &TeaVariety,
  config: &AnalysisConfig,
) -> AnalysisResult {
  let raw_score = raw_score_with_config(variety, config);
  let total_score = if config.normalize_to_100 {
    let max = raw_max_score(config);
    if max > 0.0 {
      (raw_score / max) * 100.0
    } else {
      0.0
    }
  } else {
    raw_score
  };

  let decision = if total_score >= config.keep_threshold {
    Decision::Keep
  } else if total_score >= config.review_threshold {
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

/* Compute total score and decision for a variety (default config) */
fn analyze_one(variety: &TeaVariety) -> AnalysisResult {
  analyze_one_with_config(variety, &default_config())
}

/* Load tea varieties from a CSV file path */
#[tauri::command]
fn load_csv(path: String) -> Result<Vec<TeaVariety>, String> {
  let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
  let mut reader = csv::Reader::from_reader(file);
  let mut out: Vec<TeaVariety> = Vec::new();

  for (idx, row) in reader.deserialize::<TeaVariety>().enumerate() {
    let variety = row.map_err(|e| format!("row {}: {}", idx + 1, e))?;
    validate_variety(&variety)
      .map_err(|e| format!("row {} (id={}): {}", idx + 1, variety.id, e))?;
    out.push(variety);
  }

  Ok(out)
}

/* Analyze tea varieties and return scoring decisions */
#[tauri::command]
fn analyze_varieties(data: Vec<TeaVariety>) -> Result<Vec<AnalysisResult>, String> {
  let results = data.iter().map(analyze_one).collect::<Vec<_>>();
  Ok(results)
}

/* Analyze tea varieties with a custom config */
#[tauri::command]
fn analyze_varieties_with_config(
  data: Vec<TeaVariety>,
  config: AnalysisConfig,
) -> Result<Vec<AnalysisResult>, String> {
  validate_config(&config)?;
  let results = data
    .iter()
    .map(|v| analyze_one_with_config(v, &config))
    .collect::<Vec<_>>();
  Ok(results)
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
    analyze_one_with_config,
    default_config,
    deserialize_percent_f32,
    Decision,
    TeaVariety,
  };
  use serde::Deserialize;

  /* Helper struct to directly test percent deserializer */
  #[derive(Debug, Deserialize)]
  struct PercentWrap {
    #[serde(deserialize_with = "deserialize_percent_f32")]
    value: f32,
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
    let r = analyze_one_with_config(&v, &default_config());
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
    let r = analyze_one_with_config(&v, &default_config());
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
    let r = analyze_one_with_config(&v, &default_config());
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
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      load_csv,
      analyze_varieties,
      analyze_varieties_with_config,
      save_analysis_csv
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
