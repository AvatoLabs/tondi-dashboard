# Tondi Devnet节点问题诊断和解决方案

## 🔍 问题分析

### 当前状态
- ✅ **网络连接正常**：所有端口可达，延迟稳定(37ms)
- ✅ **节点服务运行**：成功读取到27字节gRPC数据
- ❌ **Dashboard无法获取metrics**：PEERS 0, BLOCKS 0, HEADERS 0

### 问题诊断
基于测试结果和dashboard日志分析，主要问题可能是：

1. **协议配置不匹配**
2. **节点还在同步中**
3. **API版本兼容性问题**

## 🛠️ 解决方案

### 方案1: 检查并修正网络配置

在tondi dashboard中，确保以下配置：

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

### 方案2: 检查节点同步状态

您的节点可能还在同步中。建议：

1. **等待节点完全同步**：devnet节点首次启动需要时间同步
2. **检查节点日志**：查看是否有同步进度信息
3. **验证区块数据**：确认节点是否已经下载了区块

### 方案3: 尝试wRPC连接

如果gRPC有问题，可以尝试wRPC：

```rust
// 启用wRPC Borsh
settings.node.enable_wrpc_borsh = true;
settings.node.wrpc_borsh_network_interface = NetworkInterfaceConfig {
    kind: NetworkInterfaceKind::Custom,
    custom: "8.210.45.192:17610".parse().unwrap(),
};
```

### 方案4: 检查API兼容性

基于bp-tondi示例，正确的API调用应该是：

```rust
// 连接到gRPC服务
let client = TondiClient::connect("8.210.45.192:16610").await?;

// 获取服务器信息
let info = client.get_server_info().await?;

// 获取区块信息
let blocks = client.get_blocks(None, true, true).await?;
```

## 🔧 具体操作步骤

### 1. 在Dashboard中配置
1. 打开Settings → Node Settings
2. 选择Network: Devnet
3. 在"Devnet custom URL"中输入：`8.210.45.192:16610`
4. 确保"Enable gRPC"已勾选
5. 保存设置并重启dashboard

### 2. 检查连接状态
1. 查看dashboard日志中的连接信息
2. 确认是否显示"Connected to remote devnet node"
3. 检查是否有错误信息

### 3. 等待节点同步
1. 如果节点还在同步，等待一段时间
2. 检查节点日志中的同步进度
3. 确认是否有区块数据

## 📊 预期结果

配置正确后，您应该看到：

- ✅ Dashboard成功连接到远端devnet节点
- ✅ 显示实际的PEERS数量（而不是0）
- ✅ 显示实际的BLOCKS数量（而不是0）
- ✅ 显示实际的HEADERS数量（而不是0）
- ✅ Metrics信息正常更新

## 🚨 如果问题仍然存在

1. **检查节点状态**：确认8.210.45.192:16610上的服务完全启动
2. **查看节点日志**：了解是否有错误或警告信息
3. **验证网络配置**：确认防火墙和网络设置
4. **尝试其他端口**：测试16611(P2P)和17610(wRPC)端口

## 📞 技术支持

如果按照以上步骤仍然无法解决问题，请提供：
1. Dashboard的完整错误日志
2. 节点的配置信息
3. 网络连接测试结果

---

**总结**：您的节点运行正常，问题主要在于配置和协议匹配。按照上述步骤配置后，应该能够正常获取metrics信息。
