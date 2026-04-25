//! PBEM envelope signing and host-side collection.
#![forbid(unsafe_code)]

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderEnvelope {
    pub player_id: String,
    pub game_id: String,
    pub turn: u32,
    /// Canonical JSON of `Vec<Order>`.
    pub orders_json: String,
    /// Ed25519 signature over `player_id:game_id:turn:orders_json`.
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PbemHost {
    pub game_id: String,
    pub turn: u32,
    pub envelopes: Vec<OrderEnvelope>,
}

pub fn sign_envelope(
    signing_key: &SigningKey,
    player_id: &str,
    game_id: &str,
    turn: u32,
    orders_json: &str,
) -> OrderEnvelope {
    let message = envelope_message(player_id, game_id, turn, orders_json);
    let signature = signing_key.sign(message.as_bytes()).to_bytes().to_vec();

    OrderEnvelope {
        player_id: player_id.to_owned(),
        game_id: game_id.to_owned(),
        turn,
        orders_json: orders_json.to_owned(),
        signature,
    }
}

pub fn verify_envelope(verifying_key: &VerifyingKey, envelope: &OrderEnvelope) -> bool {
    let Ok(signature) = Signature::try_from(envelope.signature.as_slice()) else {
        return false;
    };

    let message = envelope_message(
        &envelope.player_id,
        &envelope.game_id,
        envelope.turn,
        &envelope.orders_json,
    );

    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}

pub fn collect_envelope(
    host: &mut PbemHost,
    envelope: OrderEnvelope,
    verifying_key: &VerifyingKey,
) -> Result<(), String> {
    if host.game_id != envelope.game_id {
        return Err(format!(
            "game id mismatch: host `{}` envelope `{}`",
            host.game_id, envelope.game_id
        ));
    }

    if host.turn != envelope.turn {
        return Err(format!(
            "turn mismatch: host turn {} envelope turn {}",
            host.turn, envelope.turn
        ));
    }

    if !verify_envelope(verifying_key, &envelope) {
        return Err("invalid envelope signature".to_owned());
    }

    if host
        .envelopes
        .iter()
        .any(|existing| existing.player_id == envelope.player_id)
    {
        return Err(format!(
            "duplicate envelope for player `{}`",
            envelope.player_id
        ));
    }

    host.envelopes.push(envelope);
    Ok(())
}

pub fn all_envelopes_received(host: &PbemHost, expected_players: &[String]) -> bool {
    expected_players.iter().all(|expected| {
        host.envelopes
            .iter()
            .any(|envelope| envelope.player_id == *expected)
    })
}

fn envelope_message(player_id: &str, game_id: &str, turn: u32, orders_json: &str) -> String {
    format!("{}:{}:{}:{}", player_id, game_id, turn, orders_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    use gc1805_core_schema::canonical::to_canonical_string;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestOrder {
        kind: String,
        corps: String,
        target: String,
    }

    fn signing_key(byte: u8) -> SigningKey {
        SigningKey::from_bytes(&[byte; 32])
    }

    fn canonical_orders_json() -> String {
        let orders = vec![
            TestOrder {
                kind: "MOVE".to_owned(),
                corps: "CORPS_FRA_001".to_owned(),
                target: "AREA_ULM".to_owned(),
            },
            TestOrder {
                kind: "HOLD".to_owned(),
                corps: "CORPS_AUS_001".to_owned(),
                target: "AREA_VIENNA".to_owned(),
            },
        ];

        to_canonical_string(&orders).expect("canonical orders json")
    }

    fn sample_envelope() -> OrderEnvelope {
        let key = signing_key(7);
        sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json())
    }

    fn sample_host() -> PbemHost {
        PbemHost {
            game_id: "gc1805-test".to_owned(),
            turn: 3,
            envelopes: Vec::new(),
        }
    }

    #[test]
    fn sign_and_verify_valid() {
        let key = signing_key(1);
        let envelope = sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json());

        assert!(verify_envelope(&key.verifying_key(), &envelope));
    }

    #[test]
    fn verify_tampered_orders_fails() {
        let key = signing_key(2);
        let mut envelope = sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json());
        envelope.orders_json = "[{\"kind\":\"MOVE\",\"target\":\"AREA_PARIS\"}]".to_owned();

        assert!(!verify_envelope(&key.verifying_key(), &envelope));
    }

    #[test]
    fn verify_wrong_key_fails() {
        let correct_key = signing_key(3);
        let wrong_key = signing_key(4);
        let envelope = sign_envelope(
            &correct_key,
            "FRA",
            "gc1805-test",
            3,
            &canonical_orders_json(),
        );

        assert!(!verify_envelope(&wrong_key.verifying_key(), &envelope));
    }

    #[test]
    fn collect_valid_envelope_ok() {
        let key = signing_key(5);
        let envelope = sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json());
        let mut host = sample_host();

        let result = collect_envelope(&mut host, envelope.clone(), &key.verifying_key());

        assert_eq!(result, Ok(()));
        assert_eq!(host.envelopes, vec![envelope]);
    }

    #[test]
    fn collect_wrong_turn_rejected() {
        let key = signing_key(6);
        let envelope = sign_envelope(&key, "FRA", "gc1805-test", 4, &canonical_orders_json());
        let mut host = sample_host();

        let result = collect_envelope(&mut host, envelope, &key.verifying_key());

        assert_eq!(
            result,
            Err("turn mismatch: host turn 3 envelope turn 4".to_owned())
        );
        assert!(host.envelopes.is_empty());
    }

    #[test]
    fn collect_duplicate_player_rejected() {
        let key = signing_key(8);
        let envelope = sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json());
        let mut host = sample_host();
        collect_envelope(&mut host, envelope.clone(), &key.verifying_key()).unwrap();

        let result = collect_envelope(&mut host, envelope, &key.verifying_key());

        assert_eq!(
            result,
            Err("duplicate envelope for player `FRA`".to_owned())
        );
        assert_eq!(host.envelopes.len(), 1);
    }

    #[test]
    fn collect_invalid_signature_rejected() {
        let key = signing_key(9);
        let mut envelope = sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json());
        envelope.signature[0] ^= 0xFF;
        let mut host = sample_host();

        let result = collect_envelope(&mut host, envelope, &key.verifying_key());

        assert_eq!(result, Err("invalid envelope signature".to_owned()));
        assert!(host.envelopes.is_empty());
    }

    #[test]
    fn all_received_false_when_missing_player() {
        let mut host = sample_host();
        host.envelopes.push(sample_envelope());
        let expected = vec!["FRA".to_owned(), "AUS".to_owned()];

        assert!(!all_envelopes_received(&host, &expected));
    }

    #[test]
    fn all_received_true_when_all_present() {
        let fra_key = signing_key(10);
        let aus_key = signing_key(11);
        let mut host = sample_host();
        host.envelopes.push(sign_envelope(
            &fra_key,
            "FRA",
            "gc1805-test",
            3,
            &canonical_orders_json(),
        ));
        host.envelopes.push(sign_envelope(
            &aus_key,
            "AUS",
            "gc1805-test",
            3,
            &canonical_orders_json(),
        ));
        let expected = vec!["FRA".to_owned(), "AUS".to_owned()];

        assert!(all_envelopes_received(&host, &expected));
    }

    #[test]
    fn envelope_serializes_to_json() {
        let envelope = sample_envelope();

        let json = serde_json::to_string(&envelope).unwrap();

        assert!(json.contains("\"player_id\":\"FRA\""));
        assert!(json.contains("\"game_id\":\"gc1805-test\""));
        assert!(json.contains("\"turn\":3"));
        assert!(json.contains("\"orders_json\":"));
        assert!(json.contains("\"signature\":"));
    }

    #[test]
    fn envelope_deserializes_from_json() {
        let envelope = sample_envelope();
        let json = serde_json::to_string(&envelope).unwrap();

        let parsed: OrderEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, envelope);
    }

    #[test]
    fn sign_deterministic_same_key_same_message() {
        let key = signing_key(12);
        let orders_json = canonical_orders_json();

        let first = sign_envelope(&key, "FRA", "gc1805-test", 3, &orders_json);
        let second = sign_envelope(&key, "FRA", "gc1805-test", 3, &orders_json);

        assert_eq!(first, second);
    }

    #[test]
    fn verify_tampered_player_id_fails() {
        let key = signing_key(13);
        let mut envelope = sign_envelope(&key, "FRA", "gc1805-test", 3, &canonical_orders_json());
        envelope.player_id = "AUS".to_owned();

        assert!(!verify_envelope(&key.verifying_key(), &envelope));
    }

    #[test]
    fn verify_invalid_signature_length_fails() {
        let mut envelope = sample_envelope();
        envelope.signature.pop();
        let key = signing_key(7);

        assert!(!verify_envelope(&key.verifying_key(), &envelope));
    }

    #[test]
    fn collect_wrong_game_id_rejected() {
        let key = signing_key(14);
        let envelope = sign_envelope(&key, "FRA", "another-game", 3, &canonical_orders_json());
        let mut host = sample_host();

        let result = collect_envelope(&mut host, envelope, &key.verifying_key());

        assert_eq!(
            result,
            Err("game id mismatch: host `gc1805-test` envelope `another-game`".to_owned())
        );
        assert!(host.envelopes.is_empty());
    }

    #[test]
    fn all_received_ignores_submission_order() {
        let fra_key = signing_key(15);
        let rus_key = signing_key(16);
        let mut host = sample_host();
        host.envelopes.push(sign_envelope(
            &rus_key,
            "RUS",
            "gc1805-test",
            3,
            &canonical_orders_json(),
        ));
        host.envelopes.push(sign_envelope(
            &fra_key,
            "FRA",
            "gc1805-test",
            3,
            &canonical_orders_json(),
        ));
        let expected = vec!["FRA".to_owned(), "RUS".to_owned()];

        assert!(all_envelopes_received(&host, &expected));
    }

    #[test]
    fn all_received_empty_expected_players_is_true() {
        let host = sample_host();

        assert!(all_envelopes_received(&host, &[]));
    }
}
