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

#[derive(Debug)]
pub struct APDUCommand {
    pub cla: u8,
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct APDUAnswer {
    pub data: Vec<u8>,
    pub retcode: u16,
}

impl APDUCommand {
    pub fn serialize(&self) -> Vec<u8> {
        let mut v = vec![self.cla, self.ins, self.p1, self.p2, self.data.len() as u8];
        v.extend(&self.data);
        v
    }
}

impl APDUAnswer {
    pub fn from_answer(answer: Vec<u8>) -> APDUAnswer {
        let apdu_retcode =
            (u16::from(answer[answer.len() - 2]) << 8) + u16::from(answer[answer.len() - 1]);
        let apdu_data = &answer[..answer.len() - 2];

        APDUAnswer {
            data: apdu_data.to_vec(),
            retcode: apdu_retcode,
        }
    }
}

#[derive(Copy, Clone)]
pub enum APDUErrorCodes {
    NoError = 0x9000,
    ExecutionError = 0x6400,
    WrongLength = 0x6700,
    EmptyBuffer = 0x6982,
    OutputBufferTooSmall = 0x6983,
    DataInvalid = 0x6984,
    ConditionsNotSatisfied = 0x6985,
    CommandNotAllowed = 0x6986,
    BadKeyHandle = 0x6A80,
    InvalidP1P2 = 0x6B00,
    InsNotSupported = 0x6D00,
    ClaNotSupported = 0x6E00,
    Unknown = 0x6F00,
    SignVerifyError = 0x6F01,
}

pub fn map_apdu_error_description(retcode: u16) -> &'static str {
    match retcode {
        0x6400 => "APDU_CODE_EXECUTION_ERROR - No information given (NV-Ram not changed)",
        0x6700 => "APDU_CODE_WRONG_LENGTH - Wrong length",
        0x6982 => "APDU_CODE_EMPTY_BUFFER",
        0x6983 => "APDU_CODE_OUTPUT_BUFFER_TOO_SMALL - ",
        0x6984 => "APDU_CODE_DATA_INVALID - data reversibly blocked (invalidated)",
        0x6985 => "APDU_CODE_CONDITIONS_NOT_SATISFIED - Conditions of use not satisfied",
        0x6986 => "APDU_CODE_COMMAND_NOT_ALLOWED - Command not allowed (no current EF)",
        0x6A80 => "APDU_CODE_BAD_KEY_HANDLE - The parameters in the data field are incorrect",
        0x6B00 => "APDU_CODE_INVALIDP1P2 - Wrong parameter(s) P1-P2",
        0x6D00 => "APDU_CODE_INS_NOT_SUPPORTED - Instruction code not supported or invalid",
        0x6E00 => "APDU_CODE_CLA_NOT_SUPPORTED - Class not supported",
        0x6F00 => "APDU_CODE_UNKNOWN - ",
        0x6F01 => "APDU_CODE_SIGN_VERIFY_ERROR - ",
        _ => "[APDU_ERROR] Unknown",
    }
}
