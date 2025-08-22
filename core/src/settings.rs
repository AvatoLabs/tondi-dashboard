use crate::imports::*;
use tondi_metrics_core::Metric;
use tondi_utils::networking::ContextualNetAddress;
use tondi_wallet_core::storage::local::storage::Storage;
use tondi_wrpc_client::WrpcEncoding;
use workflow_core::{runtime, task::spawn};

const SETTINGS_REVISION: &str = "0.0.0";

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum TondidNodeKind {
            Disable,
            Remote,
            IntegratedInProc,
            #[default]
            IntegratedAsDaemon,
            IntegratedAsPassiveSync,
            ExternalAsDaemon,
        }

        const TONDID_NODE_KINDS: [TondidNodeKind; 6] = [
            TondidNodeKind::Disable,
            TondidNodeKind::Remote,
            TondidNodeKind::IntegratedInProc,
            TondidNodeKind::IntegratedAsDaemon,
            TondidNodeKind::IntegratedAsPassiveSync,
            TondidNodeKind::ExternalAsDaemon,
        ];

        impl std::fmt::Display for TondidNodeKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    TondidNodeKind::Disable => write!(f, "{}", i18n("Disabled")),
                    TondidNodeKind::Remote => write!(f, "{}", i18n("Remote")),
                    TondidNodeKind::IntegratedInProc => write!(f, "{}", i18n("Integrated Node")),
                    TondidNodeKind::IntegratedAsDaemon => write!(f, "{}", i18n("Integrated Daemon")),
                    TondidNodeKind::IntegratedAsPassiveSync => write!(f, "{}", i18n("Passive Sync")),
                    TondidNodeKind::ExternalAsDaemon => write!(f, "{}", i18n("External Daemon")),
                }
            }
        }

    } else {
        #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
        #[serde(rename_all = "kebab-case")]
        pub enum TondidNodeKind {
            #[default]
            Disable,
            Remote,
        }

        const TONDID_NODE_KINDS: [TondidNodeKind; 1] = [
            TondidNodeKind::Remote,
        ];

        impl std::fmt::Display for TondidNodeKind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    TondidNodeKind::Disable => write!(f, "Disable"),
                    TondidNodeKind::Remote => write!(f, "Remote"),
                }
            }
        }
    }
}

impl TondidNodeKind {
    pub fn iter() -> impl Iterator<Item = &'static TondidNodeKind> {
        TONDID_NODE_KINDS.iter()
    }

    pub fn describe(&self) -> &str {
        match self {
            TondidNodeKind::Disable => i18n("Disables node connectivity (Offline Mode)."),
            TondidNodeKind::Remote => i18n("Connects to a Remote Tondi Client Node via wRPC."),
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedInProc => i18n("The node runs as a part of the Tondi Dashboard application process. This reduces communication overhead (experimental)."),
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedAsDaemon => i18n("The node is spawned as a child daemon process (recommended)."),
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedAsPassiveSync => i18n("The node synchronizes in the background while Tondi Dashboard is connected to a public node. Once the node is synchronized, you can switch to the 'Integrated Daemon' mode."),
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::ExternalAsDaemon => i18n("A binary at another location is spawned a child process (experimental, for development purposes only)."),
        }
    }

    pub fn is_config_capable(&self) -> bool {
        match self {
            TondidNodeKind::Disable => false,
            TondidNodeKind::Remote => false,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedInProc => true,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedAsDaemon => true,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedAsPassiveSync => true,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::ExternalAsDaemon => true,
        }
    }

    pub fn is_local(&self) -> bool {
        match self {
            TondidNodeKind::Disable => false,
            TondidNodeKind::Remote => false,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedInProc => true,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedAsDaemon => true,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::IntegratedAsPassiveSync => true,
            #[cfg(not(target_arch = "wasm32"))]
            TondidNodeKind::ExternalAsDaemon => true,
        }
    }
}

