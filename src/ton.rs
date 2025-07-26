use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::{env, fs};
use std::{net::Ipv4Addr, time::Duration};
use zed::LanguageServerId;
use zed_extension_api::{
    self as zed, resolve_tcp_template, DebugAdapterBinary, DebugConfig, DebugRequest,
    DebugScenario, DebugTaskDefinition, Result, StartDebuggingRequestArguments,
    StartDebuggingRequestArgumentsRequest, TcpArgumentsTemplate, Worktree,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default, Serialize, Deserialize)]
struct TonDebugConfig {
    request: String,
    #[serde(default)]
    host: String,
    command: Option<String>,
    cwd: Option<String>,
    args: Vec<String>,
    env: HashMap<String, String>,
    #[serde(rename = "stopOnEntry", default)]
    stop_on_entry: bool,
    #[serde(rename = "stopOnBreakpoint", default)]
    stop_on_breakpoint: bool,
    #[serde(rename = "stopOnStep", default)]
    stop_on_step: Option<bool>,
    #[serde(default)]
    port: Option<u16>,
}

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

    fn dap_request_kind(
        &mut self,
        _: String,
        config: serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        config
            .get("request")
            .and_then(|v| {
                v.as_str().and_then(|s| {
                    s.eq("launch")
                        .then(|| StartDebuggingRequestArgumentsRequest::Launch)
                })
            })
            .ok_or_else(|| "Invalid config".into())
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        _: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        let configuration: serde_json::Value = serde_json::from_str(&config.config)
            .map_err(|e| format!("`config` is not a valid JSON: {e}"))?;
        let ton_config: TonDebugConfig = serde_json::from_value(configuration.clone())
            .map_err(|e| format!("`config` is not a valid ton config: {e}"))?;

        let tcp_connection = config.tcp_connection.unwrap_or(TcpArgumentsTemplate {
            port: Some(ton_config.port.unwrap_or(42069)),
            host: Some(
                Ipv4Addr::from_str(ton_config.host.as_str())
                    .unwrap_or(Ipv4Addr::LOCALHOST)
                    .to_bits(),
            ),
            timeout: Some(DEFAULT_TIMEOUT.as_millis() as u64),
        });

        let connection = resolve_tcp_template(tcp_connection)?;

        let request_type = self.dap_request_kind(adapter_name.clone(), configuration.clone())?;

        let arguments = vec![];
        Ok(DebugAdapterBinary {
            command: None,
            arguments,
            connection: Some(connection),
            cwd: None,
            envs: worktree.shell_env(),
            request_args: StartDebuggingRequestArguments {
                configuration: configuration.to_string(),
                request: request_type,
            },
        })
    }
    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        let obj = match &config.request {
            DebugRequest::Attach(_) => {
                return Err("Ton adapter doesn't support attaching".into());
            }
            DebugRequest::Launch(launch_config) => json!({
                "type": "tvm",
                "request": "launch",
                "program": launch_config.program,
                "cwd": launch_config.cwd,
                "args": launch_config.args,
                "env": serde_json::Value::Object(
                    launch_config.envs
                        .iter()
                        .map(|(k, v)| (k.clone(), v.to_owned().into()))
                        .collect::<serde_json::Map<String, serde_json::Value>>(),
                ),
                "stopOnEntry": config.stop_on_entry.unwrap_or_default(),
            }),
        };

        Ok(DebugScenario {
            adapter: config.adapter,
            label: config.label,
            build: None,
            config: obj.to_string(),
            tcp_connection: None,
        })
    }
}

zed::register_extension!(TonExtension);
