use std::sync::Arc;

use mongodb::{error::{Error, TRANSIENT_TRANSACTION_ERROR, UNKNOWN_TRANSACTION_COMMIT_RESULT}, Client, ClientSession};
use spin::{Mutex, MutexGuard};

#[derive(Clone)]
pub struct SessionHolder {
    session: Arc<Mutex<ClientSession>>,
}

pub struct SessionHolderRef<'a>(MutexGuard<'a, ClientSession>);

impl std::ops::Deref for SessionHolderRef<'_> {
    type Target = ClientSession;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SessionHolderRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<ClientSession> for SessionHolderRef<'_> {
    fn as_ref(&self) -> &ClientSession {
        self
    }
}

impl AsMut<ClientSession> for SessionHolderRef<'_> {
    fn as_mut(&mut self) -> &mut ClientSession {
        self
    }
}

impl SessionHolder {
    pub fn get(&mut self) -> SessionHolderRef<'_> {
        SessionHolderRef(
            self.session
                .try_lock()
                .expect("Session is already borrowed"),
        )
    }
}

pub trait ClientExt {
    fn with_transaction<'a, T, F, Fut>(
        &self,
        f: F,
    ) -> impl std::future::Future<Output = Result<T, Error>>
    where
        F: Fn(SessionHolder) -> Fut + 'a,
        Fut: std::future::Future<Output = Result<T, Error>> + 'a;
}

impl ClientExt for Client {
    fn with_transaction<'a, T, F, Fut>(
        &self,
        f: F,
    ) -> impl std::future::Future<Output = Result<T, Error>>
    where
        F: Fn(SessionHolder) -> Fut + 'a,
        Fut: std::future::Future<Output = Result<T, Error>> + 'a,
    {
        async move {
            let mut session = self.start_session().await?;
            session.start_transaction().await?;

            let mut holder = SessionHolder {
                session: Arc::new(Mutex::new(session)),
            };

            'retry_operation: loop {
                let result = f(holder.clone()).await;
                let mut session = holder.get();
                match &result {
                    Ok(_) => 'retry_commit: loop {
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
    }
}
