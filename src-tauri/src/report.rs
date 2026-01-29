use crate::{Decision, ExportRow, Summary, ViewModel};

/* Escape a value for safe Markdown output */
fn escape_md(s: &str) -> String {
  s.replace('|', "\\|").replace('\n', " ")
}

/* Build a Markdown report from a computed ViewModel */
pub fn build_report_markdown(view_model: &ViewModel) -> String {
  let Summary {
    total,
    counts,
    avg_score,
    top,
    bottom,
  } = &view_model.summary;

  let keep = counts.keep;
  let review = counts.review;
  let discard = counts.discard;

  let pct = |n: u32| -> u32 {
    if *total == 0 {
      0
    } else {
      ((n as f32) / (*total as f32) * 100.0).round() as u32
    }
  };

  let mut out = String::new();
  out.push_str("# TeaBreed Analyzer Report\n\n");
  out.push_str("## Summary\n\n");
  out.push_str(&format!("- total: {}\n", total));
  out.push_str(&format!(
    "- avgScore: {:.1}\n",
    avg_score
  ));
  out.push_str(&format!(
    "- keep: {} ({}%)\n",
    keep,
    pct(keep)
  ));
  out.push_str(&format!(
    "- review: {} ({}%)\n",
    review,
    pct(review)
  ));
  out.push_str(&format!(
    "- discard: {} ({}%)\n",
    discard,
    pct(discard)
  ));

  out.push_str("\n## Top\n\n");
  if top.is_empty() {
    out.push_str("- (none)\n");
  } else {
    out.push_str("| id | name | total_score |\n");
    out.push_str("|---|---|---:|\n");
    for r in top {
      out.push_str(&format!(
        "| {} | {} | {:.1} |\n",
        escape_md(&r.id),
        escape_md(&r.name),
        r.total_score
      ));
    }
  }

  out.push_str("\n## Bottom\n\n");
  if bottom.is_empty() {
    out.push_str("- (none)\n");
  } else {
    out.push_str("| id | name | total_score |\n");
    out.push_str("|---|---|---:|\n");
    for r in bottom {
      out.push_str(&format!(
        "| {} | {} | {:.1} |\n",
        escape_md(&r.id),
        escape_md(&r.name),
        r.total_score
      ));
    }
  }

  out.push_str("\n## Charts\n\n");
  out.push_str("### Generation averages\n\n");
  if view_model.generation_averages.is_empty() {
    out.push_str("- (none)\n");
  } else {
    out.push_str("| generation | avgScore | count |\n");
    out.push_str("|---|---:|---:|\n");
    for d in &view_model.generation_averages {
      out.push_str(&format!(
        "| {} | {:.1} | {} |\n",
        escape_md(&d.generation),
        d.avg_score,
        d.count
      ));
    }
  }

  out.push_str("\n### Yearly trend\n\n");
  if view_model.yearly_trend.is_empty() {
    out.push_str("- (none)\n");
  } else {
    out.push_str("| year | avgScore | count |\n");
    out.push_str("|---:|---:|---:|\n");
    for d in &view_model.yearly_trend {
      out.push_str(&format!(
        "| {} | {:.1} | {} |\n",
        d.year,
        d.avg_score,
        d.count
      ));
    }
  }

  out
}

/* Save analysis rows to JSON file */
#[tauri::command]
pub fn save_analysis_json(path: String, rows: Vec<ExportRow>) -> Result<(), String> {
  let bytes = serde_json::to_vec_pretty(&rows).map_err(|e| e.to_string())?;
  std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
  Ok(())
}

/* Save a Markdown report to file */
#[tauri::command]
pub fn save_report_markdown(path: String, view_model: ViewModel) -> Result<(), String> {
  let md = build_report_markdown(&view_model);
  std::fs::write(&path, md.as_bytes()).map_err(|e| e.to_string())?;
  Ok(())
}

/* Save validation issues as JSON */
#[tauri::command]
pub fn save_issues_json(path: String, issues: Vec<crate::CsvIssue>) -> Result<(), String> {
  let bytes = serde_json::to_vec_pretty(&issues).map_err(|e| e.to_string())?;
  std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
  Ok(())
}

