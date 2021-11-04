/*******************************************************************************
*   (c) 2018 Zondax GmbH
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
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerHIDError {
    /// Device not found error
    #[error("Ledger device not found")]
    DeviceNotFound,
    /// Communication error
    #[error("Ledger device: communication error `{0}`")]
    Comm(&'static str),
    /// Ioctl error
    #[error("Ledger device: Ioctl error")]
    Ioctl(#[from] crate::nix::Error),
    /// i/o error
    #[error("Ledger device: i/o error")]
    Io(#[from] std::io::Error),
    /// HID error
    #[error("Ledger device: Io error")]
    Hid(#[from] hidapi::HidError),
    /// UT8F error
    #[error("Ledger device: UTF8 error")]
    UTF8(#[from] std::str::Utf8Error),
}
