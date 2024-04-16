use sailfish::TemplateOnce;

use crate::format::Number;
use crate::jobs::JobOutcome;

#[derive(TemplateOnce)]
#[template(path = "report.stpl")]
pub struct ReportTemplate {
	pub outcomes: Vec<JobOutcome>,
	pub took_seconds: f64,
	pub total_documents: Number<u64>,
	pub total_rows: Number<u64>,
	pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Default for ReportTemplate {
	fn default() -> Self {
		Self {
			outcomes: Vec::new(),
			took_seconds: 0.0,
			total_documents: 0.into(),
			total_rows: 0.into(),
			created_at: chrono::Utc::now(),
		}
	}
}
