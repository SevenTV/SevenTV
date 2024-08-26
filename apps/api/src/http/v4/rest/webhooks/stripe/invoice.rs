use std::{str::FromStr, sync::Arc};

use mongodb::options::FindOneAndUpdateOptions;
use shared::database::{
	product::{invoice::{Invoice, InvoiceItem, InvoiceStatus}, InvoiceId},
	queries::{filter, update},
	user::UserId,
};
use stripe::Object;

use crate::{
	global::Global,
	http::error::ApiError,
	transactions::{TransactionError, TransactionResult, TransactionSession},
};

fn invoice_items(items: Option<&stripe::List<stripe::InvoiceLineItem>>) -> Result<Vec<InvoiceItem>, ApiError> {
	// TODO: paginate?
	items
		.ok_or(ApiError::BAD_REQUEST)?
		.data
		.iter()
		.map(|line| {
			let product_id = line.price.as_ref().ok_or(ApiError::BAD_REQUEST)?.id().into();

			Ok(InvoiceItem {
				id: line.id.clone().into(),
				product_id,
				discount_codes: vec![],
			})
		})
		.collect()
}

pub async fn created(
	global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	if let Some(subscription) = invoice.subscription {
		let subscription = stripe::Subscription::retrieve(&global.stripe_client, &subscription.id(), &[])
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to retrieve subscription");
				TransactionError::custom(ApiError::INTERNAL_SERVER_ERROR)
			})?;

		let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

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
	} else {
		// TODO: do something here?
		Ok(())
	}
}

/// Called for `invoice.updated`, `invoice.paid`, `invoice.voided` and `invoice.marked_uncollectible`
pub async fn updated(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	let id: InvoiceId = invoice.id.into();

	let status: InvoiceStatus = invoice.status.ok_or(TransactionError::custom(ApiError::BAD_REQUEST))?.into();

	let items = invoice_items(invoice.lines.as_ref()).map_err(TransactionError::custom)?;

	let invoice = tx
		.find_one_and_update(
			filter::filter! {
				Invoice {
					#[query(rename = "_id")]
					id,
				}
			},
			update::update! {
				#[query(set)]
				Invoice {
					#[query(serde)]
					status: status,
					#[query(serde)]
					items: items,
					updated_at: chrono::Utc::now(),
				}
			},
			FindOneAndUpdateOptions::builder()
				.return_document(mongodb::options::ReturnDocument::After)
				.build(),
		)
		.await?
		.ok_or(TransactionError::custom(ApiError::NOT_FOUND))?;

	if invoice.status == InvoiceStatus::Paid {
		// TODO: if subscription, add periods
	}

	Ok(())
}

pub async fn deleted(
	_global: &Arc<Global>,
	mut tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	let id: InvoiceId = invoice.id.into();

	tx.find_one_and_delete(
		filter::filter! {
			Invoice {
				#[query(rename = "_id")]
				id,
			}
		},
		None,
	)
	.await?;

	Ok(())
}

pub async fn payment_failed(
	global: &Arc<Global>,
	tx: TransactionSession<'_, ApiError>,
	invoice: stripe::Invoice,
) -> TransactionResult<(), ApiError> {
	Err(TransactionError::custom(ApiError::NOT_IMPLEMENTED))
}
