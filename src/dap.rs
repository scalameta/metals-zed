// This file includes code originally from:
//   Project: zed-extensions / java
//   Source: https://github.com/zed-extensions/java/
// Original code is licensed under the Apache License, Version 2.0.
// Modifications copyright (c) 2025 Scalameta Maintainers.
// Licensed under the Apache License, Version 2.0 (the "License").

// DAP (Debug Adapter Protocol) specific implementation

use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use zed_extension_api::{
    self as zed, DebugConfig, DebugRequest, TcpArgumentsTemplate,
    http_client::{self as http, HttpMethod, HttpRequest},
    serde_json::{self, Map, Value, json},
};

const LSP_REQUEST: &str = "workspace/executeCommand"; // LSP request to send a command
const DAP_START_COMMAND: &str = "debug-adapter-start"; // The command send to LSP to initialize debugger
const PROXY_FOLDER: &str = "proxy"; // The folder (inside Zed's `extentions/work/scala` folder) to put port info to
const DEFAULT_LAUNCH_RUN_TYPE: &str = "run"; // Default runType for autodiscovery debugee launch mode
const DEFAULT_ATTACH_HOST_NAME: &str = "localhost"; // Default hostName for debugee attach mode
const DEFAULT_ATTACH_PORT: u16 = 5005; // Default port number for debugee attach mode

// Struct representing debugging configuration as required by Metals' "debug-adapter-start" command
// See https://scalameta.org/metals/docs/integrations/debug-adapter-protocol/
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScalaDebugTaskDefinition {
    Launch(ScalaDebugLauchDefinition),
    Attach(ScalaDebugAttachDefinition),
}

// Debugging configuration for launch mode
// For launching vs attaching see https://zed.dev/docs/debugger#launching--attaching
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalaDebugLauchDefinition {
    request: String,
    #[serde(flatten)]
    entry: EntryPoint,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "buildTaget")]
    build_taget: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "jvmOptions")]
    jvm_options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "envFile")]
    env_file: Option<String>,
}

// Debugging configuration for attach mode
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScalaDebugAttachDefinition {
    request: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "buildTaget")]
    build_taget: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hostName")]
    host_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
}

// Debugger needs an entry point to launch a program or test.
// There are 3 ways to provide it:
// - Auto: Automatically detect the entry point based on the provided file (path)
// - Main: Specify the main class to run
// - Test: Specify the test class to run
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum EntryPoint {
    Auto {
        path: String,
        #[serde(rename = "runType")]
        run_type: Option<String>,
    },
    Main {
        #[serde(rename = "mainClass")]
        main_class: String,
    },
    Test {
        #[serde(rename = "testClass")]
        test_class: String,
    },
}

// Struct representing response from LSP server (Metals)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LspResponse<T> {
    Success { result: T },
    Error { error: LspError },
}

#[derive(Debug, Deserialize)]
struct LspError {
    code: i64,
    message: String,
    data: Option<Value>,
}

// Struct representing response to "debug-adapter-start" command
#[derive(Debug, Deserialize)]
#[serde(rename = "result")]
struct DapStartResult {
    #[serde(rename = "name")]
    _name: String,
    uri: String,
}

// Struct containing methods responsible for debugger initialization
pub struct Debugger;

impl Debugger {
    pub fn new() -> Self {
        Self
    }

    // Starts the debugger by sending "debug-adapter-start" request to Metals
    pub fn start(
        &self,
        workspace: &str,
        arguments: &ScalaDebugTaskDefinition,
    ) -> zed::Result<TcpArgumentsTemplate> {
        // Send the "debug-adapter-start" request to LSP
        let response = self.lsp_request::<DapStartResult>(
            workspace,
            LSP_REQUEST,
            json!({
              "command": DAP_START_COMMAND,
              "arguments": [ arguments ]
            }),
        )?;

        // Get debugger port from request
        let port = get_port_from_uri(response.uri.as_str())?;

        // Return TCP connection data
        Ok(TcpArgumentsTemplate {
            host: None,
            port: Some(port),
            timeout: None,
        })
    }

    // Get the arguments for "debug-adapter-start" request.
    // Although they are mainly provided by user through debugger configuration in `debug_task_def`
    // (see: https://zed.dev/docs/debugger#configuration), this method verifies key ones
    // and provides default values where possible.
    pub fn enrich_config(
        &self,
        workspace: &str,
        debug_task_def: ScalaDebugTaskDefinition,
    ) -> zed::Result<ScalaDebugTaskDefinition> {
        match debug_task_def.clone() {
            // Launch mode
            ScalaDebugTaskDefinition::Launch(config) if config.request == "launch" => {
                match config.entry {
                    // For autodiscovery, prefix path with file scheme and provide default runType if missing
                    // Please note, that we cannot provide "$ZED_FILE" as default for path,
                    // because it's evaluated only before calling extension.
                    EntryPoint::Auto { path, run_type } => {
                        let path = if path.starts_with("file://") {
                            path
                        } else {
                            format!("file://{}", full_path(&path, workspace))
                        };
                        let run_type = run_type.or(Some(DEFAULT_LAUNCH_RUN_TYPE.to_string()));
                        let config = ScalaDebugLauchDefinition {
                            entry: EntryPoint::Auto { path, run_type },
                            ..config
                        };
                        Ok(ScalaDebugTaskDefinition::Launch(config))
                    }
                    // No defaults for mainClass
                    EntryPoint::Main { main_class: _ } => Ok(debug_task_def),
                    // No defaults for testClass
                    EntryPoint::Test { test_class: _ } => Ok(debug_task_def),
                }
            }
            // Attach mode - provide default host and port if missing
            ScalaDebugTaskDefinition::Attach(config) if config.request == "attach" => {
                let host_name = config
                    .host_name
                    .or(Some(DEFAULT_ATTACH_HOST_NAME.to_string()));
                let port = config.port.or(Some(DEFAULT_ATTACH_PORT));
                let config = ScalaDebugAttachDefinition {
                    host_name,
                    port,
                    ..config
                };
                Ok(ScalaDebugTaskDefinition::Attach(config))
            }
            _ => Err(format!("Incorrect format of debug task definition")),
        }
    }

