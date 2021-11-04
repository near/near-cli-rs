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

// #![deny(warnings, trivial_casts, trivial_numeric_casts)]
// #![deny(unused_import_braces, unused_qualifications)]
// #![deny(missing_docs)]
// #![doc(html_root_url = "https://docs.rs/ledger-filecoin/0.1.0")]

use crate::errors::TransportError;
use ledger_apdu::{APDUAnswer, APDUCommand};

use js_sys;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

/// Trait for any APDU transport
pub trait TransportWrapperTrait {
    /// Send an APDU command and receive a promise of a response back
    fn exchange_apdu(&self, apdu_command: &[u8]) -> js_sys::Promise;
}

/// Transport struct for non-wasm arch
pub struct APDUTransport {
    /// Contain javascript transport object
    pub transport_wrapper: Box<dyn TransportWrapperTrait>,
}

/// Transport Impl for wasm
impl APDUTransport {
    /// Use to talk to the ledger device
    pub async fn exchange(&self, apdu_command: &APDUCommand) -> Result<APDUAnswer, TransportError> {
        let promise = self
            .transport_wrapper
            .exchange_apdu(&apdu_command.serialize());

        let future = JsFuture::from(promise);
        let result = future.await.map_err(|e| {
            // All javascript Error are based on this model : https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error
            // Will work also for Ledger specific Transport error : https://github.com/LedgerHQ/ledgerjs/blob/master/packages/errors/src/index.ts#L228
            // Also TransportStatus Error https://github.com/LedgerHQ/ledgerjs/blob/master/packages/errors/src/index.ts#L300
            // Which should cover most it of our need
            if js_sys::Reflect::has(&e, &JsValue::from_str("message")).unwrap()
                && js_sys::Reflect::has(&e, &JsValue::from_str("name")).unwrap()
            {
                let error_message =
                    js_sys::Reflect::get(&e, &JsValue::from_str("message")).unwrap();
                let error_name = js_sys::Reflect::get(&e, &JsValue::from_str("name")).unwrap();
                return TransportError::JavascriptError(
                    error_name.as_string().unwrap(),
                    error_message.as_string().unwrap(),
                );
            }

            return TransportError::UnknownError;
        })?;
        let answer = js_sys::Uint8Array::new(&result).to_vec();

        // if the reply is < 2 bytes, this is a serious error
        if answer.len() < 2 {
            return Err(TransportError::ResponseTooShort);
        }

        Ok(APDUAnswer::from_answer(answer))
    }
}
