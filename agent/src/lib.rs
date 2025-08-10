//! Agent crate: provides basic rig agent helpers and demo function.

use std::env;
use rig::agent::AgentBuilder;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers;

pub fn client() -> providers::xai::Client {
    providers::xai::Client::new(&env::var("XAI_API_KEY").expect("XAI_API_KEY not set"))
}

pub fn partial_agent() -> AgentBuilder<providers::xai::completion::CompletionModel> {
    let client = client();
    client.agent(providers::xai::GROK_3_MINI)
}

pub async fn basic() -> Result<(), anyhow::Error> {
    let comedian_agent = partial_agent()
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .build();
    let response = comedian_agent.prompt("Entertain me!").await?;
    println!("{response}");
    Ok(())
}
