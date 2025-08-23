# Wallet Endpoint Testing Guide

本指南介绍如何使用Tondi Dashboard的钱包端点测试功能，从钱包创建到完整的交互测试。

## 概述

我们创建了一个完整的钱包端点测试系统，包括：

1. **图形界面测试模块** - 在Dashboard界面中进行交互式测试
2. **集成测试框架** - 自动化的全面测试套件
3. **命令行测试工具** - 独立的CLI测试工具

## 测试功能

### 🔐 钱包核心功能测试
- ✅ 钱包创建和初始化
- ✅ 助记词生成和验证
- ✅ 账户创建和管理
- ✅ 私钥数据管理

### 💰 交易和支付测试
- ✅ 余额查询
- ✅ 地址生成（接收/找零）
- ✅ UTXO查询和管理
- ✅ 交易估算
- ✅ 费用估算
- ✅ 交易发送（模拟）

### 🌐 网络和RPC测试
- ✅ RPC连接测试
- ✅ 服务器信息查询
- ✅ 区块链数据查询
- ✅ 对等节点信息
- ✅ 内存池查询
- ✅ 网络同步状态

## 使用方法

### 1. 图形界面测试（推荐）

在Tondi Dashboard中：

1. 打开Dashboard应用
2. 导航到"Wallet Endpoint Test"模块
3. 配置测试参数：
   - 钱包名称
   - 密码
   - 测试地址
   - 测试金额
4. 选择要运行的测试：
   - 单个功能测试
   - 综合测试套件

#### 测试配置示例
```
Wallet Name: test_wallet
Password: test123456
Test Address: tondi:qzgyhexvcaasfdawmghcavhx0qxgpat7d2uxzx5k2k6dzalr2grs20j6hwrgtt
Test Amount: 0.01 TONDI
```

### 2. 命令行测试

#### 快速连接测试
```bash
cargo run --bin wallet_test_cli quick
```

#### 完整测试套件
```bash
cargo run --bin wallet_test_cli run \
  --wallet-name "my_test_wallet" \
  --password "secure_password" \
  --test-address "tondi:your_test_address" \
  --amount 0.01 \
  --network testnet-10
```

#### 特定功能测试
```bash
# 仅测试钱包功能
cargo run --bin wallet_test_cli wallet

# 仅测试RPC端点
cargo run --bin wallet_test_cli rpc
```

### 3. 编程接口测试

```rust
use tondi_dashboard_core::tests::wallet_endpoint_integration_tests::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用默认配置
    let results = run_wallet_integration_tests().await?;
    
    // 或使用自定义配置
    let config = TestConfig {
        wallet_name: "custom_test".to_string(),
        password: "my_password".to_string(),
        test_address: "tondi:your_address".to_string(),
        test_amount_tondi: 0.1,
        network: Network::Testnet10,
        timeout_seconds: 60,
    };
    
    let results = run_wallet_integration_tests_with_config(config).await?;
    
    // 处理测试结果
    for result in results {
        println!("{}: {} - {}", 
            result.name, 
            if result.success { "PASS" } else { "FAIL" },
            result.message
        );
    }
    
    Ok(())
}
```

## 测试用例详解

### 1. RPC连接测试
- **目的**: 验证与Tondi节点的基本连接
- **测试内容**: 服务器信息、区块数量、网络状态
- **预期结果**: 成功连接并获取基本信息

### 2. 钱包创建测试
- **目的**: 验证钱包的完整创建流程
- **测试内容**: 
  - 生成12词助记词
  - 创建钱包描述符
  - 创建私钥数据
  - 创建默认账户
  - 数据持久化
- **预期结果**: 钱包和账户创建成功

### 3. 账户管理测试
- **目的**: 验证账户管理功能
- **测试内容**:
  - 账户列表查询
  - 账户激活
  - 账户状态管理
- **预期结果**: 账户正确列出和激活

### 4. 余额查询测试
- **目的**: 验证账户余额查询功能
- **测试内容**:
  - 成熟余额查询
  - 待确认余额查询
  - UTXO统计
- **预期结果**: 返回准确的余额信息

### 5. 地址生成测试
- **目的**: 验证地址生成功能
- **测试内容**:
  - 接收地址生成
  - 找零地址生成
  - 地址格式验证
- **预期结果**: 生成有效的Tondi地址

### 6. UTXO查询测试
- **目的**: 验证UTXO管理功能
- **测试内容**:
  - UTXO列表查询
  - UTXO值统计
  - UTXO状态验证
- **预期结果**: 返回准确的UTXO信息

### 7. 交易估算测试
- **目的**: 验证交易构建和估算
- **测试内容**:
  - 交易输出构建
  - 费用估算
  - 总金额计算
- **预期结果**: 准确的交易估算

### 8. 费用估算测试
- **目的**: 验证网络费用估算
- **测试内容**:
  - 标准费用估算
  - 实验性费用估算
  - 优先级费用计算
- **预期结果**: 合理的费用建议

### 9. 网络信息测试
- **目的**: 验证网络状态查询
- **测试内容**:
  - 同步状态
  - 网络类型
  - UTXO索引状态
