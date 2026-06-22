fn main() {
    // Embed DISCORD_CLIENT_ID from .env (or the environment) at build time.
    dotenvy::dotenv().ok();
    let client_id = std::env::var("DISCORD_CLIENT_ID").unwrap_or_default();
    println!("cargo:rustc-env=DISCORD_CLIENT_ID={client_id}");
    println!("cargo:rerun-if-changed=.env");

    tauri_build::build()
}
