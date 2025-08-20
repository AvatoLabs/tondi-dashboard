# Tondi Dashboard Devnet 支持

## 概述

本文档描述了为 tondi-dashboard 添加的 devnet 支持功能。devnet 是一个开发网络，允许开发者在不影响主网的情况下测试和开发功能。

## 主要功能

### 1. 网络枚举支持

在 `core/src/network.rs` 中添加了 `Network::Devnet` 变体：

```rust
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,  // 新增
}
```

### 2. 网络配置

- **网络后缀**: devnet 使用后缀 11 (`--netsuffix=11`)
- **地址前缀**: `tondidev:` 用于 devnet 地址
- **Explorer URL**: `https://explorer-dev11.tondi.org`

### 3. 自定义 Devnet URL 支持

用户可以在设置中配置自定义的 devnet 节点 URL：

- 在设置界面的 "Tondi Network" 部分
- 当选择 devnet 网络时，显示自定义 URL 输入框
- 支持可选的 URL 配置，留空则使用默认配置

### 4. 设置存储

- 在 `NodeSettings` 中添加了 `devnet_custom_url: Option<String>` 字段
- 支持设置的保存和加载
- 在设置比较逻辑中包含 devnet 自定义 URL 的比较

### 5. 节点配置

在 tondid 节点启动时：

- 自动添加 `--testnet --netsuffix=11` 参数
- 如果配置了自定义 devnet URL，会通过 `--uacomment` 参数传递
- 支持通过命令行参数配置自定义节点

### 6. UI 支持

- **网络选择器**: 在欢迎页面和设置页面支持 devnet 选择
- **资源链接**: 在概览页面提供 devnet explorer 链接
- **地址生成**: 支持 devnet 地址的生成和验证
- **交易处理**: 支持 devnet 交易的显示和链接

### 7. 国际化支持

添加了 devnet 相关的中英文翻译：

- 英文: "Devnet", "Devnet network", "Custom Devnet URL (optional):"
- 中文: "开发网", "开发网络", "自定义开发网 URL (可选):"

## 技术实现

### 配置结构

```rust
pub struct NodeSettings {
    // ... 其他字段
    pub devnet_custom_url: Option<String>,
}

pub struct Config {
    // ... 其他字段
    pub devnet_custom_url: Option<String>,
}
```

### 网络转换

```rust
impl From<Network> for NetworkId {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkId::new(network.into()),
            Network::Testnet => NetworkId::with_suffix(network.into(), 10),
            Network::Devnet => NetworkId::with_suffix(network.into(), 11),
        }
    }
}
```

### 设置比较

```rust
impl NodeSettings {
    pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
        // ... 其他比较逻辑
        if self.devnet_custom_url != other.devnet_custom_url {
            Some(true)
        } else {
            None
        }
    }
}
```

## 使用方法

### 1. 选择 Devnet 网络

1. 打开设置页面
2. 在 "Tondi Network" 部分选择 "Devnet"
3. 可选择配置自定义 devnet 节点 URL

### 2. 配置自定义 Devnet URL

1. 选择 devnet 网络
2. 在 "Custom Devnet URL (optional):" 输入框中输入节点 URL
3. 点击 "Apply" 保存设置
4. 重启应用程序以应用新设置

### 3. 使用默认 Devnet 配置

1. 选择 devnet 网络
2. 保持自定义 URL 为空
3. 使用系统默认的 devnet 配置

## 注意事项

1. **网络切换**: 切换到 devnet 网络后，需要重启节点服务
2. **数据隔离**: devnet 使用独立的数据目录和配置
3. **地址格式**: devnet 地址使用 `tondidev:` 前缀
4. **Explorer**: devnet 交易和地址在专门的 explorer 上查看

## 未来改进

1. **节点发现**: 实现自动的 devnet 节点发现机制
2. **配置验证**: 添加 devnet URL 的格式验证
3. **网络状态**: 显示 devnet 网络的连接状态和同步进度
4. **多 devnet 支持**: 支持多个不同的 devnet 网络配置

## 总结

通过添加 devnet 支持，tondi-dashboard 现在可以：

- 连接到开发网络进行测试
- 配置自定义的 devnet 节点
- 在开发环境中安全地测试功能
- 支持开发者的本地开发需求

这为 tondi 生态系统的开发和测试提供了更好的支持。