    // Create basic Metals' specific debug task definition based on general Zed's debug task.
    // Leave optional arguments empty to be enriched with default values.
    pub fn convert_generic_config(&self, generic_config: DebugConfig) -> ScalaDebugTaskDefinition {
        match generic_config.request {
            // For lauch request start DAP in autodiscover mode
            DebugRequest::Launch(launch_request) => {
                let entry = EntryPoint::Auto {
                    path: launch_request
                        .cwd
                        .map(|cwd| full_path(&launch_request.program, &cwd))
                        .unwrap_or(launch_request.program),
                    run_type: None,
                };
                let config = ScalaDebugLauchDefinition {
                    request: "launch".to_string(),
                    entry,
                    build_taget: None,
                    args: if launch_request.args.is_empty() {
                        None
                    } else {
                        Some(launch_request.args)
                    },
                    jvm_options: None,
                    env: if launch_request.envs.is_empty() {
                        None
                    } else {
                        Some(launch_request.envs.into_iter().collect())
                    },
                    env_file: None,
                };
                ScalaDebugTaskDefinition::Launch(config)
            }
            // Metals don't support attaching to a process by ID,
            // so we cannot use the provided process identifier and must fall back to the defaults.
            DebugRequest::Attach(_attach_request) => {
                let config = ScalaDebugAttachDefinition {
                    request: "attach".to_string(),
                    build_taget: None,
                    host_name: None,
                    port: None,
                };
                ScalaDebugTaskDefinition::Attach(config)
            }
        }
    }

    // Send request to Metals through the proxy-exposed HTTP port
    fn lsp_request<T>(&self, workspace: &str, method: &str, params: Value) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        // Get the port number saved by proxy
        // We cannot cache it because the user may restart the LSP
        let port = {
            let filename = string_to_hex(workspace);
            let port_path = Path::new(PROXY_FOLDER).join(filename);
            if !fs::metadata(&port_path).is_ok_and(|file| file.is_file()) {
                return Err("Failed to find LSP port file".to_string());
            }

            fs::read_to_string(port_path)
                .map_err(|e| format!("Failed to read a LSP proxy port from file: {e}"))?
                .parse::<u16>()
                .map_err(|e| format!("Failed to read a LSP proxy port, file corrupted: {e}"))?
        };

        let mut body = Map::new();
        body.insert("method".to_string(), Value::String(method.to_string()));
        body.insert("params".to_string(), params);
        let request = &HttpRequest::builder()
            .method(HttpMethod::Post)
            .url(format!("http://localhost:{port}"))
            .body(Value::Object(body).to_string())
            .build()?;

        let res =
            http::fetch(request).map_err(|e| format!("Failed to send request to LSP proxy {e}"))?;
        let data: LspResponse<T> = serde_json::from_slice(&res.body)
            .map_err(|e| format!("Failed to parse response from LSP proxy {e}"))?;
        match data {
            LspResponse::Success { result } => Ok(result),
            LspResponse::Error { error } => Err(format!(
                "{} {} {}",
                error.code,
                error.message,
                error.data.map(|v| v.to_string()).unwrap_or(String::new())
            )),
        }
    }
}

// Retrieve port number from URI
fn get_port_from_uri(s: &str) -> Result<u16, String> {
    s.rsplit_once(':') // split the string at the last colon
        .ok_or("Cannot find port part in the URI".to_string())
        .and_then(|(_, port_str)| {
            port_str // if a colon is found, take the part after it
                .trim()
                .parse::<u16>() // try to parse as u16
                .map_err(|e| format!("Cannot parse port number from URI: {e}"))
        })
}

// Encode string as hexadecimal to use it as file name
fn string_to_hex(s: &str) -> String {
    let mut hex_string = String::new();
    for byte in s.as_bytes() {
        hex_string.push_str(&format!("{:02x}", byte));
    }
    hex_string
}

// Check if path is full (absolute) and if not prefix it with base
fn full_path(path: &str, base: &str) -> String {
    let p_path = Path::new(path);
    if p_path.is_absolute() {
        path.to_string()
    } else {
        let p_base = Path::new(base);
        p_base.join(p_path).to_string_lossy().to_string()
    }
}
