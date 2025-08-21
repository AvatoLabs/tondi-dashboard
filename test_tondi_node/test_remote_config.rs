use std::net::SocketAddr;
use std::str::FromStr;

// 模拟tondi dashboard的配置结构
#[derive(Debug, Clone)]
pub enum NetworkInterfaceKind {
    Local,
    Any,
    Custom,
}

#[derive(Debug, Clone)]
pub struct NetworkInterfaceConfig {
    pub kind: NetworkInterfaceKind,
    pub custom: String,
}

#[derive(Debug, Clone)]
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,
}

#[derive(Debug, Clone)]
pub struct NodeSettings {
    pub network: Network,
    pub enable_grpc: bool,
    pub grpc_network_interface: NetworkInterfaceConfig,
    pub devnet_custom_url: Option<String>,
}

impl NodeSettings {
    pub fn new_devnet_remote() -> Self {
        Self {
            network: Network::Devnet,
            enable_grpc: true,
            grpc_network_interface: NetworkInterfaceConfig {
                kind: NetworkInterfaceKind::Custom,
                custom: "8.210.45.192:16610".to_string(),
            },
            devnet_custom_url: Some("8.210.45.192:16610".to_string()),
        }
    }
    
    pub fn new_devnet_local() -> Self {
        Self {
            network: Network::Devnet,
            enable_grpc: true,
            grpc_network_interface: NetworkInterfaceConfig {
                kind: NetworkInterfaceKind::Local,
                custom: "127.0.0.1:16610".to_string(),
            },
            devnet_custom_url: None,
        }
    }
    
    pub fn validate_connection(&self) -> Result<(), String> {
        // 验证gRPC接口配置
        if self.enable_grpc {
            match &self.grpc_network_interface.kind {
                NetworkInterfaceKind::Custom => {
                    // 验证自定义地址格式
                    if let Err(e) = SocketAddr::from_str(&self.grpc_network_interface.custom) {
                        return Err(format!("Invalid gRPC address: {}", e));
                    }
                    println!("✅ gRPC接口配置有效: {}", self.grpc_network_interface.custom);
                }
                NetworkInterfaceKind::Local => {
                    println!("✅ gRPC接口配置为本地: {}", self.grpc_network_interface.custom);
                }
                NetworkInterfaceKind::Any => {
                    println!("✅ gRPC接口配置为任意: {}", self.grpc_network_interface.custom);
                }
            }
        }
        
        // 验证devnet自定义URL
        if let Some(url) = &self.devnet_custom_url {
            if let Err(e) = SocketAddr::from_str(url) {
                return Err(format!("Invalid devnet URL: {}", e));
            }
            println!("✅ Devnet自定义URL配置有效: {}", url);
        }
        
        Ok(())
    }
    
    pub fn get_connection_info(&self) -> String {
        match self.network {
            Network::Devnet => {
                if let Some(url) = &self.devnet_custom_url {
                    format!("连接到远端Devnet节点: {}", url)
                } else {
                    format!("连接到本地Devnet节点: {}", self.grpc_network_interface.custom)
                }
            }
            Network::Testnet => "连接到Testnet节点".to_string(),
            Network::Mainnet => "连接到Mainnet节点".to_string(),
        }
    }
}

fn main() {
    println!("测试Tondi Devnet节点配置...\n");
    
    // 测试远端devnet配置
    println!("1. 测试远端Devnet配置:");
    let remote_config = NodeSettings::new_devnet_remote();
    println!("配置: {:?}", remote_config);
    
    match remote_config.validate_connection() {
        Ok(_) => println!("✅ 远端配置验证成功"),
        Err(e) => println!("❌ 远端配置验证失败: {}", e),
    }
    
    println!("连接信息: {}", remote_config.get_connection_info());
    
    println!("\n2. 测试本地Devnet配置:");
    let local_config = NodeSettings::new_devnet_local();
    println!("配置: {:?}", local_config);
    
    match local_config.validate_connection() {
        Ok(_) => println!("✅ 本地配置验证成功"),
        Err(e) => println!("❌ 本地配置验证失败: {}", e),
    }
    
    println!("连接信息: {}", local_config.get_connection_info());
    
    println!("\n配置测试完成！");
    println!("要连接到您的远端节点 8.210.45.192:16610，请使用远端配置。");
}
