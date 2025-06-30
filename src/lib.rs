use zed_extension_api::{
    self as zed,
    lsp::{Completion, CompletionKind, Symbol, SymbolKind},
    serde_json,
    settings::LspSettings,
    CodeLabel, CodeLabelSpan, Result,
};

struct ScalaExtension;

impl zed::Extension for ScalaExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = worktree
            .which("metals")
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

        Ok(zed::Command {
            command: path,
            args: arguments,
            env: worktree.shell_env(),
        })
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
}

use zed_extension_api::{DebugAdapterBinary, DebugTaskDefinition, StartDebuggingRequestArgumentsRequest, Worktree};

impl ScalaExtension {
    fn get_metals_dap_path(&self, worktree: &zed::Worktree) -> Result<String> {
        worktree
            .which("metals-dap")
            .ok_or_else(|| "Metals DAP server (metals-dap) must be installed manually. Please ensure it is in your PATH.".to_string())
    }
}

impl zed::Extension for ScalaExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = worktree
            .which("metals")
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

        Ok(zed::Command {
            command: path,
            args: arguments,
            env: worktree.shell_env(),
        })
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

    fn get_dap_binary(
        &mut self,
        adapter_name: zed::DapAdapterName,
        _config: DebugTaskDefinition,
        _user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary> {
        if adapter_name.as_ref() != "metals-dap" {
            return Err("Unsupported debug adapter".to_string());
        }

        let dap_path = self.get_metals_dap_path(worktree)?;

        // Most DAP servers communicate via stdio by default.
        // Arguments might be needed based on how Metals DAP server is implemented.
        // For now, we'll assume no special arguments are needed.
        Ok(DebugAdapterBinary {
            path: dap_path,
            arguments: Vec::new(),
            environment: worktree.shell_env(),
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: zed::DapAdapterName,
        _config: zed_extension_api::serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest> {
        let config_map = match _config {
            serde_json::Value::Object(map) => map,
            _ => return Ok(StartDebuggingRequestArgumentsRequest::Launch), // Default to launch if not an object
        };

        if config_map.contains_key("processId") {
            Ok(StartDebuggingRequestArgumentsRequest::Attach)
        } else {
            Ok(StartDebuggingRequestArgumentsRequest::Launch)
        }
    }

    fn dap_config_to_scenario(
        &mut self,
        _adapter_name: zed::DapAdapterName,
        _config: zed::DebugConfig,
    ) -> Result<zed::DebugScenario> {
        let mut dap_config = serde_json::Map::new();

        // Common DAP fields
        dap_config.insert("name".to_string(), serde_json::Value::String(_config.label.unwrap_or("Scala Debug".to_string())));
        dap_config.insert("type".to_string(), serde_json::Value::String("scala".to_string()));
        // `request` will be determined by `dap_request_kind` based on presence of `processId`
        // but we can set a default here if not already present from a more specific rule.
        // This might be refined later.
        let request_type = if _config.config.contains_key("processId") { "attach" } else { "launch" };
        dap_config.insert("request".to_string(), serde_json::Value::String(request_type.to_string()));


        // Metals specific fields - mapping from generic DebugConfig
        // Users will provide these in their .zed/debug.json
        if let Some(main_class) = _config.config.get("mainClass").and_then(|v| v.as_str()) {
            dap_config.insert("mainClass".to_string(), serde_json::Value::String(main_class.to_string()));
        }

        if let Some(test_class) = _config.config.get("testClass").and_then(|v| v.as_str()) {
            dap_config.insert("testClass".to_string(), serde_json::Value::String(test_class.to_string()));
        }

        if let Some(program) = _config.program {
            // If neither mainClass nor testClass is set, 'program' could potentially be used as mainClass.
            // This assumption needs validation against how Metals DAP behaves.
            if !dap_config.contains_key("mainClass") && !dap_config.contains_key("testClass") {
                 dap_config.insert("mainClass".to_string(), serde_json::Value::String(program.clone()));
            }
            // Or, if 'program' is meant to be the path to a JAR file for execution:
            // dap_config.insert("jar".to_string(), serde_json::Value::String(program));
        }

        if let Some(args) = _config.args {
            dap_config.insert("args".to_string(), serde_json::Value::Array(args.into_iter().map(serde_json::Value::String).collect()));
        }

        if let Some(cwd) = _config.cwd {
            dap_config.insert("cwd".to_string(), serde_json::Value::String(cwd));
        }

        if let Some(jvm_options) = _config.config.get("jvmOptions").and_then(|v| v.as_array()) {
            dap_config.insert("jvmOptions".to_string(), serde_json::Value::Array(jvm_options.clone()));
        } else if let Some(jvm_options_str) = _config.config.get("jvmOptions").and_then(|v| v.as_str()) {
            // Handle if jvmOptions is provided as a single string
             let options_array: Vec<serde_json::Value> = jvm_options_str.split_whitespace().map(|s| serde_json::Value::String(s.to_string())).collect();
            dap_config.insert("jvmOptions".to_string(), serde_json::Value::Array(options_array));
        }


        if let Some(build_target) = _config.config.get("buildTarget").and_then(|v| v.as_str()) {
            dap_config.insert("buildTarget".to_string(), serde_json::Value::String(build_target.to_string()));
        }

        // Handle processId for attach requests
        if let Some(process_id) = _config.config.get("processId") {
            dap_config.insert("processId".to_string(), process_id.clone());
        }

        // TODO: Handle environment variables (_config.env) if Metals DAP supports them.
        // let env_map: serde_json::Map<String, serde_json::Value> = _config.env
        //     .unwrap_or_default()
        //     .into_iter()
        //     .map(|(k, v)| (k, serde_json::Value::String(v)))
        //     .collect();
        // dap_config.insert("env".to_string(), serde_json::Value::Object(env_map));


        Ok(zed::DebugScenario {
            label: _config.label.unwrap_or_else(|| "Scala Debug Scenario".to_string()),
            config: serde_json::Value::Object(dap_config),
            build_task: _config.build, // Pass through any build task
        })
    }
}

zed::register_extension!(ScalaExtension);
