use crate::Did;
use blake3::Hasher;

// Strategy: Use 6-7 words to get enough combinations while staying pronounceable
// 32 words per position × 6 positions = 32^6 = 1,073,741,824 combinations (1 billion)
// 32 words per position × 7 positions = 32^7 = 34,359,738,368 combinations (34 billion)

/// Adjectives (32 words) - describing qualities
const ADJECTIVES: &[&str] = &[
    "ancient", "bright", "cosmic", "divine", "eternal", "fierce", "golden", "hidden",
    "infinite", "joyful", "kindled", "lunar", "mystic", "noble", "oceanic", "primal",
    "quantum", "radiant", "sacred", "timeless", "unified", "valiant", "whispering", "xeric",
    "yearning", "zealous", "astral", "blazing", "crystal", "dancing", "ethereal", "frozen",
];

/// Colors/Materials (32 words) - visual/material qualities
const MATERIALS: &[&str] = &[
    "amber", "bronze", "cobalt", "diamond", "emerald", "flame", "granite", "helium",
    "iron", "jasper", "kunzite", "lapis", "marble", "neon", "onyx", "pearl",
    "quartz", "ruby", "silver", "topaz", "uranium", "violet", "willow", "xenon",
    "yellow", "zinc", "arctic", "basalt", "copper", "dusk", "ebony", "frost",
];

/// Creatures/Beings (32 words) - living entities
const CREATURES: &[&str] = &[
    "angel", "bear", "crow", "dragon", "eagle", "falcon", "griffin", "hawk",
    "ibis", "jaguar", "kraken", "lion", "mantis", "nomad", "owl", "phoenix",
    "quail", "raven", "sphinx", "tiger", "unicorn", "viper", "wolf", "xerus",
    "yak", "zebra", "ape", "bat", "cat", "deer", "elk", "fox",
];

/// Actions/Verbs (32 words) - dynamic words
const ACTIONS: &[&str] = &[
    "ascending", "blazing", "cascading", "dancing", "echoing", "flowing", "gliding", "howling",
    "igniting", "jumping", "kindling", "leaping", "morphing", "navigating", "orbiting", "pulsing",
    "questing", "rising", "soaring", "turning", "unfolding", "voyaging", "waking", "xeroxing",
    "yielding", "zoning", "arcing", "burning", "casting", "diving", "emerging", "flying",
];

/// Concepts/Abstract (32 words) - abstract ideas
const CONCEPTS: &[&str] = &[
    "abyss", "beacon", "cosmos", "dream", "echo", "fate", "glory", "haven",
    "infinity", "journey", "key", "light", "mirror", "nexus", "oracle", "prism",
    "quest", "realm", "storm", "truth", "unity", "void", "wisdom", "axis",
    "youth", "zenith", "alpha", "bridge", "crown", "dawn", "edge", "forge",
];

/// Elements/Nature (32 words) - natural phenomena
const ELEMENTS: &[&str] = &[
    "aurora", "blizzard", "cloud", "desert", "eclipse", "forest", "glacier", "horizon",
    "island", "jungle", "kelp", "lightning", "mountain", "nebula", "ocean", "prairie",
    "quasar", "river", "star", "tundra", "universe", "volcano", "wind", "xylem",
    "yard", "zone", "air", "breeze", "canyon", "delta", "earth", "fire",
];

/// Generate a human-readable username from a DID
///
/// Format: 6 words from different categories
/// Example: "ancient-amber-phoenix-soaring-cosmos-aurora"
/// This gives us 32^6 = 1,073,741,824 unique combinations (over 1 billion)
pub fn generate_username(did: &Did) -> String {
    // Hash the DID to get deterministic randomness
    let mut hasher = Hasher::new();
    hasher.update(did.to_string().as_bytes());
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();
    
    // Use hash bytes to select words (using modulo to map to our 32-word arrays)
    let indices = [
        hash_bytes[0] as usize % 32,
        hash_bytes[1] as usize % 32,
        hash_bytes[2] as usize % 32,
        hash_bytes[3] as usize % 32,
        hash_bytes[4] as usize % 32,
        hash_bytes[5] as usize % 32,
    ];
    
    format!("{}-{}-{}-{}-{}-{}", 
        ADJECTIVES[indices[0]],
        MATERIALS[indices[1]],
        CREATURES[indices[2]],
        ACTIONS[indices[3]],
        CONCEPTS[indices[4]],
        ELEMENTS[indices[5]]
    )
}

/// Parse a username to extract its components
pub fn parse_username(username: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = username.split('-').collect();
    if parts.len() != 6 {
        return None;
    }
    
    Some(parts.iter().map(|s| s.to_string()).collect())
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
        
        // Should match format: 6 words separated by hyphens
        let parts: Vec<&str> = username.split('-').collect();
        assert_eq!(parts.len(), 6);
        
        // Verify each part is a valid word from the respective category
        assert!(ADJECTIVES.contains(&parts[0]));
        assert!(MATERIALS.contains(&parts[1]));
        assert!(CREATURES.contains(&parts[2]));
        assert!(ACTIONS.contains(&parts[3]));
        assert!(CONCEPTS.contains(&parts[4]));
        assert!(ELEMENTS.contains(&parts[5]));
    }
    
    #[test]
    fn test_username_parsing() {
        let username = "ancient-amber-phoenix-soaring-cosmos-aurora";
        let parsed = parse_username(username);
        assert!(parsed.is_some());
        
        let parts = parsed.unwrap();
        assert_eq!(parts.len(), 6);
        assert_eq!(parts[0], "ancient");
        assert_eq!(parts[1], "amber");
        assert_eq!(parts[2], "phoenix");
        assert_eq!(parts[3], "soaring");
        assert_eq!(parts[4], "cosmos");
        assert_eq!(parts[5], "aurora");
    }
}