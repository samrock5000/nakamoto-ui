use nakamoto_client::handle;
use thiserror::Error;

/// An error occuring in the wallet.
#[derive(Error, Debug)]
pub enum Error {
    #[error("client handle error: {0}")]
    Handle(#[from] handle::Error),
    #[error("client error: {0}")]
    Client(#[from] nakamoto_client::Error),
    #[error("channel error: {0}")]
    Channel(#[from] crossbeam_channel::RecvError),
}
