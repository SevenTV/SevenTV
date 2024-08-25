use std::{str::FromStr, sync::Arc};

use axum::http::StatusCode;
use shared::database::{
	product::invoice::{Invoice, InvoiceItem},
	user::UserId,
};
use stripe::Object;

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{with_transaction, TransactionError},
};

pub async fn created(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	if let Some(subscription) = invoice.subscription {
		let subscription = stripe::Subscription::retrieve(&global.stripe_client, &subscription.id(), &[])
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		let res = with_transaction(global, |mut tx| async move {
			// TODO: paginate
			let items = invoice
				.lines
				.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?
				.data
				.into_iter()
				.map(|line| {
					let product_id = line.price.ok_or(ApiError::BAD_REQUEST)?.id().into();

					Ok(InvoiceItem {
						id: line.id.into(),
						product_id,
						discount_codes: vec![],
					})
				})
				.collect::<Result<_, ApiError>>()
				.map_err(TransactionError::custom)?;

			let customer_id = invoice
				.customer
				.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?
				.id()
				.into();

			let user_id = subscription
				.metadata
				.get("USER_ID")
				.and_then(|i| UserId::from_str(i).ok())
				.ok_or(TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR))?;

			let status = invoice.status.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.into();

			let created_at = invoice
				.created
				.and_then(|t| chrono::DateTime::from_timestamp(t, 0))
				.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?;

			let invoice = Invoice {
				id: invoice.id.into(),
				items,
				customer_id,
				user_id,
				paypal_payment_ids: vec![],
				status,
				note: None,
				created_at,
				updated_at: created_at,
				search_updated_at: None,
			};

			tx.insert_one(invoice, None).await?;

			Ok(())
		})
		.await;

		match res {
			Ok(_) => Ok(StatusCode::OK),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::INTERNAL_SERVER_ERROR)
			}
		}
	} else {
		// TODO: do something here?
		Ok(StatusCode::OK)
	}
}

pub async fn updated(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn deleted(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn paid(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn payment_failed(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn voided(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn marked_uncollectible(global: &Arc<Global>, invoice: stripe::Invoice) -> Result<StatusCode, ApiError> {
	Ok(StatusCode::NOT_IMPLEMENTED)
}
