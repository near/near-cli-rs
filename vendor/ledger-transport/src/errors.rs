/*******************************************************************************
*   (c) 2020 Zondax GmbH
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Transport Error
#[derive(Clone, Debug, Eq, Error, PartialEq, Deserialize, Serialize)]
pub enum TransportError {
    /// Transport specific error
    #[error("APDU Exchange Error")]
    APDUExchangeError,
    /// Response was too short (< 2 bytes)
    #[error("APDU Response was too short")]
    ResponseTooShort,
    /// Javscript error
    #[error("Javascript error : `[{0}] {1}`")]
    JavascriptError(String, String),
    /// Error Unknown
    #[error("Unknown Error")]
    UnknownError,
}
