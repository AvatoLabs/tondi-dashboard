use crate::app::{GIT_DESCRIBE, VERSION};
use crate::imports::*;
use crate::settings::NodeMemoryScale;
use crate::utils::Arglist;
use tondi_core::tondid_env;
#[cfg(not(target_arch = "wasm32"))]
pub use tondid_lib::args::Args;

fn user_agent_comment() -> String {
    format!("tondi-ng:{}-{}", VERSION, GIT_DESCRIBE)
}

#[allow(dead_code)]
fn user_agent() -> String {
    format!(
        "/{}:{}/tondi-ng:{}-{}/",
        tondid_env::name(),
        tondid_env::version(),
        VERSION,
        GIT_DESCRIBE
    )
}

#[derive(Debug, Clone)]
pub struct Config {
    network: Network,
    enable_upnp: bool,
    enable_wrpc_borsh: bool,
    #[allow(dead_code)]
    enable_wrpc_json: bool,
    enable_grpc: bool,
    grpc_network_interface: NetworkInterfaceConfig,
    tondid_daemon_args_enable: bool,
    tondid_daemon_args: String,
    tondid_daemon_storage_folder_enable: bool,
    tondid_daemon_storage_folder: String,
    memory_scale: NodeMemoryScale,
    devnet_custom_url: Option<String>,
}

impl From<NodeSettings> for Config {
    fn from(node_settings: NodeSettings) -> Self {
        Self {
            network: node_settings.network,
            enable_upnp: node_settings.enable_upnp,
            enable_wrpc_borsh: node_settings.enable_wrpc_borsh,
            enable_wrpc_json: node_settings.enable_wrpc_json,
            enable_grpc: node_settings.enable_grpc,
            grpc_network_interface: node_settings.grpc_network_interface,
            tondid_daemon_args_enable: node_settings.tondid_daemon_args_enable,
            tondid_daemon_args: node_settings.tondid_daemon_args,
            tondid_daemon_storage_folder_enable: node_settings.tondid_daemon_storage_folder_enable,
            tondid_daemon_storage_folder: node_settings.tondid_daemon_storage_folder,
            memory_scale: node_settings.memory_scale,
            devnet_custom_url: node_settings.devnet_custom_url,
        }
    }
}

cfg_if! {

    if #[cfg(not(target_arch = "wasm32"))] {
        impl TryFrom<Config> for Args {
            type Error = Error;
            fn try_from(config: Config) -> Result<Self> {
                let mut args = Args::default();
                match config.network {
                    Network::Mainnet => {}
                    Network::Testnet => {
                        args.testnet = true;
                        args.testnet_suffix = 10;
                    }
                    Network::Devnet => {
                        args.testnet = true;
                        args.testnet_suffix = 11;
                    }
                }

                args.perf_metrics = true;
                args.perf_metrics_interval_sec = 1;
                args.yes = true;
                args.utxoindex = true;
                args.disable_upnp = !config.enable_upnp;

                if config.enable_grpc {
                    args.rpclisten = Some(config.grpc_network_interface.into());
                }

                args.user_agent_comments = vec![user_agent_comment()];

                // Add custom devnet URL if specified
                if let Some(custom_url) = &config.devnet_custom_url {
                    if !custom_url.is_empty() {
                        // Add the custom URL as a command line argument
                        // This will be parsed by tondid to connect to the custom devnet node
                        args.user_agent_comments.push(format!("devnet-url:{}", custom_url));
                    }
                }

                // TODO - parse custom args and overlap on top of the defaults

                Ok(args)
            }
        }

        impl From<Config> for Vec<String> {
            fn from(config: Config) -> Self {
                let mut args = Arglist::default();

                match config.network {
                    Network::Mainnet => {}
                    Network::Testnet => {
                        args.push("--testnet");
                        args.push("--netsuffix=10");
                    }
                    Network::Devnet => {
                        // Devnet is a separate network type, not testnet with suffix
                        // No additional arguments needed
                    }
                }

                args.push("--perf-metrics");
                args.push("--perf-metrics-interval-sec=1");
                args.push("--yes");
                args.push("--utxoindex");

                match config.memory_scale {
                    NodeMemoryScale::Default => {},
                    _ => {
                        args.push(format!("--ram-scale={:1.2}", config.memory_scale.get()));
                    }
                }

                if !config.enable_upnp {
                    args.push("--disable-upnp");
                }

                if config.enable_grpc {
                    args.push(format!("--rpclisten={}", config.grpc_network_interface));
                } else {
                    args.push("--nogrpc");
                }

                if config.enable_wrpc_borsh {
                    args.push("--rpclisten-borsh=0.0.0.0");
                } else {
                    args.push("--rpclisten-borsh=127.0.0.1");
                }

                args.push(format!("--uacomment={}", user_agent_comment()));

                // Add custom devnet URL if specified
                if let Some(custom_url) = &config.devnet_custom_url {
                    if !custom_url.is_empty() {
                        // Add the custom URL as a command line argument
                        args.push(format!("--uacomment=devnet-url:{}", custom_url));
                    }
                }

                if config.tondid_daemon_storage_folder_enable && !config.tondid_daemon_storage_folder.is_empty() && !(config.tondid_daemon_args_enable && config.tondid_daemon_args.contains("--appdir")) {
                    args.push(format!("--appdir={}", config.tondid_daemon_storage_folder));
                }

                if config.tondid_daemon_args_enable {
                    config.tondid_daemon_args.trim().split(' ').filter(|arg|!arg.trim().is_empty()).for_each(|arg| {
                        args.push(arg);
                    });
                }

                args.into()
            }
        }

        impl IntoIterator for Config {
            type Item = String;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                let args: Vec<String> = self.into();
                args.into_iter()
            }
        }
    }
}
