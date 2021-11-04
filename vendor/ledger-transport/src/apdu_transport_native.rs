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
//! Support library for Filecoin Ledger Nano S/X apps

#![deny(warnings, trivial_casts, trivial_numeric_casts)]
#![deny(unused_import_braces, unused_qualifications)]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/ledger-filecoin/0.1.0")]

use crate::errors::TransportError;
use crate::Exchange;
use ledger_apdu::{APDUAnswer, APDUCommand};

/// Transport struct for non-wasm arch
pub struct APDUTransport {
    /// Native rust transport
    pub transport_wrapper: Box<dyn Exchange>,
}

impl APDUTransport {
    /// Creates a native rust transport
    pub fn new(wrapper: impl Exchange + 'static) -> Self {
        Self {
            transport_wrapper: Box::new(wrapper),
        }
    }

    /// Use to talk to the ledger device
    pub async fn exchange(&self, command: &APDUCommand) -> Result<APDUAnswer, TransportError> {
        self.transport_wrapper.exchange(command).await
    }
}
