use crate::Did;
use blake3::Hasher;

/// Adjectives for username generation
const ADJECTIVES: &[&str] = &[
    "brave", "bright", "calm", "clever", "cool",
    "eager", "fair", "gentle", "happy", "kind",
    "lovely", "merry", "nice", "proud", "quick",
    "sharp", "smart", "swift", "warm", "wise",
    "amber", "azure", "coral", "crystal", "emerald",
    "golden", "indigo", "jade", "pearl", "ruby",
    "silver", "violet", "cosmic", "lunar", "solar",
    "stellar", "astral", "ethereal", "mystic", "quantum",
];

/// Nouns for username generation
const NOUNS: &[&str] = &[
    "moon", "star", "sun", "sky", "cloud",
    "river", "ocean", "mountain", "valley", "forest",
    "phoenix", "dragon", "wolf", "eagle", "falcon",
    "crystal", "diamond", "prism", "comet", "nebula",
    "horizon", "aurora", "galaxy", "cosmos", "nova",
    "beacon", "harbor", "bridge", "garden", "sanctuary",
    "echo", "harmony", "melody", "rhythm", "symphony",
    "spirit", "essence", "aura", "energy", "light",
];

/// Generate a human-readable username from a DID
///
/// Format: adjective-noun-number (e.g., "cosmic-phoenix-42")
pub fn generate_username(did: &Did) -> String {
    // Hash the DID to get deterministic randomness
    let mut hasher = Hasher::new();
    hasher.update(did.to_string().as_bytes());
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    
    // Use hash bytes to select words
    let adj_index = u16::from_le_bytes([hash_bytes[0], hash_bytes[1]]) as usize % ADJECTIVES.len();
    let noun_index = u16::from_le_bytes([hash_bytes[2], hash_bytes[3]]) as usize % NOUNS.len();
    let number = u16::from_le_bytes([hash_bytes[4], hash_bytes[5]]) % 1000;
    
    format!("{}-{}-{}", ADJECTIVES[adj_index], NOUNS[noun_index], number)
}

/// Parse a username back to find matching DID (requires lookup in storage)
pub fn parse_username(username: &str) -> Option<(String, String, u16)> {
    let parts: Vec<&str> = username.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    
    let number = parts[2].parse().ok()?;
    Some((parts[0].to_string(), parts[1].to_string(), number))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_username_generation() {
        let did = Did::parse("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").unwrap();
        let username = generate_username(&did);
        
        // Should be deterministic
        let username2 = generate_username(&did);
        assert_eq!(username, username2);
        
        // Should match format
        let parts: Vec<&str> = username.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[2].parse::<u16>().is_ok());
    }
    
    #[test]
    fn test_username_parsing() {
        let username = "cosmic-phoenix-42";
        let parsed = parse_username(username);
        assert!(parsed.is_some());
        
        let (adj, noun, num) = parsed.unwrap();
        assert_eq!(adj, "cosmic");
        assert_eq!(noun, "phoenix");
        assert_eq!(num, 42);
    }
}