# Tondi Dashboard gRPC 集成

本文档描述了如何在 Tondi Dashboard 中使用 gRPC 客户端功能。

## 概述

Tondi Dashboard 现在支持通过 gRPC 协议连接到 Tondi 节点，这是对现有 wRPC 支持的补充。gRPC 支持通过集成 `bp-tondi-client` 项目实现。

## 功能特性

- 支持 gRPC 协议连接
- 与现有 wRPC 客户端接口兼容
- 支持基本的 RPC 方法调用
- 自动类型转换和错误处理

## 配置

### 启用 gRPC 支持

在配置文件中，将 RPC 类型设置为 `Grpc`：

```toml
[node]
rpc_type = "grpc"
rpc_config = { type = "grpc", url = { type = "custom", custom = "127.0.0.1:16610" } }
```

### 默认配置

如果不指定 URL，gRPC 客户端将默认连接到 `127.0.0.1:16610`（可通过 `DEFAULT_GRPC_URL` 常量配置）。

## 支持的 RPC 方法

目前支持以下 gRPC RPC 方法：

- `get_server_info()` - 获取服务器信息
- `get_block()` - 获取区块信息
- `get_transaction()` - 获取交易信息
- `get_blocks()` - 获取区块列表
- `get_block_status()` - 获取区块状态
- `get_header()` - 获取区块头

## 使用方法

### 1. 创建 gRPC 客户端

```rust
use tondi_dashboard_core::runtime::services::tondi::TondiGrpcClient;

let client = TondiGrpcClient::connect(DEFAULT_GRPC_URL).await?;
```

### 2. 调用 RPC 方法

```rust
// 获取服务器信息
let server_info = client.get_server_info().await?;

// 获取区块信息
let block = client.get_block(hash, true).await?;

// 获取交易信息
let transaction = client.get_transaction(tx_hash).await?;
```

## 架构

### 主要组件

1. **TondiGrpcClient** - gRPC 客户端实现
2. **GrpcRpcCtl** - gRPC RPC 控制实现
3. **类型转换层** - 将 bp-tondi-client 响应转换为 tondi-rpc-core 类型

### 集成点

- `core/src/runtime/services/tondi/grpc_client.rs` - gRPC 客户端实现
- `core/src/runtime/services/tondi/mod.rs` - 集成到现有 RPC 系统

## 依赖关系

- `bp-tondi` - 来自 bp-tondi-client 项目
- `tondi-grpc-client` - 核心 gRPC 客户端库
- `tondi-rpc-core` - RPC 核心类型定义

## 限制和注意事项

1. **功能支持** - 某些高级功能（如 metrics、fee estimation）可能不完全支持
2. **类型转换** - 需要处理 bp-tondi-client 和 tondi-rpc-core 之间的类型差异
3. **错误处理** - gRPC 特定的错误需要转换为通用错误类型

## 开发状态

- [x] 基本 gRPC 客户端结构
- [x] RPC 控制接口实现
- [x] 与现有系统的集成
- [ ] 完整的类型转换实现
- [ ] 错误处理优化
- [ ] 性能测试和优化

## 故障排除

### 常见问题

1. **连接失败** - 检查 gRPC 服务器是否运行在指定端口
2. **类型错误** - 确保 bp-tondi-client 版本兼容
3. **编译错误** - 检查依赖路径和版本

### 调试

启用详细日志记录：

```rust
use log::LevelFilter;
log::set_max_level(LevelFilter::Debug);
```

## 贡献

欢迎贡献代码来改进 gRPC 集成功能。请确保：

1. 遵循现有的代码风格
2. 添加适当的测试
3. 更新相关文档
4. 处理错误情况

## 许可证

本功能遵循与 Tondi Dashboard 相同的许可证。
