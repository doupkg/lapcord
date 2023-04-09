#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]

use anyhow::{ anyhow, Result };
use lapce_plugin::{
    psp_types::{
        lsp_types::{
            request::Initialize,
            DocumentFilter,
            DocumentSelector,
            InitializeParams,
            Url,
            MessageType,
        },
        Request,
    },
    register_plugin,
    LapcePlugin,
    VoltEnvironment,
    PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

macro_rules! ok {
    ($x:expr) => {
        match ($x) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!(e)),
        }
    };
}

macro_rules! string {
    ($x:expr) => {
      String::from($x)
    };
}

//When the user opens Lapce execute this
fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![DocumentFilter {
        // lsp language id Some(string!("typescript"))
        language: None,
        // glob pattern
        pattern: Some(string!("*")),
        // like file:
        scheme: None,
    }];

    let mut server_args = vec![string!("--stdio")];
    // Check for user specified LSP server path
    // ```
    // [lapce-plugin-name.lsp]
    // serverPath = "[path or filename]"
    // serverArgs = ["--arg1", "--arg2"]
    // ```
    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(lsp) = options.get("volt") {
            if let Some(args) = lsp.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    if !args.is_empty() {
                        server_args = vec![];
                    }
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }

            if let Some(server_path) = lsp.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    if !server_path.is_empty() {
                        let server_uri = Url::parse(&format!("urn:{}", server_path))?;
                        if
                            let Err(e) = PLUGIN_RPC.start_lsp(
                                server_uri,
                                server_args,
                                document_selector,
                                params.initialization_options
                            )
                        {
                            ok!(
                                PLUGIN_RPC.window_show_message(
                                    MessageType::ERROR,
                                    format!("plugin returned with error: {e}")
                                )
                            );
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    // Plugin working directory
    let server_uri = match VoltEnvironment::operating_system().as_deref() {
        Ok("windows") => ok!(Url::parse("urn:lapcord.cmd")),
        _ => ok!(Url::parse("urn:lapcord")),
    };

    // if you want to use server from PATH
    // let server_uri = Url::parse(&format!("urn:{filename}"))?;

    // Available language IDs
    // https://github.com/lapce/lapce/blob/HEAD/lapce-proxy/src/buffer.rs#L173
    if
        let Err(e) = PLUGIN_RPC.start_lsp(
            server_uri,
            server_args,
            document_selector,
            params.initialization_options
        )
    {
        ok!(
            PLUGIN_RPC.window_show_message(
                MessageType::ERROR,
                format!("plugin returned with error: {e}")
            )
        );
    }
    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                if let Err(e) = initialize(params) {
                    PLUGIN_RPC.window_show_message(
                        MessageType::ERROR,
                        format!("plugin returned with error: {e}")
                    ).expect("Error")
                }
            }
            _ => {}
        }
    }
}