#[derive(Default)]
pub struct RpcOptions {
    pub blacklist_servers: Vec<String>,
}

impl RpcOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn blacklist(mut self, server: String) -> Self {
        self.blacklist_servers.push(server);
        self
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RpcKind {
    #[default]
    Wrpc,
    Grpc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcConfig {
    Wrpc {
        url: Option<String>,
        encoding: WrpcEncoding,
        resolver_urls: Option<Vec<Arc<String>>>,
    },
    Grpc {
        url: Option<NetworkInterfaceConfig>,
    },
}

impl Default for RpcConfig {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let url = "127.0.0.1";
            } else {
                use workflow_dom::utils::*;
                let url = window().location().hostname().expect("TondidNodeKind: Unable to get hostname");
            }
        }
        RpcConfig::Wrpc {
            url: Some(url.to_string()),
            encoding: WrpcEncoding::Borsh,
            resolver_urls: None,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkInterfaceKind {
    Local,
    Any,
    #[default]
    Custom,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkInterfaceConfig {
    #[serde(rename = "type")]
    pub kind: NetworkInterfaceKind,
    pub custom: ContextualNetAddress,
}

impl Default for NetworkInterfaceConfig {
    fn default() -> Self {
        Self {
            kind: NetworkInterfaceKind::Custom,
            custom: "127.0.0.1:16610".parse().unwrap(), // 默认Tondi devnet gRPC端口
        }
    }
}

impl From<NetworkInterfaceConfig> for ContextualNetAddress {
    fn from(network_interface_config: NetworkInterfaceConfig) -> Self {
        match network_interface_config.kind {
            NetworkInterfaceKind::Local => "127.0.0.1".parse().unwrap(),
            NetworkInterfaceKind::Any => "0.0.0.0".parse().unwrap(),
            NetworkInterfaceKind::Custom => network_interface_config.custom,
        }
    }
}

impl std::fmt::Display for NetworkInterfaceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ContextualNetAddress::from(self.clone()).fmt(f)
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum NodeConnectionConfigKind {
    #[default]
    PublicServerRandom,
    PublicServerCustom,
    Custom,
    // Local,
}

impl std::fmt::Display for NodeConnectionConfigKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeConnectionConfigKind::PublicServerRandom => {
                write!(f, "{}", i18n("Random Public Node"))
            }
            NodeConnectionConfigKind::PublicServerCustom => {
                write!(f, "{}", i18n("Custom Public Node"))
            }
            NodeConnectionConfigKind::Custom => write!(f, "{}", i18n("Custom")),
            // NodeConnectionConfigKind::Local => write!(f, "{}", i18n("Local")),
        }
    }
}

impl NodeConnectionConfigKind {
    pub fn iter() -> impl Iterator<Item = &'static NodeConnectionConfigKind> {
        [
            NodeConnectionConfigKind::PublicServerRandom,
            // NodeConnectionConfigKind::PublicServerCustom,
            NodeConnectionConfigKind::Custom,
            // NodeConnectionConfigKind::Local,
        ]
        .iter()
    }

    pub fn is_public(&self) -> bool {
        matches!(
            self,
            NodeConnectionConfigKind::PublicServerRandom
                | NodeConnectionConfigKind::PublicServerCustom
        )
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum NodeMemoryScale {
    #[default]
    Default,
    Conservative,
    Performance,
}

impl NodeMemoryScale {
    pub fn iter() -> impl Iterator<Item = &'static NodeMemoryScale> {
        [
            NodeMemoryScale::Default,
            NodeMemoryScale::Conservative,
            NodeMemoryScale::Performance,
        ]
        .iter()
    }
}

impl std::fmt::Display for NodeMemoryScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeMemoryScale::Default => write!(f, "{}", i18n("Default")),
            NodeMemoryScale::Conservative => write!(f, "{}", i18n("Conservative")),
            NodeMemoryScale::Performance => write!(f, "{}", i18n("Performance")),
        }
    }
}

