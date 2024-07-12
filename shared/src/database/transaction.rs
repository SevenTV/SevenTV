use mongodb::{error::{Error, TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT}, Client, ClientSession};

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'a>>;

pub async fn with_transaction<F, T>(client: &Client, f: F) -> Result<T, Error>
where
    for<'a> F: Fn(&'a mut ClientSession) -> BoxFuture<'a, Result<T, Error>>,
    // F: Fn(&mut ClientSession) -> Fut,
    // Fut: std::future::Future<Output = Result<T, Error>>,
{
    let mut session = client.start_session().await?;
    session.start_transaction().await?;

    'retry_operation:
    loop {
        let result = f(&mut session).await;
        match &result {
            Ok(_) => {
                'retry_commit:
                loop {
                    match session.commit_transaction().await {
                        Ok(_) => return result,
                        Err(err) => {
                            if err.contains_label(UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                                continue 'retry_commit;
                            } else if err.contains_label(TRANSIENT_TRANSACTION_ERROR) {
                                continue 'retry_operation;
                            }

                            return Err(err);
                        }
                    }
                }
            },
            Err(err) => {
                if err.contains_label(TRANSIENT_TRANSACTION_ERROR) {
                    continue 'retry_operation;
                }

                session.abort_transaction().await?;
                return result;
            }
        }
    }
}