/* Save validation issues as CSV */
#[tauri::command]
pub fn save_issues_csv(path: String, issues: Vec<crate::CsvIssue>) -> Result<(), String> {
  let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
  let mut w = csv::Writer::from_writer(file);
  for it in issues {
    // simple CSV row: row,line,id,message
    w.write_record(&[
      it.row.to_string(),
      it.line.to_string(),
      it.id.clone().unwrap_or_default(),
      it.message.clone(),
    ])
    .map_err(|e| e.to_string())?;
  }
  w.flush().map_err(|e| e.to_string())?;
  Ok(())
}

/* Convert Decision to a human label */
#[allow(dead_code)]
fn decision_label(decision: Decision) -> &'static str {
  match decision {
    Decision::Keep => "keep",
    Decision::Review => "review",
    Decision::Discard => "discard",
  }
}

#[cfg(test)]
mod tests {
  use super::{build_report_markdown, save_analysis_json, save_report_markdown};
  use crate::{
    Decision,
    ExportRow,
    Summary,
    SummaryCounts,
    SummaryRow,
    ViewModel,
    YearAvg,
    GenerationAvg,
  };
  use std::time::{SystemTime, UNIX_EPOCH};

  /* build_report_markdown should include key sections */
  #[test]
  fn report_contains_sections() {
    let vm = ViewModel {
      generations: vec!["F1".to_string()],
      filtered_rows: vec![],
      summary: Summary {
        total: 2,
        counts: SummaryCounts {
          keep: 1,
          review: 1,
          discard: 0,
        },
        avg_score: 70.0,
        top: vec![SummaryRow {
          id: "A".to_string(),
          name: "A|Name".to_string(),
          total_score: 80.0,
        }],
        bottom: vec![SummaryRow {
          id: "B".to_string(),
          name: "B".to_string(),
          total_score: 60.0,
        }],
      },
      generation_averages: vec![GenerationAvg {
        generation: "F1".to_string(),
        avg_score: 70.0,
        count: 2,
      }],
      yearly_trend: vec![YearAvg {
        year: 2024,
        avg_score: 70.0,
        count: 2,
      }],
    };

    let md = build_report_markdown(&vm);
    assert!(md.contains("# TeaBreed Analyzer Report"));
    assert!(md.contains("## Summary"));
    assert!(md.contains("## Top"));
    assert!(md.contains("## Bottom"));
    assert!(md.contains("## Charts"));

    /* Ensure escaping for Markdown tables */
    assert!(md.contains("A\\|Name"));

    /* Decision label strings are present via summary lines */
    assert!(md.contains("keep: 1"));
    assert!(md.contains("review: 1"));

    /* Avoid unused import warning for Decision in this test module */
    let _ = Decision::Keep;
  }

  /* save_analysis_json should write a JSON file */
  #[test]
  fn save_analysis_json_writes_file() {
    let rows = vec![ExportRow {
      id: "A".to_string(),
      name: "A".to_string(),
      generation: "F1".to_string(),
      location: "X".to_string(),
      year: 2024,
      germination_rate: 90.0,
      growth_score: 4,
      disease_resistance: 4,
      aroma_score: 3,
      total_score: 88.0,
      decision: Decision::Keep,
    }];

    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    path.push(format!(
      "tea_breed_analyzer_save_json_{}_{}.json",
      std::process::id(),
      suffix
    ));

    save_analysis_json(path.to_string_lossy().to_string(), rows).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert!(text.contains("\"id\""));
    assert!(text.contains("\"A\""));
  }

  /* save_report_markdown should write a Markdown file */
  #[test]
  fn save_report_markdown_writes_file() {
    let vm = ViewModel {
      generations: vec!["F1".to_string()],
      filtered_rows: vec![],
      summary: Summary {
        total: 0,
        counts: SummaryCounts {
          keep: 0,
          review: 0,
          discard: 0,
        },
        avg_score: 0.0,
        top: vec![],
        bottom: vec![],
      },
      generation_averages: vec![],
      yearly_trend: vec![],
    };

    let mut path = std::env::temp_dir();
    let suffix = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    path.push(format!(
      "tea_breed_analyzer_save_md_{}_{}.md",
      std::process::id(),
      suffix
    ));

    save_report_markdown(path.to_string_lossy().to_string(), vm).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert!(text.contains("# TeaBreed Analyzer Report"));
    assert!(text.contains("## Summary"));
  }
}