- **预期结果**: 准确的网络信息

### 10. 对等节点测试
- **目的**: 验证P2P网络信息
- **测试内容**:
  - 连接的对等节点
  - 已知节点地址
  - 禁用节点列表
- **预期结果**: 网络连接信息

### 11. 区块链数据测试
- **目的**: 验证区块链数据查询
- **测试内容**:
  - 区块数量
  - DAG信息
  - 币供应量
- **预期结果**: 最新的区块链数据

### 12. 内存池测试
- **目的**: 验证内存池查询功能
- **测试内容**:
  - 待处理交易列表
  - 地址相关交易
  - 孤儿交易池
- **预期结果**: 内存池状态信息

## 故障排除

### 常见问题

#### 1. RPC连接失败
**症状**: `No RPC connection available`
**解决方案**:
- 确保Tondi节点正在运行
- 检查RPC端口配置（默认16210）
- 验证网络连接

#### 2. 钱包创建失败
**症状**: `Wallet creation error`
**解决方案**:
- 检查文件权限
- 确保存储路径可写
- 验证密码强度

#### 3. 余额查询返回空
**症状**: `No balance information`
**解决方案**:
- 这是正常的（新钱包没有余额）
- 可以向测试地址发送测试币

#### 4. 交易估算失败
**症状**: `Transaction estimation failed`
**解决方案**:
- 检查目标地址格式
- 确保账户有足够余额
- 验证网络连接

### 调试技巧

#### 1. 启用详细日志
```bash
RUST_LOG=debug cargo run --bin wallet_test_cli run
```

#### 2. 增加超时时间
```bash
cargo run --bin wallet_test_cli run --timeout 60
```

#### 3. 测试特定网络
```bash
cargo run --bin wallet_test_cli run --network testnet-11
```

## 测试环境要求

### 必需条件
- ✅ Rust 1.70+
- ✅ 运行中的Tondi节点
- ✅ 网络连接

### 推荐配置
- 🔧 测试网络环境（testnet-10/testnet-11）
- 🔧 本地Tondi节点（更快的响应）
- 🔧 充足的磁盘空间（钱包存储）

### 网络配置

#### 测试网络端点
```
Testnet-10: 127.0.0.1:16210
Testnet-11: 127.0.0.1:16310
Mainnet: 127.0.0.1:16110
```

#### 远程节点配置
如果使用远程节点，需要配置gRPC客户端：
```rust
let config = TestConfig {
    // ... 其他配置
    rpc_url: "remote_node:16210".to_string(),
};
```

## 持续集成

### GitHub Actions示例
```yaml
name: Wallet Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run wallet tests
      run: |
        # 仅在有测试环境时运行
        if [ "$TONDI_TEST_ENABLED" = "true" ]; then
          cargo test --package tondi-dashboard-core --test wallet_endpoint_integration_tests
        fi
      env:
        TONDI_TEST_ENABLED: ${{ secrets.TONDI_TEST_ENABLED }}
```

## 最佳实践

### 1. 测试隔离
- 为每次测试使用唯一的钱包名称
- 定期清理测试钱包文件
- 使用测试网络避免真实资金损失

### 2. 错误处理
- 捕获和记录所有错误
- 提供有意义的错误消息
- 实现重试机制

### 3. 性能优化
- 并行运行独立测试
- 重用连接和钱包实例
- 设置合理的超时时间

### 4. 安全考虑
- 不在生产环境运行测试
- 使用强密码
- 不提交敏感信息到版本控制

## 扩展开发

### 添加新测试用例

1. 在`WalletEndpointIntegrationTests`中添加新方法：
```rust
async fn test_new_feature(config: &TestConfig) -> Result<String> {
    // 测试实现
    Ok("Test result message".to_string())
}
```

2. 将测试添加到测试套件：
```rust
let test_cases = vec![
    // ... 现有测试
    ("New Feature Test", Self::test_new_feature),
];
```

### 自定义测试配置

创建特定场景的配置：
```rust
impl TestConfig {
    pub fn for_mainnet() -> Self {
        Self {
            network: Network::Mainnet,
            test_amount_tondi: 0.001, // 较小金额
            timeout_seconds: 120, // 更长超时
            ..Default::default()
        }
    }
    
    pub fn for_stress_test() -> Self {
        Self {
            timeout_seconds: 300,
            // 压力测试配置
            ..Default::default()
        }
    }
}
```

## 总结

这个钱包端点测试系统提供了：

1. **全面覆盖** - 从钱包创建到交易的完整测试
2. **多种接口** - 图形界面、CLI和编程接口
3. **灵活配置** - 支持不同网络和参数
4. **详细报告** - 清晰的成功/失败信息
5. **易于扩展** - 模块化设计便于添加新测试

通过这个测试系统，你可以：
- ✅ 验证钱包功能的正确性
- ✅ 确保RPC端点的兼容性
- ✅ 进行回归测试
- ✅ 调试网络连接问题
- ✅ 验证新功能的集成

开始使用：选择适合你的测试方式，配置测试参数，然后运行测试来验证你的Tondi钱包端点功能！
