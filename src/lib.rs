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

        Ok(zed::Command {
            command: path,
            args: vec![],
            env: worktree.shell_env(),
        })
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
