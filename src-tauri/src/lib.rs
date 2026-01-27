use serde::{Deserialize, Serialize};

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
        .map_err(|_| E::custom("invalid u16 string"))
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
        .map_err(|_| E::custom("invalid u8 string"))
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
        .map_err(serde::de::Error::custom)
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
  use super::{analyze_one, deserialize_percent_f32, Decision, TeaVariety};
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
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      load_csv,
      analyze_varieties,
      save_analysis_csv
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
