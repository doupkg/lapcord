#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use anyhow::Result;
use lapce_plugin::{
    psp_types::{
        lsp_types::{request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, Url, MessageType},
        Request,
    },
    register_plugin, LapcePlugin, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

#[derive(Default)]
struct State {}

register_plugin!(State);

//When the user opens Lapce execute this
fn initialize(params: InitializeParams) -> Result<()> {
    let mut client = DiscordIpcClient::new("1000000000000000000")?;

    client.connect()?;
    client.set_activity(activity::Activity::new()
        .state("Testing")
        .details("RPC Rust")
    )?;
    client.close()?;

    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.window_show_message(MessageType::ERROR, format!("plugin returned with error: {e}"))
                }
            }
            _ => {}
        }
    }
}