impl NodeMemoryScale {
    pub fn describe(&self) -> &str {
        match self {
            NodeMemoryScale::Default => i18n("Managed by the Tondi Client daemon"),
            NodeMemoryScale::Conservative => i18n("Use 50%-75% of available system memory"),
            NodeMemoryScale::Performance => i18n("Use all available system memory"),
        }
    }

    pub fn get(&self) -> f64 {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {

                const GIGABYTE: u64 = 1024 * 1024 * 1024;
                const MEMORY_8GB: u64 = 8 * GIGABYTE;
                const MEMORY_16GB: u64 = 16 * GIGABYTE;
                const MEMORY_32GB: u64 = 32 * GIGABYTE;
                const MEMORY_64GB: u64 = 64 * GIGABYTE;
                const MEMORY_96GB: u64 = 96 * GIGABYTE;
                const MEMORY_128GB: u64 = 128 * GIGABYTE;

                let total_memory = runtime().system().as_ref().map(|system|system.total_memory).unwrap_or(MEMORY_16GB);

                let target_memory = if total_memory <= MEMORY_8GB {
                    MEMORY_8GB
                } else if total_memory <= MEMORY_16GB {
                    MEMORY_16GB
                } else if total_memory <= MEMORY_32GB {
                    MEMORY_32GB
                } else if total_memory <= MEMORY_64GB {
                    MEMORY_64GB
                } else if total_memory <= MEMORY_96GB {
                    MEMORY_96GB
                } else if total_memory <= MEMORY_128GB {
                    MEMORY_128GB
                } else {
                    MEMORY_16GB
                };

                match self {
                    NodeMemoryScale::Default => 1.0,
                    NodeMemoryScale::Conservative => match target_memory {
                        MEMORY_8GB => 0.3,
                        MEMORY_16GB => 1.0,
                        MEMORY_32GB => 1.5,
                        MEMORY_64GB => 2.0,
                        MEMORY_96GB => 3.0,
                        MEMORY_128GB => 4.0,
                        _ => 1.0,
                    },
                    NodeMemoryScale::Performance => match target_memory {
                        MEMORY_8GB => 0.4,
                        MEMORY_16GB => 1.0,
                        MEMORY_32GB => 2.0,
                        MEMORY_64GB => 4.0,
                        MEMORY_96GB => 6.0,
                        MEMORY_128GB => 8.0,
                        _ => 1.0,
                    },
                }
            } else {
                panic!("NodeMemoryScale::get() is not supported on this platform");
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeSettings {
    pub connection_config_kind: NodeConnectionConfigKind,
    pub rpc_kind: RpcKind,
    pub wrpc_url: String,
    #[serde(default)]
    pub enable_wrpc_borsh: bool,
    #[serde(default)]
    pub wrpc_borsh_network_interface: NetworkInterfaceConfig,
    pub wrpc_encoding: WrpcEncoding,
    pub enable_wrpc_json: bool,
    pub wrpc_json_network_interface: NetworkInterfaceConfig,
    pub enable_grpc: bool,
    pub grpc_network_interface: NetworkInterfaceConfig,
    pub enable_upnp: bool,
    pub memory_scale: NodeMemoryScale,

    pub network: Network,
    pub node_kind: TondidNodeKind,
    pub tondid_daemon_binary: String,
    pub tondid_daemon_args: String,
    pub tondid_daemon_args_enable: bool,
    #[serde(default)]
    pub tondid_daemon_storage_folder_enable: bool,
    #[serde(default)]
    pub tondid_daemon_storage_folder: String,
    #[serde(default)]
    pub devnet_custom_url: Option<String>,
}

impl Default for NodeSettings {
    fn default() -> Self {
        // 使用带端口的默认配置，便于连接本地或远程节点
        let default_grpc_interface = if cfg!(target_arch = "wasm32") {
            // Web版本使用本地地址
            NetworkInterfaceConfig {
                kind: NetworkInterfaceKind::Custom,
                custom: "127.0.0.1:16610".parse().unwrap(),
            }
        } else {
            // 桌面版本，Devnet默认使用远程节点
            NetworkInterfaceConfig {
                kind: NetworkInterfaceKind::Custom,
                custom: "8.210.45.192:16610".parse().unwrap(), // 默认连接到远程devnet节点
            }
        };
        
        println!("[NODESETTINGS DEBUG] NodeSettings::default() 被调用");
        println!("[NODESETTINGS DEBUG] default_grpc_interface: {:?}", default_grpc_interface);

        let settings = Self {
            connection_config_kind: NodeConnectionConfigKind::Custom,  // 改为Custom以启用自定义RPC配置
            rpc_kind: RpcKind::Grpc,  // 默认使用gRPC而不是Wrpc
            wrpc_url: "127.0.0.1".to_string(),
            wrpc_encoding: WrpcEncoding::Borsh,
            enable_wrpc_borsh: false,
            wrpc_borsh_network_interface: NetworkInterfaceConfig::default(),
            enable_wrpc_json: false,
            wrpc_json_network_interface: NetworkInterfaceConfig::default(),
            enable_grpc: true,  // 默认启用gRPC
            grpc_network_interface: default_grpc_interface,
            enable_upnp: true,
            memory_scale: NodeMemoryScale::default(),
            network: Network::Devnet,  // 改为Devnet
            node_kind: TondidNodeKind::Remote,  // 改为Remote以连接远程节点
            tondid_daemon_binary: String::default(),
            tondid_daemon_args: String::default(),
            tondid_daemon_args_enable: false,
            tondid_daemon_storage_folder_enable: false,
            tondid_daemon_storage_folder: String::default(),
            devnet_custom_url: Some("https://8.210.45.192/".to_string()),  // 设置远程devnet地址
        };
        
        println!("[NODESETTINGS DEBUG] 创建的settings.grpc_network_interface: {:?}", settings.grpc_network_interface);
        println!("[NODESETTINGS DEBUG] 创建的settings.network: {:?}", settings.network);
        println!("[NODESETTINGS DEBUG] 创建的settings.node_kind: {:?}", settings.node_kind);
        
        settings
    }
}

impl NodeSettings {
    /// 根据网络类型自动更新端口配置
    pub fn update_ports_for_network(&mut self) {
        println!("[UPDATE PORTS DEBUG] update_ports_for_network 被调用");
        println!("[UPDATE PORTS DEBUG] self.network: {:?}", self.network);
        println!("[UPDATE PORTS DEBUG] 调用前 grpc_network_interface: {:?}", self.grpc_network_interface);
        
        match self.network {
            Network::Mainnet => {
                // Mainnet: gRPC 16110, wRPC 17110
                if self.enable_grpc {
                    self.grpc_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.grpc_network_interface.custom = "127.0.0.1:16110".parse().unwrap();
                }
                if self.enable_wrpc_borsh {
                    self.wrpc_borsh_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.wrpc_borsh_network_interface.custom = "127.0.0.1:17110".parse().unwrap();
                }
                if self.enable_wrpc_json {
                    self.wrpc_json_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.wrpc_json_network_interface.custom = "127.0.0.1:18110".parse().unwrap();
                }
            }
            Network::Testnet => {
                // Testnet: gRPC 16210, wRPC 17210
                if self.enable_grpc {
                    self.grpc_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.grpc_network_interface.custom = "127.0.0.1:16210".parse().unwrap();
                }
                if self.enable_wrpc_borsh {
                    self.wrpc_borsh_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.wrpc_borsh_network_interface.custom = "127.0.0.1:17210".parse().unwrap();
                }
                if self.enable_wrpc_json {
                    self.wrpc_json_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.wrpc_json_network_interface.custom = "127.0.0.1:18210".parse().unwrap();
                }
            }
            Network::Devnet => {
                // Devnet: gRPC 16610, wRPC 17610
                if self.network == Network::Devnet {
                    // 对于devnet，默认连接到远程节点
                    println!("[UPDATE PORTS DEBUG] 处理Devnet配置");
                    self.grpc_network_interface.kind = NetworkInterfaceKind::Custom;
                    self.grpc_network_interface.custom = "8.210.45.192:16610".parse().unwrap();
                    println!("[UPDATE PORTS DEBUG] 设置grpc_network_interface: {:?}", self.grpc_network_interface);
                    
                    if self.enable_wrpc_borsh {
                        self.wrpc_borsh_network_interface.kind = NetworkInterfaceKind::Custom;
                        self.wrpc_borsh_network_interface.custom = "8.210.45.192:17610".parse().unwrap();
                    }
                    if self.enable_wrpc_json {
                        self.wrpc_json_network_interface.kind = NetworkInterfaceKind::Custom;
                        self.wrpc_json_network_interface.custom = "8.210.45.192:18610".parse().unwrap();
                    }
                }
            }

        }
    }

    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            #[allow(clippy::if_same_then_else)]
            pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
                if self.network != other.network {
                    Some(true)
                } else if self.node_kind != other.node_kind {
                    Some(true)
                } else if self.memory_scale != other.memory_scale {
                    Some(true)
                } else if self.connection_config_kind != other.connection_config_kind
                {
                    Some(true)
                } else if self.tondid_daemon_storage_folder_enable != other.tondid_daemon_storage_folder_enable
                    || other.tondid_daemon_storage_folder_enable && (self.tondid_daemon_storage_folder != other.tondid_daemon_storage_folder)
                {
                    Some(true)
                } else if self.enable_grpc != other.enable_grpc
                    || self.grpc_network_interface != other.grpc_network_interface
                    || self.wrpc_url != other.wrpc_url
                    || self.wrpc_encoding != other.wrpc_encoding
                    || self.enable_wrpc_json != other.enable_wrpc_json
                    || self.wrpc_json_network_interface != other.wrpc_json_network_interface
                    || self.enable_upnp != other.enable_upnp
                {
                    Some(self.node_kind != TondidNodeKind::IntegratedInProc)
                } else if self.tondid_daemon_args != other.tondid_daemon_args
                    || self.tondid_daemon_args_enable != other.tondid_daemon_args_enable
                {
                    Some(self.node_kind.is_config_capable())
                } else if self.tondid_daemon_binary != other.tondid_daemon_binary {
                    Some(self.node_kind == TondidNodeKind::ExternalAsDaemon)
                } else if self.devnet_custom_url != other.devnet_custom_url {
                    Some(true)
                } else {
                    None
                }
            }
        } else {
            #[allow(clippy::if_same_then_else)]
            pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
                if self.network != other.network {
                    Some(true)
                } else if self.node_kind != other.node_kind {
                    Some(true)
                } else if self.connection_config_kind != other.connection_config_kind {
                    Some(true)
                } else if self.rpc_kind != other.rpc_kind
                    || self.wrpc_url != other.wrpc_url
                    || self.wrpc_encoding != other.wrpc_encoding
                {
                    Some(true)
                } else if self.devnet_custom_url != other.devnet_custom_url {
                    Some(true)
                } else {
                    None
                }
            }

        }
    }
}

