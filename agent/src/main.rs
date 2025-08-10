//! Binary entry point for the agent crate.
//! Runs the simple Rig demo defined in the agent library.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Call the demo function from the library. This expects XAI_API_KEY to be set.
    agent::basic().await
}
