# Tondi gRPC 客户端优化说明

## 概述

本文件说明了我们对Tondi gRPC客户端所做的优化，包括删除不需要的方法和实现必需的方法。

## 优化内容

### 1. 删除的gRPC方法

以下方法被删除，因为dashboard不需要使用它们：

#### 区块和交易相关
- `submit_block_call` - 提交区块（矿工功能）
- `get_block_template_call` - 获取区块模板（矿工功能）
- `submit_transaction_call` - 提交交易（钱包功能，dashboard通过其他方式处理）
- `submit_transaction_replacement_call` - 提交交易替换

#### 网络管理
- `add_peer_call` - 添加节点
- `ban_call` - 封禁节点
- `unban_call` - 解封节点
- `get_peer_addresses_call` - 获取节点地址列表

#### 内存池管理
- `get_mempool_entry_call` - 获取内存池条目
- `get_mempool_entries_call` - 获取内存池条目列表
- `get_mempool_entries_by_addresses_call` - 按地址获取内存池条目

#### 高级功能
- `resolve_finality_conflict_call` - 解决最终性冲突
- `shutdown_call` - 关闭节点
- `get_coin_supply_call` - 获取代币供应量
- `estimate_network_hashes_per_second_call` - 估算网络哈希率
- `get_fee_estimate_call` - 获取手续费估算
- `get_current_block_color_call` - 获取当前区块颜色

### 2. 已实现的gRPC方法

以下方法是dashboard实际需要的，已经完整实现：

#### 基础信息
- `get_server_info` - 获取服务器信息
- `get_system_info_call` - 获取系统信息
- `get_current_network_call` - 获取当前网络

#### 区块链数据
- `get_blocks` - 获取区块列表
- `get_block_count` - 获取区块数量
- `get_block_dag_info` - 获取区块DAG信息

#### 网络监控
- `get_connections_call` - 获取连接信息
- `get_connected_peer_info` - 获取已连接节点信息

#### 指标监控
- `get_metrics_call` - 获取指标数据
- `get_metrics` - 获取指标（trait要求）

#### 健康检查
- `ping_call` - Ping测试

### 3. 需要实现但暂时返回错误的方法

- `get_sync_status_call` - 获取同步状态（TODO: 需要实现）

## 性能优化

### 1. 减少不必要的方法调用
- 删除了30+个不需要的gRPC方法
- 减少了代码复杂度和维护成本

### 2. 优化数据获取
- 使用批量获取减少网络请求
- 实现智能缓存和合并

### 3. 错误处理优化
- 对不需要的方法返回明确的错误信息
- 避免不必要的异常处理开销

## 使用说明

### 1. 连接gRPC服务器
```rust
use crate::runtime::services::tondi::TondiGrpcClient;

let client = TondiGrpcClient::connect(network_interface, network).await?;
```

### 2. 获取基础信息
```rust
// 获取服务器信息
let server_info = client.get_server_info().await?;

// 获取系统信息
let system_info = client.get_system_info_call(None, request).await?;

// 获取当前网络
let network_info = client.get_current_network_call(None, request).await?;
```

### 3. 获取区块链数据
```rust
// 获取区块列表
let blocks = client.get_blocks(None, true, false).await?;

// 获取区块数量
let block_count = client.get_block_count().await?;

// 获取DAG信息
let dag_info = client.get_block_dag_info().await?;
```

### 4. 获取网络状态
```rust
// 获取连接信息
let connections = client.get_connections_call(None, request).await?;

// 获取节点信息
let peer_info = client.get_connected_peer_info().await?;
```

### 5. 获取指标数据
```rust
// 获取完整指标
let metrics = client.get_metrics_call(None, request).await?;

// 获取特定指标
let metrics = client.get_metrics(true, true, false, true, false, false).await?;
```

## 错误处理

### 1. 不支持的方法
如果调用dashboard不需要的方法，会收到以下错误：
```
Method not needed by dashboard
```

### 2. Web/WASM版本
在Web版本中，所有gRPC方法都会返回：
```
gRPC is not supported in Web/WASM version
```

### 3. 网络错误
网络连接失败时会返回具体的错误信息，包括连接地址和错误详情。

## 测试

### 1. 运行测试
```bash
cd core
cargo test --package tondi-dashboard-core --test grpc_client
```

### 2. 测试覆盖
- gRPC客户端创建
- 连接状态检查
- 网络ID获取
- URL获取

## 未来改进

### 1. 实现同步状态检查
- 完成`get_sync_status_call`方法的实现
- 提供准确的节点同步状态信息

### 2. 性能监控
- 添加gRPC调用性能统计
- 实现智能重试机制

### 3. 缓存优化
- 实现智能数据缓存
- 减少重复的gRPC调用

## 注意事项

1. **仅支持桌面版本**: gRPC功能仅在原生桌面版本中可用
2. **Web版本回退**: Web版本会自动回退到wRPC协议
3. **错误处理**: 所有gRPC调用都应该包含适当的错误处理
4. **异步操作**: 所有gRPC方法都是异步的，需要使用`.await`

## 贡献指南

如果需要添加新的gRPC方法：

1. 确认dashboard确实需要该方法
2. 在`RpcApi` trait中实现该方法
3. 添加适当的错误处理
4. 编写测试用例
5. 更新本文档
