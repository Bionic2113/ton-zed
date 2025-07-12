use std::{env, fs};
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

pub const LANGUAGE_SERVER_ID: &'static str = "kotlin-language-server";
const SERVER_PATH: &str = "node_modules/func-extracted-ls/bin/bundle.js";
const PACKAGE_NAME: &str = "func-extracted-ls";

struct TonExtension {
    did_find_server: bool,
}

impl TonExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).map_or(false, |stat| stat.is_file())
    }

    fn server_script_path(&mut self, config: &LanguageServerId) -> Result<String> {
        let server_exists = self.server_exists();
        if self.did_find_server && server_exists {
            return Ok(SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            config,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !server_exists
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                config,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }

        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }

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
        // TODO(bionic2113): add "/dist"
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
        Self {
            did_find_server: false,
        }
    }
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path = match language_server_id.as_ref() {
            "func" => self.server_script_path(language_server_id),
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
