use async_graphql::{InputObject, Object};
use shared::database::TicketPermission;
use shared::old_types::{ObjectId, TicketObjectId};

use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::{
	error::ApiError,
	v3::gql::queries::{Report, ReportStatus},
};

#[derive(Default)]
pub struct ReportsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl ReportsMutation {
	#[graphql(guard = "PermissionGuard::one(TicketPermission::Create)")]
	async fn create_report(&self, data: CreateReportInput) -> Result<Report, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	#[graphql(guard = "PermissionGuard::one(TicketPermission::Edit)")]
	async fn edit_report(&self, report_id: TicketObjectId, data: EditReportInput) -> Result<Report, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CreateReportInput {
	target_kind: u32,
	target_id: ObjectId<()>,
	subject: String,
	body: String,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportInput {
	priority: Option<u32>,
	status: Option<ReportStatus>,
	assignee: Option<String>,
	note: Option<EditReportNoteInput>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct EditReportNoteInput {
	timestamp: Option<String>,
	content: Option<String>,
	internal: Option<bool>,
	reply: Option<String>,
}
