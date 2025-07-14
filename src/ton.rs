use std::{env, fs};
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

struct TonExtension {}

impl TonExtension {
    fn find_ton_lsp(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "ton-blockchain/ton-language-server",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = format!("ton-language-server-{}.zip", release.version);
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| "no asset found")?;
        let version_dir = format!("ton-lsp-{}", release.version);

        fs::create_dir_all(&version_dir).map_err(|e| format!("failed to create directory: {e}"))?;
        let binary_path = format!("{version_dir}/ton-language-server/server.js");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file error: {e}"))?;
        }

        return Ok(binary_path);
    }

    fn find_tact_lsp(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "tact-lang/tact-language-server",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        // TODO(bionic2113): change to "tact-language-server-{}.zip"
        let asset_name = format!(
            "vscode-tact-{}.vsix",
            release.version.trim_start_matches('v')
        );
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found {}", release.version.trim_start_matches('v')))?;
        let version_dir = format!("tact-lsp-{}", release.version);

        fs::create_dir_all(&version_dir).map_err(|e| format!("failed to create directory: {e}"))?;
        let binary_path = format!("{version_dir}/extension/dist/server.js");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file error: {e}"))?;
        }

        return Ok(binary_path);
    }
}

impl zed::Extension for TonExtension {
    fn new() -> Self {
        Self {}
    }
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path = match language_server_id.as_ref() {
            "func" => self.find_ton_lsp(language_server_id),
            "tolk" => self.find_ton_lsp(language_server_id),
            "fift" => self.find_ton_lsp(language_server_id),
            "tlb" => self.find_ton_lsp(language_server_id),
            "tact" => self.find_tact_lsp(language_server_id),
            _ => {
                return Err(format!(
                    "Unrecognized language server for {}",
                    language_server_id
                ))
            }
        }?;

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_string(),
            ],
            env: Default::default(),
        })
    }
}

zed::register_extension!(TonExtension);
