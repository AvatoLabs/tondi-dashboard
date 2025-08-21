# Tondi Dashboard

一个现代化的、功能丰富的Tondi区块链钱包和账户管理仪表板。使用Rust和WebAssembly构建，提供最佳性能和安全性。

## 🚀 主要特性

- **多平台支持**: Web应用、Chrome扩展和原生桌面应用
- **钱包管理**: 创建、导入和管理Tondi钱包
- **账户管理**: 支持BIP32派生的多账户系统
- **交易工具**: 发送、接收和监控Tondi交易
- **实时更新**: 实时区块链数据和钱包同步
- **安全性**: 加密存储和安全密钥管理
- **用户友好**: 基于egui框架构建的现代UI
- **gRPC集成**: 与Tondi节点的gRPC通信支持

## 🛠️ 技术栈

- **后端**: Rust 1.89+ with async/await
- **前端**: WebAssembly (WASM) via egui 0.31.1
- **区块链**: Tondi链集成 (v0.17.0)
- **存储**: 本地加密存储
- **网络**: gRPC客户端用于节点通信
- **UI框架**: egui with eframe
- **构建工具**: Trunk for WASM构建

## 📦 安装

### 前置要求

- Rust 1.89+ 和 Cargo
- Node.js 18+ (用于Web构建)
- 本地或远程运行的Tondi节点

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/AvatoLabs/tondi-dashboard.git
cd tondi-dashboard

# 构建项目
cargo build --release

# 构建Web应用
cd app
trunk build

# 构建Chrome扩展
cd ../extensions/chrome
cargo build --target wasm32-unknown-unknown --release
```

### 运行应用

```bash
# 原生桌面应用
cargo run --bin tondi-dashboard

# Web应用
cd app
trunk serve

# Chrome扩展
# 从 extensions/chrome/dist/ 目录加载扩展
```

## 🔧 配置

仪表板通过gRPC连接到Tondi节点。在设置中配置您的节点连接：

- **网络**: 主网或测试网
- **节点URL**: 您的Tondi节点RPC端点
- **连接类型**: 直接或代理连接

## 📱 使用方法

### 创建钱包

1. 启动应用
2. 点击"创建新钱包"
3. 设置钱包名称和安全选项
4. 生成或导入助记词短语
5. 创建初始账户

### 管理账户

- **创建账户**: 生成新的BIP32账户
- **导入账户**: 通过助记词导入现有账户
- **账户切换**: 在账户间无缝切换
- **地址管理**: 根据需要生成新地址

### 发送交易

1. 选择源账户
2. 输入接收方地址
3. 指定金额和优先级费用
4. 审查交易详情
5. 确认并广播

## 🏗️ 项目架构

```
tondi-dashboard/
├── app/                    # Web应用
├── core/                   # 核心框架
│   ├── modules/           # 功能模块
│   ├── runtime/           # 运行时服务
│   └── utils/             # 工具函数
├── extensions/            # 浏览器扩展
│   └── chrome/           # Chrome扩展
└── macros/               # 过程宏
```

## 🔒 安全特性

- **加密存储**: 所有敏感数据在静态时加密
- **安全密钥派生**: BIP32/BIP39兼容的密钥生成
- **内存保护**: 安全的内存处理和零化
- **网络安全**: 与节点的加密通信

## 🌐 网络支持

- **主网**: 生产Tondi网络
- **测试网**: 开发和测试网络
- **自定义网络**: 支持自定义节点配置

## 📊 监控和分析

- **实时余额**: 实时账户余额更新
- **交易历史**: 完整的交易记录
- **网络状态**: 节点连接和同步状态
- **性能指标**: 交易处理统计

## 🧪 测试

```bash
# 运行测试
cargo test

# 运行特定模块测试
cargo test --package tondi-dashboard-core

# 运行集成测试
cargo test --workspace
```

## 🤝 贡献

我们欢迎贡献！请查看我们的贡献指南了解详情。

### 开发设置

```bash
# 安装开发依赖
cargo install trunk
cargo install wasm-bindgen-cli

# 运行测试
cargo test

# 代码检查
cargo check
```

## 📄 许可证

本项目采用ISC许可证。详见 [LICENSE](LICENSE) 文件。

## 🔗 相关链接

- [Tondi项目](https://github.com/AvatoLabs/Tondi)
- [egui框架](https://github.com/emilk/egui)
- [Rust编程语言](https://www.rust-lang.org/)

