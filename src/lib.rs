use zed_extension_api::{self as zed, serde_json, settings::LspSettings, Result};

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
            .ok_or_else(|| "Metals must be installed via coursier. Please install coursier (https://get-coursier.io/), and then run `cs install metals`.".to_string())?;

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
}

zed::register_extension!(ScalaExtension);
