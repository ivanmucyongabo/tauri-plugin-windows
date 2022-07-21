//! Error.
//!
//! This module contains basic implemention of error types and methods.

use std::{
    io::Error as IoError,
    result::Result as StdResult,
};

use serde_json::Error as SerdeJsonError;

use tauri::{
    api::Error as TauriApiError,
    Error as TauriError
};

/// Errors that can happen during window management.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Cached window state not found with label
  #[error("a window state with label `{0}` doesn't exist")]
  WindowStateWithLabelNotFound(String),
  /// Managed state[all are rwlock] error
  #[error("{0}")]
  RwLock(String),
  /// Failed doing io on files for window state backup
  #[error(transparent)]
  Io(#[from] IoError),
  /// Tauri specific errors
  #[error(transparent)]
  Tauri(#[from] TauriError),
  /// Tauri api specific errors
  #[error(transparent)]
  TauriApi(#[from] TauriApiError),
  #[error(transparent)]
  SerdeJson(#[from] SerdeJsonError)
}

impl Error {
  #[allow(dead_code)]
  pub(crate) fn into_anyhow(self) -> anyhow::Error {
    anyhow::anyhow!(self.to_string())
  }
}