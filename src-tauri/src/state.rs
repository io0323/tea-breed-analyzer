use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

/* Persistent app state stored on disk */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
  pub csv_path: String,
  pub params: crate::ViewParams,
  pub selected_id: String,
}

/* Resolve state file path inside the app data dir */
pub fn resolve_state_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
  let mut dir = app
    .path()
    .app_data_dir()
    .map_err(|e| e.to_string())?;

  std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
  dir.push("tea_breed_analyzer_state.json");
  Ok(dir)
}

/* Load AppState from an explicit path */
pub fn load_from_path(path: &Path) -> Result<Option<AppState>, String> {
  if !path.exists() {
    return Ok(None);
  }
  let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
  let state = serde_json::from_slice::<AppState>(&bytes).map_err(|e| e.to_string())?;
  Ok(Some(state))
}

/* Save AppState to an explicit path */
pub fn save_to_path(path: &Path, state: &AppState) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
  }
  let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
  std::fs::write(path, bytes).map_err(|e| e.to_string())?;
  Ok(())
}

/* Load persisted app state */
#[tauri::command]
pub fn load_app_state(app: tauri::AppHandle) -> Result<Option<AppState>, String> {
  let path = resolve_state_path(&app)?;
  load_from_path(&path)
}

/* Save persisted app state */
#[tauri::command]
pub fn save_app_state(app: tauri::AppHandle, state: AppState) -> Result<(), String> {
  let path = resolve_state_path(&app)?;
  save_to_path(&path, &state)
}

#[cfg(test)]
mod tests {
  use super::{load_from_path, save_to_path, AppState};
  use crate::{SortDir, SortKey, ViewParams};
  use std::time::{SystemTime, UNIX_EPOCH};

  /* AppState should round-trip via JSON file */
  #[test]
  fn app_state_round_trip() {
    let params = ViewParams {
      query: "q".to_string(),
      decision: None,
      generation: None,
      year_from: Some(2020),
      year_to: Some(2025),
      sort_key: SortKey::TotalScore,
      sort_dir: SortDir::Desc,
      top_n: 3,
    };

    let state = AppState {
      csv_path: "/tmp/example.csv".to_string(),
      params,
      selected_id: "A-001".to_string(),
    };

    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    path.push(format!(
      "tea_breed_analyzer_state_roundtrip_{}_{}.json",
      std::process::id(),
      suffix
    ));

    save_to_path(&path, &state).unwrap();
    let loaded = load_from_path(&path).unwrap().expect("missing state");
    let _ = std::fs::remove_file(&path);

    assert_eq!(loaded.csv_path, state.csv_path);
    assert_eq!(loaded.selected_id, state.selected_id);
    assert_eq!(loaded.params.query, state.params.query);
    assert_eq!(loaded.params.year_from, state.params.year_from);
    assert_eq!(loaded.params.sort_key as u8, state.params.sort_key as u8);
  }

  /* load_from_path should return None for missing file */
  #[test]
  fn load_from_path_missing_returns_none() {
    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    path.push(format!(
      "tea_breed_analyzer_state_missing_{}_{}.json",
      std::process::id(),
      suffix
    ));

    if path.exists() {
      let _ = std::fs::remove_file(&path);
    }
    let loaded = load_from_path(&path).unwrap();
    assert!(loaded.is_none());
  }
}

