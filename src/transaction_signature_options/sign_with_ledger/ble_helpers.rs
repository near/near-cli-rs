//! Shared helpers for Ledger BLE (Bluetooth) operations.
//!
//! A **multi-thread** tokio runtime is required because `btleplug`
//! spawns background tasks that must run concurrently with `block_on`.

const BLE_SCAN_MAX_RETRIES: usize = 3;

pub fn ble_connect_and_get_public_key(
    seed_phrase_hd_path: slipped10::BIP32Path,
) -> color_eyre::eyre::Result<ed25519_dalek::VerifyingKey> {
    let rt = new_ble_runtime()?;

    rt.block_on(async {
        let transport = scan_and_connect().await?;
        open_near_app(&transport).await?;
        get_public_key(&transport, seed_phrase_hd_path).await
    })
}

pub fn ble_get_public_key_and_sign_nep413(
    seed_phrase_hd_path: slipped10::BIP32Path,
    payload: near_ledger::NEP413Payload,
) -> color_eyre::eyre::Result<(ed25519_dalek::VerifyingKey, near_ledger::SignatureBytes)> {
    let rt = new_ble_runtime()?;

    rt.block_on(async {
        let transport = scan_and_connect().await?;
        open_near_app(&transport).await?;
        let public_key = get_public_key(&transport, seed_phrase_hd_path.clone()).await?;

        let signature =
            near_ledger::sign_message_nep413_ble(&transport, &payload, seed_phrase_hd_path)
                .await
                .map_err(|e| {
                    color_eyre::Report::msg(format!(
                        "Error signing NEP-413 message via Bluetooth: {e:?}"
                    ))
                })?;

        Ok((public_key, signature))
    })
}

/// Keeps the tokio runtime alive across multiple BLE operations.
/// Used for the transaction-signing flow where synchronous RPC work
/// happens between get_public_key and sign.
pub struct BleSession {
    rt: tokio::runtime::Runtime,
    transport: near_ledger::TransportBle,
}

impl BleSession {
    pub fn connect_open_and_get_public_key(
        seed_phrase_hd_path: slipped10::BIP32Path,
    ) -> color_eyre::eyre::Result<(Self, ed25519_dalek::VerifyingKey)> {
        let rt = new_ble_runtime()?;

        let (transport, public_key) = rt.block_on(async {
            let transport = scan_and_connect().await?;
            open_near_app(&transport).await?;
            let public_key = get_public_key(&transport, seed_phrase_hd_path).await?;
            Ok::<_, color_eyre::Report>((transport, public_key))
        })?;

        Ok((Self { rt, transport }, public_key))
    }

    pub fn sign_transaction(
        &self,
        unsigned_tx: &[u8],
        seed_phrase_hd_path: slipped10::BIP32Path,
    ) -> Result<near_ledger::SignatureBytes, near_ledger::NEARLedgerError> {
        self.rt.block_on(async {
            near_ledger::sign_transaction_ble(&self.transport, unsigned_tx, seed_phrase_hd_path)
                .await
        })
    }

    pub fn sign_message_nep366_delegate_action(
        &self,
        payload: &[u8],
        seed_phrase_hd_path: slipped10::BIP32Path,
    ) -> Result<near_ledger::SignatureBytes, near_ledger::NEARLedgerError> {
        self.rt.block_on(async {
            near_ledger::sign_message_nep366_delegate_action_ble(
                &self.transport,
                payload,
                seed_phrase_hd_path,
            )
            .await
        })
    }
}

fn new_ble_runtime() -> color_eyre::eyre::Result<tokio::runtime::Runtime> {
    tokio::runtime::Runtime::new()
        .map_err(|e| color_eyre::Report::msg(format!("Failed to create async runtime: {e}")))
}

#[tracing::instrument(name = "Scanning for Ledger devices via Bluetooth ...", skip_all)]
async fn scan_and_connect() -> color_eyre::eyre::Result<near_ledger::TransportBle> {
    let mut last_err = None;
    for attempt in 1..=BLE_SCAN_MAX_RETRIES {
        match near_ledger::TransportBle::new().await {
            Ok(t) => {
                return Ok(t);
            }
            Err(e) => {
                if attempt < BLE_SCAN_MAX_RETRIES {
                    eprintln!(
                        "Ledger not found on scan attempt \
                         {attempt}/{BLE_SCAN_MAX_RETRIES}, retrying..."
                    );
                }
                last_err = Some(e);
            }
        }
    }

    Err(color_eyre::Report::msg(format!(
        "No Ledger device found via Bluetooth after \
         {BLE_SCAN_MAX_RETRIES} scan attempts. Make sure \
         Bluetooth is enabled on your Ledger and it is in \
         range.\nLast error: {}",
        last_err
            .as_ref()
            .map(|e| format!("{e}"))
            .unwrap_or_default()
    )))
}

#[tracing::instrument(
    name = "Opening the NEAR application on the Ledger via Bluetooth ...",
    skip_all
)]
async fn open_near_app(transport: &near_ledger::TransportBle) -> color_eyre::eyre::Result<()> {
    near_ledger::open_near_application_ble(transport)
        .await
        .map_err(|e| {
            color_eyre::Report::msg(format!(
                "An error happened while trying to open the NEAR \
                 application on the Ledger via Bluetooth: {e:?}"
            ))
        })
}

#[tracing::instrument(
    name = "Getting the PublicKey from Ledger device via Bluetooth ...",
    skip_all
)]
async fn get_public_key(
    transport: &near_ledger::TransportBle,
    seed_phrase_hd_path: slipped10::BIP32Path,
) -> color_eyre::eyre::Result<ed25519_dalek::VerifyingKey> {
    near_ledger::get_public_key_ble(transport, seed_phrase_hd_path)
        .await
        .map_err(|e| {
            let hint = if format!("{e:?}").contains("Timeout") {
                "\nHint: The Ledger timed out waiting for \
                 confirmation. Please approve the request on \
                 the device within 30 seconds."
            } else {
                ""
            };
            color_eyre::Report::msg(format!(
                "An error occurred while trying to get PublicKey \
                 from Ledger device via Bluetooth: {e:?}{hint}"
            ))
        })
}
