use serde::{Deserialize, Serialize};
use std::fmt;

/// Decentralized Identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(String);

/// DID methods supported by Bits
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DidMethod {
    Key,  // did:key - self-contained, no blockchain needed
    Bits, // did:bits - future on-chain method
}

impl Did {
    /// Create a did:key from a public key
    pub fn from_key(public_key: &crate::PublicKey) -> Self {
        let multibase = Self::encode_key(public_key);
        Did(format!("did:key:{}", multibase))
    }

    /// Parse a DID string
    pub fn parse(s: &str) -> Result<Self, DidError> {
        if !s.starts_with("did:") {
            return Err(DidError::InvalidFormat);
        }

        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 3 {
            return Err(DidError::InvalidFormat);
        }

        match parts[1] {
            "key" => Ok(Did(s.to_string())),
            "bits" => Ok(Did(s.to_string())),
            _ => Err(DidError::UnsupportedMethod),
        }
    }

    /// Get the method of this DID
    pub fn method(&self) -> DidMethod {
        if self.0.starts_with("did:key:") {
            DidMethod::Key
        } else if self.0.starts_with("did:bits:") {
            DidMethod::Bits
        } else {
            panic!("Invalid DID method")
        }
    }

    /// Encode a public key for did:key
    fn encode_key(public_key: &crate::PublicKey) -> String {
        // Ed25519 public key with multicodec prefix
        let mut bytes = vec![0xed, 0x01]; // ed25519-pub multicodec
        bytes.extend_from_slice(public_key.as_bytes());

        // Multibase encode with 'z' prefix (base58btc)
        format!("z{}", bs58::encode(bytes).into_string())
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// DID Document - the core identity metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: Did,
    pub verification_method: Vec<VerificationMethod>,
    #[serde(default)]
    pub authentication: Vec<String>,
    #[serde(default)]
    pub assertion_method: Vec<String>,
    #[serde(default)]
    pub key_agreement: Vec<String>,
    #[serde(default)]
    pub service: Vec<Service>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

/// Verification method in DID document
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    pub r#type: String,
    pub controller: Did,
    pub public_key_multibase: String,
}

/// Service endpoint in DID document
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub r#type: String,
    pub service_endpoint: String,
}

impl DidDocument {
    /// Create a new DID document
    pub fn new(did: Did) -> Self {
        Self {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            id: did,
            verification_method: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            service: Vec::new(),
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
        }
    }

    /// Add a verification method
    pub fn with_verification_method(mut self, key: crate::PublicKey, purpose: &str) -> Self {
        let id = format!("{}#{}", self.id, purpose);
        let vm = VerificationMethod {
            id: id.clone(),
            r#type: "Ed25519VerificationKey2020".to_string(),
            controller: self.id.clone(),
            public_key_multibase: format!("z{}", bs58::encode(key.as_bytes()).into_string()),
        };
        self.verification_method.push(vm);
        self
    }

    /// Add authentication method
    pub fn with_authentication(mut self, key_ref: &str) -> Self {
        self.authentication.push(format!("{}#{}", self.id, key_ref));
        self
    }

    /// Add assertion method
    pub fn with_assertion_method(mut self, key_ref: &str) -> Self {
        self.assertion_method
            .push(format!("{}#{}", self.id, key_ref));
        self
    }

    /// Add a service endpoint
    pub fn with_service(mut self, service_type: &str, endpoint: &str) -> Self {
        let service = Service {
            id: format!("{}#{}", self.id, service_type),
            r#type: service_type.to_string(),
            service_endpoint: endpoint.to_string(),
        };
        self.service.push(service);
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DidError {
    #[error("Invalid DID format")]
    InvalidFormat,
    #[error("Unsupported DID method")]
    UnsupportedMethod,
    #[error("Key decode error")]
    KeyDecodeError,
}
