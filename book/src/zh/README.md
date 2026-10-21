# Soroban Cookbook 中文文档

**使用 Stellar 和 Soroban 构建智能合约的综合指南**

## 关于

Soroban Cookbook 是开发者在 Stellar 网络上使用 Soroban 构建智能合约的指南。本文档为各个级别的开发者提供清晰、文档完善的示例和实用模式——从你的第一个 "Hello World" 合约到复杂的 DeFi 协议。

## 快速开始

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 添加 WebAssembly 目标
rustup target add wasm32-unknown-unknown

# 安装 Stellar CLI
cargo install --locked stellar-cli --version 22.1.0

# 克隆仓库
git clone https://github.com/Soroban-Cookbook/Soroban-Cookbook-.git
cd Soroban-Cookbook-

# 运行基础示例
cd examples/basics/01-hello-world
cargo test
```

## 仓库结构

```
Soroban-Cookbook/
├── examples/               # 智能合约示例
│   ├── basics/             # 初学者基础
│   ├── intermediate/       # 常见模式
│   ├── advanced/           # 复杂系统和协议
│   ├── defi/               # DeFi 示例
│   ├── nfts/               # NFT 实现
│   ├── governance/         # DAO 和投票系统
│   └── tokens/             # 代币标准
├── book/                   # mdBook 文档源码
├── tests/                  # 集成和安全测试
└── scripts/                # 构建和部署脚本
```

## 按难度分类的示例

### 基础
核心 Soroban 概念，逐一学习。
- 01-hello-world: 合约结构，`#[contract]` / `#[contractimpl]`，单元测试
- 02-storage-patterns: `persistent`，`instance`，`temporary` 存储，TTL
- 03-authentication: `require_auth()`，管理员角色，余额管理
- 03-custom-errors: `#[contracterror]`，错误码
- 04-events: `env.events().publish()`，主题设计
- 05-error-handling: 错误枚举，验证，传播
- 06-validation-patterns: 前置条件检查，溢出安全运算

### 中级
常见模式和实际用例。
- 跨合约模式：工厂、代理、注册表
- 访问控制：多签模式、RBAC、时间锁
- 代币交互和包装器

### 高级
适合有经验开发者的复杂系统。
- 01-multi-party-auth: 阈值签名、多方授权
- 02-timelock: 延时执行、队列/取消/执行

## 代码质量标准

本 Cookbook 中的每个示例：
- 使用最新的稳定 Soroban SDK 成功编译
- 包含全面的单元测试和集成测试
- 强制执行严格的安全边界和 Rust/Soroban 最佳实践
- 通过所有 CI/CD 流水线，包括格式化、Clippy 和测试套件

## 贡献

欢迎贡献。请参阅贡献指南和社区准则。也请阅读我们的行为准则。

## 更多资源

- [Soroban 文档](https://developers.stellar.org/docs/smart-contracts)
- [Stellar 开发者门户](https://developers.stellar.org)
- [Soroban Rust SDK](https://github.com/stellar/rs-soroban-sdk)
- [Stellar 社区 Discord](https://discord.gg/stellardev)
