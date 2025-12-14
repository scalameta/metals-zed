use std::{
    collections::HashSet,
    env,
    str::FromStr,
    sync::{Arc, RwLock},
};

use zed_extension_api::{
    self as zed, CodeLabel, CodeLabelSpan, DebugAdapterBinary, DebugConfig, DebugScenario,
    DebugTaskDefinition, Extension, Result, StartDebuggingRequestArguments,
    StartDebuggingRequestArgumentsRequest, Worktree,
    lsp::{Completion, CompletionKind, Symbol, SymbolKind},
    serde_json::{self, Value},
    settings::LspSettings,
};

use crate::dap::{Debugger, ScalaDebugTaskDefinition};

mod dap;

const LSP_DAP_NAME: &str = "Metals";
// Proxy is required to send request to LSP and to be able to start the DAP server
// Zed doesn't support sesnding requests to LSP from extensions
const PROXY_CODE: &str = include_str!("proxy.mjs");
const USE_PROXY: bool = true;

struct ScalaExtension {
    dap: Debugger,                           // DAP specific methods
    wrks_lock: Arc<RwLock<HashSet<String>>>, // List of initialized workspaces - set by LSP, checked by DAP
}

impl Extension for ScalaExtension {
    fn new() -> Self {
        Self {
            dap: Debugger::new(),
            wrks_lock: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    // This method is called by Zed to start LSP
    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let metals_path = worktree
            .which(LSP_DAP_NAME)
            .ok_or_else(|| "Metals must be installed manually. Recommended way is to install coursier (https://get-coursier.io/), and then run `cs install metals`.".to_string())?;

        let arguments = LspSettings::for_worktree("metals", worktree)
            .map(|lsp_settings| {
                lsp_settings
                    .binary
                    .and_then(|binary| binary.arguments)
                    // If no arguments are provided, default to enabling the HTTP server.
                    .unwrap_or(vec!["-Dmetals.http=on".to_string()])
            })
            .unwrap_or_default();

        if USE_PROXY {
            // Get extension directory to store the proxy port number in dedicated file there
            let extension_dir = env::current_dir()
                .map_err(|err| format!("Could not get current dir: {err}"))
                .and_then(|p| {
                    p.to_str()
                        .map(|s| s.to_string())
                        .ok_or("Could not convert path to string".to_string())
                })?;

            // Provide arguments to Node to start the proxy and Metals through it
            let mut args = vec![
                "--input-type=module".to_string(),
                "-e".to_string(),
                PROXY_CODE.to_string(),
                extension_dir,
                metals_path,
            ];
            // Add arguments for Metals to pass them through
            args.extend(arguments.to_owned());

            // Add current workspace to the set of initialized workspaces (for DAP)
            // Unfortunately, LSP isn't initialized yet, and this doesn't prevent calling the DAP too early,
            // but there is no handler to implement it in a proper way
            let workspace = worktree.root_path();
            let mut workspaces = self
                .wrks_lock
                .write()
                .map_err(|e| format!("Could not mark current workspace as initialized: {e}"))?;
            workspaces.insert(workspace);

            Ok(zed::Command {
                command: zed::node_binary_path()?, // Node is used to start the proxy
                args,
                env: worktree.shell_env(),
            })
        } else {
            Ok(zed::Command {
                command: metals_path,
                args: arguments,
                env: worktree.shell_env(),
            })
        }
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let initialization_options = LspSettings::for_worktree("metals", worktree)
            .map(|lsp_settings| lsp_settings.initialization_options.clone());

        initialization_options
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree("metals", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();

        Ok(Some(serde_json::json!({
            "metals": settings
        })))
    }

    fn label_for_completion(
        &self,
        _language_server_id: &zed_extension_api::LanguageServerId,
        completion: Completion,
    ) -> Option<zed_extension_api::CodeLabel> {
        let prefix = match completion.kind? {
            CompletionKind::Method | CompletionKind::Function => "def ",
            CompletionKind::Constructor
            | CompletionKind::Class
            | CompletionKind::Interface
            | CompletionKind::Module => "class ",
            CompletionKind::Variable => "var ",
            CompletionKind::Field
            | CompletionKind::Constant
            | CompletionKind::Value
            | CompletionKind::Property => "val ",
            CompletionKind::Enum => "enum ",
            CompletionKind::Keyword => "",
            _ => return None,
        };
        let name = completion.label.replace("  ", " ").replace("\n", "");
        let code = format!("{prefix}{name}");
        let code_len = code.len();
        Some(CodeLabel {
            code,
            spans: vec![CodeLabelSpan::code_range(prefix.len()..code_len)],
            filter_range: (0..name.len()).into(),
        })
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &zed_extension_api::LanguageServerId,
        symbol: Symbol,
    ) -> Option<CodeLabel> {
        let prefix = match symbol.kind {
            SymbolKind::Module
            | SymbolKind::Class
            | SymbolKind::Interface
            | SymbolKind::Constructor => "class ",
            SymbolKind::Method | SymbolKind::Function => "def ",
            SymbolKind::Variable => "var ",
            SymbolKind::Property | SymbolKind::Field | SymbolKind::Constant => "val ",
            _ => "",
        };
        let name = symbol.name;
        let code = format!("{prefix}{name}");
        let code_len = code.len();
        Some(CodeLabel {
            code,
            spans: vec![CodeLabelSpan::code_range(prefix.len()..code_len)],
            filter_range: (0..name.len()).into(),
        })
    }

    // This method is called by Zed to start debugger
    // In case of Metals, this requires sending "debug-adapter-start" request to LSP
    // See: https://www.chris-kipp.io/blog/the-debug-adapter-protocol-and-scala
    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        _user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        if adapter_name != LSP_DAP_NAME {
            return Err(format!("Cannot get binary for adapter \"{adapter_name}\""));
        }

        let workspace = worktree.root_path();
        if USE_PROXY {
            // Check if LSP has been initialized for the current workspace and thus is able to start DAP.
            let workspaces = self
                .wrks_lock
                .read()
                .map_err(|e| format!("Could not check current workspace: {e}"))?;
            if !workspaces.contains(&workspace) {
                return Err(format!(
                    "The Metals LSP server hasn't been started yet for the current workspace {workspace}. Trigger LSP initialization before debugging."
                ));
            }
        } else {
            return Err(format!("DAP is not supported by Scala extension"));
        }

        // Parse the user-provided debug configuration
        // Please note, that "label" and "adapter", required by Zed, are stripped before passing to extension
        let conf = Value::from_str(config.config.as_str())
            .map_err(|e| format!("Invalid JSON configuration: {e}"))?;
        let scala_conf: ScalaDebugTaskDefinition = serde_json::from_value(conf.clone())
            .map_err(|e| format!("Cannot parse debug taks definition: {e}"))?;

        // Determine debug mode (lauch or attach)
        let request_kind = self.dap_request_kind(adapter_name, conf)?;
        // Check and enrich debug configuration with default values
        let arguments = self.dap.enrich_config(&workspace, scala_conf)?;

        // Return debug configuration back to Zed
        let arguments_json = serde_json::to_string(&arguments)
            .map_err(|e| format!("Cannot create debug taks definition: {e}"))?;
        let request_args = StartDebuggingRequestArguments {
            request: request_kind,
            configuration: arguments_json,
        };

        // Start the debugger with provided arguments for current workspace
        let connection = Some(zed::resolve_tcp_template(
            self.dap.start(&workspace, &arguments)?,
        )?);

        // Return connection to already started debugger
        Ok(DebugAdapterBinary {
            command: None,
            arguments: vec![],
            cwd: Some(workspace),
            envs: vec![],
            request_args,
            connection,
        })
    }

    // This method returns debug mode (launch or attach) based on configuration provided by user
    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        if adapter_name != LSP_DAP_NAME {
            return Err(format!("Cannot get binary for adapter \"{adapter_name}\""));
        }

        match config.get("request") {
            Some(req) if req == "launch" => Ok(StartDebuggingRequestArgumentsRequest::Launch),
            Some(req) if req == "attach" => Ok(StartDebuggingRequestArgumentsRequest::Attach),
            Some(req) => Err(format!(
                "Unexpected value for `request` key in Metals debug configuration: {req:?}"
            )),
            None => Err("Missing required `request` field in Metals debug configuration".into()),
        }
    }

    // Method to convert a standard debug configuration, with basic user input user,
    // into a configuration required by Metals.
    fn dap_config_to_scenario(
        &mut self,
        generic_config: DebugConfig,
    ) -> Result<DebugScenario, String> {
        if generic_config.adapter != LSP_DAP_NAME {
            return Err(format!(
                "Cannot create configuration for adapter \"{}\"",
                generic_config.adapter
            ));
        }
        // Create Scala specific debug task definition based on generic one
        let scala_config = self.dap.convert_generic_config(generic_config.clone());

        // Return debug configuration back to Zed
        let arguments_json = serde_json::to_string(&scala_config).map_err(|e| {
            format!("Cannot create debug taks definition based on generic one: {e}")
        })?;

        Ok(DebugScenario {
            label: generic_config.label,
            adapter: generic_config.adapter,
            build: None,
            config: arguments_json,
            tcp_connection: None,
        })
    }
}

zed::register_extension!(ScalaExtension);
