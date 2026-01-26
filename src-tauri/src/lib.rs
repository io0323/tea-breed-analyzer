use serde::{Deserialize, Serialize};

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
  pub decision: String,
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

/* Compute total score and decision for a variety */
fn analyze_one(variety: &TeaVariety) -> AnalysisResult {
  let total_score = variety.germination_rate * 0.4
    + (variety.growth_score as f32) * 10.0 * 0.3
    + (variety.disease_resistance as f32) * 10.0 * 0.2
    + (variety.aroma_score as f32) * 10.0 * 0.1;

  let decision = if total_score >= 75.0 {
    "keep"
  } else if total_score >= 50.0 {
    "review"
  } else {
    "discard"
  };

  AnalysisResult {
    id: variety.id.clone(),
    total_score,
    decision: decision.to_string(),
  }
}

/* Load tea varieties from a CSV file path */
#[tauri::command]
fn load_csv(path: String) -> Result<Vec<TeaVariety>, String> {
  let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
  let mut reader = csv::Reader::from_reader(file);
  let mut out: Vec<TeaVariety> = Vec::new();

  for row in reader.deserialize::<TeaVariety>() {
    let variety = row.map_err(|e| e.to_string())?;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![load_csv, analyze_varieties])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
