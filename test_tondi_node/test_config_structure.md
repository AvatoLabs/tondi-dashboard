# Tondi Devnet 节点配置结构分析

## 当前配置结构

### 1. NodeSettings 结构体
```rust
pub struct NodeSettings {
    pub connection_config_kind: NodeConnectionConfigKind,  // 连接配置类型
    pub rpc_kind: RpcKind,                               // RPC类型 (gRPC/wRPC)
    pub enable_grpc: bool,                               // 是否启用gRPC
    pub grpc_network_interface: NetworkInterfaceConfig,  // gRPC网络接口配置
    pub enable_wrpc_borsh: bool,                         // 是否启用wRPC Borsh
    pub wrpc_borsh_network_interface: NetworkInterfaceConfig, // wRPC Borsh网络接口
    pub network: Network,                                // 网络类型 (Mainnet/Testnet/Devnet)
    pub devnet_custom_url: Option<String>,               // 自定义devnet URL
    // ... 其他字段
}
```

### 2. NetworkInterfaceConfig 结构体
```rust
pub struct NetworkInterfaceConfig {
    pub kind: NetworkInterfaceKind,                      // 接口类型
    pub custom: ContextualNetAddress,                    // 自定义地址
}

pub enum NetworkInterfaceKind {
    Local,    // 127.0.0.1
    Any,      // 0.0.0.0
    Custom,   // 自定义地址
}
```

### 3. 网络类型对应的默认端口
- **Mainnet**: gRPC 16110, wRPC 17110
- **Testnet**: gRPC 16210, wRPC 17210  
- **Devnet**: gRPC 16610, wRPC 17610

## 配置远端Devnet节点的方法

### 方法1: 使用 devnet_custom_url
```rust
// 设置自定义devnet URL
settings.node.devnet_custom_url = Some("8.210.45.192:16610".to_string());
```

### 方法2: 直接修改 gRPC 网络接口
```rust
// 设置gRPC接口为远端地址
settings.node.grpc_network_interface = NetworkInterfaceConfig {
    kind: NetworkInterfaceKind::Custom,
    custom: "8.210.45.192:16610".parse().unwrap(),
};
```

## 回答用户问题

### Q: 现在的设置结构体和UI设置，可以正常配置远端devnet吗？
**A: 是的，可以！** 当前的结构支持两种方式配置远端devnet：

1. **通过UI**: 在设置界面中，当选择Devnet网络时，会显示"Devnet custom URL"配置选项
2. **通过代码**: 可以直接修改`grpc_network_interface`或`devnet_custom_url`字段

### Q: gRPC是应该听这个本地节点地址还是远端的？
**A: 这取决于您的使用场景：**

1. **如果您要连接到远端节点** (如您的 8.210.45.192:16610):
   - `grpc_network_interface` 应该设置为 `8.210.45.192:16610`
   - 或者使用 `devnet_custom_url = "8.210.45.192:16610"`

2. **如果您要运行本地节点**:
   - `grpc_network_interface` 应该设置为 `127.0.0.1:16610` (本地devnet端口)

## 推荐配置

对于您的远端devnet节点，建议使用以下配置：

```rust
// 设置网络类型为Devnet
settings.node.network = Network::Devnet;

// 启用gRPC
settings.node.enable_grpc = true;

// 设置gRPC接口为远端地址
settings.node.grpc_network_interface = NetworkInterfaceConfig {
    kind: NetworkInterfaceKind::Custom,
    custom: "8.210.45.192:16610".parse().unwrap(),
};

// 或者使用devnet_custom_url
settings.node.devnet_custom_url = Some("8.210.45.192:16610".to_string());
```

这样配置后，tondi dashboard就会连接到您的远端devnet节点，而不是尝试启动本地节点。
