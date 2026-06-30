//! `fpa` — operations & developer CLI for the Flint Platform Agent.
//!
//! A Supabase-style admin/dev entry point (run the agent, inspect fabric,
//! manage tasks). Subcommands are added as the corresponding use-cases land.
//! `anyhow` is permitted here (binary entry point).

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    println!("fpa — Flint Platform Agent CLI (scaffold)");
    Ok(())
}
