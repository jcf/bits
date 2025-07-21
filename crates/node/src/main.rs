use bits_identity::IdentityService;
use bits_core::Component;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 Bits Identity Demo\n");
    
    // Create identity service with local storage
    let mut identity_service = IdentityService::new("./data").await?;
    
    // Start the service (loads or creates identity)
    identity_service.start().await?;
    
    // Get current identity
    let identity = identity_service.current().await?;
    println!("✅ Identity loaded/created");
    println!("📋 DID: {}", identity.did);
    println!("🕐 Created: {}", identity.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    // Display keys info
    println!("\n🔑 Keys:");
    println!("  - Master key: {:?}", hex::encode(&identity.keys.master.public.as_bytes()[..8]));
    println!("  - Signing key: {:?}", hex::encode(&identity.keys.signing.public.as_bytes()[..8]));
    println!("  - Auth key: {:?}", hex::encode(&identity.keys.authentication.public.as_bytes()[..8]));
    
    // Sign a message
    let message = b"Hello from Bits identity system!";
    let signature = identity_service.sign(message).await?;
    println!("\n📝 Message signing:");
    println!("  - Message: {}", String::from_utf8_lossy(message));
    println!("  - Signature: {}", hex::encode(&signature[..16]));
    
    // Verify the signature
    let is_valid = identity.keys.signing.public.verify(message, &signature);
    println!("  - Verification: {}", if is_valid { "✅ Valid" } else { "❌ Invalid" });
    
    // Export backup
    let password = "demo-password";
    let backup = identity_service.export_backup(password).await?;
    println!("\n💾 Backup:");
    println!("  - Size: {} bytes", backup.len());
    println!("  - Encrypted: ✅");
    
    // Show DID document
    println!("\n📄 DID Document:");
    println!("  - Context: {:?}", identity.document.context);
    println!("  - Verification Methods: {}", identity.document.verification_method.len());
    for vm in &identity.document.verification_method {
        println!("    • {} ({})", vm.id, vm.r#type);
    }
    
    // Demonstrate child key derivation
    let child_key = identity.keys.master.derive_child(b"demo-child");
    println!("\n👶 Derived child key:");
    println!("  - Public: {:?}", hex::encode(&child_key.public.as_bytes()[..8]));
    
    // Clean shutdown
    identity_service.stop().await?;
    println!("\n✅ Identity service stopped gracefully");
    
    Ok(())
}