impl RpcConfig {
    pub fn from_node_settings(settings: &NodeSettings, _options: Option<RpcOptions>) -> Self {
        match settings.connection_config_kind {
            NodeConnectionConfigKind::Custom => match settings.rpc_kind {
                RpcKind::Wrpc => RpcConfig::Wrpc {
                    url: Some(settings.wrpc_url.clone()),
                    encoding: settings.wrpc_encoding,
                    resolver_urls: None,
                },
                RpcKind::Grpc => RpcConfig::Grpc {
                    url: Some(settings.grpc_network_interface.clone()),
                },
            },
            NodeConnectionConfigKind::PublicServerCustom
            | NodeConnectionConfigKind::PublicServerRandom => RpcConfig::Wrpc {
                url: None,
                encoding: settings.wrpc_encoding,
                resolver_urls: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetricsSettings {
    pub graph_columns: usize,
    pub graph_height: usize,
    pub graph_range_from: isize,
    pub graph_range_to: isize,
    pub disabled: AHashSet<Metric>,
}

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            graph_columns: 3,
            graph_height: 90,
            graph_range_from: -15 * 60,
            graph_range_to: 0,
            disabled: AHashSet::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserInterfaceSettings {
    pub theme_color: String,
    pub theme_style: String,
    pub scale: f32,
    pub metrics: MetricsSettings,
    pub balance_padding: bool,
    #[serde(default)]
    pub disable_frame: bool,
}

impl Default for UserInterfaceSettings {
    fn default() -> Self {
        // cfg_if! {
        //     if #[cfg(target_os = "windows")] {
        //         let disable_frame = true;
        //     } else {
        //         let disable_frame = false;
        //     }
        // }

        Self {
            theme_color: "Tondi".to_string(),
            theme_style: "Rounded".to_string(),
            scale: 1.0,
            metrics: MetricsSettings::default(),
            balance_padding: true,
            disable_frame: true,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeveloperSettings {
    pub enable: bool,
    pub enable_screen_capture: bool,
    pub disable_password_restrictions: bool,
    pub enable_experimental_features: bool,
    pub enable_custom_daemon_args: bool,
    pub market_monitor_on_testnet: bool,
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            enable: false,
            enable_screen_capture: true,
            disable_password_restrictions: false,
            enable_experimental_features: false,
            enable_custom_daemon_args: true,
            market_monitor_on_testnet: false,
        }
    }
}

impl DeveloperSettings {
    pub fn screen_capture_enabled(&self) -> bool {
        self.enable && self.enable_screen_capture
    }

    pub fn password_restrictions_disabled(&self) -> bool {
        self.enable && self.disable_password_restrictions
    }

    pub fn experimental_features_enabled(&self) -> bool {
        self.enable && self.enable_experimental_features
    }

    pub fn custom_daemon_args_enabled(&self) -> bool {
        self.enable && self.enable_custom_daemon_args
    }
}

#[derive(Describe, Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EstimatorMode {
    #[describe("Fee Market Only")]
    FeeMarketOnly,
    #[default]
    #[describe("Fee Market & Network Pressure")]
    NetworkPressure,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EstimatorSettings {
    pub mode: EstimatorMode,
}

impl Default for EstimatorSettings {
    fn default() -> Self {
        Self {
            mode: EstimatorMode::NetworkPressure,
        }
    }
}

impl EstimatorSettings {
    pub fn track_network_load(&self) -> EstimatorMode {
        self.mode
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    pub revision: String,
    pub initialized: bool,
    pub splash_screen: bool,
    pub version: String,
    pub update: String,
    pub developer: DeveloperSettings,
    #[serde(default)]
    pub estimator: EstimatorSettings,
    pub node: NodeSettings,
    pub user_interface: UserInterfaceSettings,
    pub language_code: String,
    pub update_monitor: bool,
    pub market_monitor: bool,
    pub update_check_timeout: u64, // 更新检查超时时间（秒）
    pub update_check_retries: u32, // 更新检查重试次数
    pub update_check_interval: u64, // 更新检查间隔（秒）
    // #[serde(default)]
    // pub disable_frame: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            initialized: false,
            revision: SETTINGS_REVISION.to_string(),

            splash_screen: true,
            version: "0.0.0".to_string(),
            update: crate::app::VERSION.to_string(),
            developer: DeveloperSettings::default(),
            estimator: EstimatorSettings::default(),
            node: NodeSettings::default(),
            user_interface: UserInterfaceSettings::default(),
            language_code: "en".to_string(),
            update_monitor: true,
            market_monitor: true,
            update_check_timeout: 30, // 默认30秒超时
            update_check_retries: 3,  // 默认3次重试
            update_check_interval: 60 * 60 * 12, // 默认12小时检查一次
            // disable_frame: false,
        }
    }
}

impl Settings {}

fn storage() -> Result<Storage> {
    Ok(Storage::try_new("tondi-ng.settings")?)
}

impl Settings {
    pub async fn store(&self) -> Result<()> {
        let storage = storage()?;
        storage.ensure_dir().await?;
        workflow_store::fs::write_json(storage.filename(), self).await?;
        Ok(())
    }

    pub fn store_sync(&self) -> Result<&Self> {
        let storage = storage()?;
        if runtime::is_chrome_extension() {
            let this = self.clone();
            spawn(async move {
                if let Err(err) = workflow_store::fs::write_json(storage.filename(), &this).await {
                    log_error!("Settings::store_sync() error: {}", err);
                }
            });
        } else {
            storage.ensure_dir_sync()?;
            workflow_store::fs::write_json_sync(storage.filename(), self)?;
        }
        Ok(self)
    }

    pub async fn load() -> Result<Self> {
        use workflow_store::fs::read_json;

        let storage = storage()?;
        if storage.exists().await.unwrap_or(false) {
            match read_json::<Self>(storage.filename()).await {
                Ok(mut settings) => {
                    if settings.revision != SETTINGS_REVISION {
                        Ok(Self::default())
                    } else {
                        if matches!(
                            settings.node.connection_config_kind,
                            NodeConnectionConfigKind::PublicServerCustom
                        ) {
                            settings.node.connection_config_kind =
                                NodeConnectionConfigKind::PublicServerRandom;
                        }

                        Ok(settings)
                    }
                }
                Err(error) => {
                    #[allow(clippy::if_same_then_else)]
                    if matches!(error, workflow_store::error::Error::SerdeJson(..)) {
                        // TODO - recovery process
                        log_warn!("Settings::load() error: {}", error);
                        Ok(Self::default())
                    } else {
                        log_warn!("Settings::load() error: {}", error);
                        Ok(Self::default())
                    }
                }
            }
        } else {
            Ok(Self::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_ports_for_network() {
        let mut settings = NodeSettings::default();
        
        // 启用所有 RPC 类型以便测试
        settings.enable_grpc = true;
        settings.enable_wrpc_borsh = true;
        settings.enable_wrpc_json = true;
        
        // 测试 Mainnet 端口
        settings.network = Network::Mainnet;
        settings.update_ports_for_network();
        assert_eq!(settings.grpc_network_interface.custom.to_string(), "127.0.0.1:16110");
        assert_eq!(settings.wrpc_borsh_network_interface.custom.to_string(), "127.0.0.1:17110");
        assert_eq!(settings.wrpc_json_network_interface.custom.to_string(), "127.0.0.1:18110");
        
        // 测试 Testnet 端口
        settings.network = Network::Testnet;
        settings.update_ports_for_network();
        assert_eq!(settings.grpc_network_interface.custom.to_string(), "127.0.0.1:16210");
        assert_eq!(settings.wrpc_borsh_network_interface.custom.to_string(), "127.0.0.1:17210");
        assert_eq!(settings.wrpc_json_network_interface.custom.to_string(), "127.0.0.1:18210");
        
        // 测试 Devnet 端口
        settings.network = Network::Devnet;
        settings.update_ports_for_network();
        assert_eq!(settings.grpc_network_interface.custom.to_string(), "8.210.45.192:16610");
        assert_eq!(settings.wrpc_borsh_network_interface.custom.to_string(), "8.210.45.192:17610");
        assert_eq!(settings.wrpc_json_network_interface.custom.to_string(), "8.210.45.192:18610");
    }
}
