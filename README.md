## Aspen 命令行工具
本项目会使用 Rust 和 `clap` 4.4.0 创建一个命令行工具 `aspen`，先实现 ssh登录服务器和查看服务器列表 两个个功能。如果你也想实现一个自己的命令行工具，可以按照以下步骤进行：

### 第 1 步：编译和安装

1. **编译项目**：

    ```bash
    cargo build --release
    ```

2. **安装到系统**：

    - 在 Linux 或 macOS 上，你可以将编译后的可执行文件复制到 `/usr/local/bin` 或其他 PATH 包含的目录：

      ```bash
      sudo cp target/release/aspen /usr/local/bin/
      ```

    - 在 Windows 上，你可以将可执行文件复制到任何 PATH 包含的目录，或者手动添加其所在目录到系统 PATH。

### 第 4 步：使用工具

一旦安装，你就可以直接在命令行中使用 `aspen`，例如：

```bash
aspen ssh 
aspen ssh 1
aspen serv "https%3A%2F%2Fexample.com"
aspen timestamp 1609459200
```
正常教程到这里就结束了，但是通过复制的方法安装命令行，实在是low，必须要使用一种装逼的方式来安装。因此，下面的步骤才是命令行装逼的关键，支持cargo安装。

### 第5步，支持cargo安装

要使你的 `my_dev_tool` 命令行工具能够通过 `cargo install` 安装，你需要将其发布到 [crates.io](https://crates.io/)，这是 Rust 的包管理仓库。在发布之前，你需要创建一个帐户并获取一个 API 令牌用于身份验证。以下是将你的工具准备并发布到 crates.io 的步骤：

#### 第（1）步：注册 crates.io 帐户

1. 访问 [crates.io](https://crates.io/) 并注册一个帐户。
2. 登录后，在 "Account Settings" 中获取你的 API 令牌。
3. 验证自己的邮箱，邮箱只有验证成功才可以publish包。

#### 第（2）步：登录 Cargo

在你的终端中，使用以下命令登录 Cargo：

```bash
cargo login [your_api_token]
```

将 `[your_api_token]` 替换为你在 crates.io 上的 API 令牌。

#### 第（3）步：准备发布

确保你的 `Cargo.toml` 文件包含所有必要的信息，这对于发布至 crates.io 是必要的。下面是一个示例：

```toml
[package]
name = "my_dev_tool"
version = "0.1.0"
authors = ["Your Name <youremail@example.com>"]
edition = "2018"

# 以下是描述和文档链接等可选字段
description = "A useful development tool for various tasks"
documentation = "https://example.com/my_dev_tool/docs"
license = "MIT OR Apache-2.0"

[dependencies]
clap = "3.0"
chrono = "0.4"
urlencoding = "2.1"
```

确保更新 `authors`、`description`、`documentation`（如果适用），以及任何其他相关信息。

#### 第（4）步：发布到 crates.io

在你的项目目录中运行以下命令来发布你的包：

```bash
cargo publish
```

这将会把你的包上传到 crates.io。

### 第6步：通过 Cargo 安装

一旦你的包被成功发布到 crates.io，其他人就可以通过运行以下命令来安装你的工具：

```bash
cargo install my_dev_tool
```
#### 展示成果
```shell
% cargo install my_dev_tool
    Updating crates.io index
  Downloaded my_dev_tool v0.1.0
  Downloaded 1 crate (10.7 KB) in 2.64s
  Installing my_dev_tool v0.1.0
    Updating crates.io index
   Compiling autocfg v1.1.0
   Compiling utf8parse v0.2.1
   Compiling anstyle-query v1.0.0
   Compiling colorchoice v1.0.0
   Compiling anstyle v1.0.4
   Compiling strsim v0.10.0
   Compiling clap_lex v0.6.0
   Compiling iana-time-zone v0.1.58
   Compiling urlencoding v2.1.3
   Compiling anstyle-parse v0.2.2
   Compiling anstream v0.6.4
   Compiling num-traits v0.2.17
   Compiling clap_builder v4.4.8
   Compiling chrono v0.4.31
   Compiling clap v4.4.8
   Compiling my_dev_tool v0.1.0
    Finished release [optimized] target(s) in 7.84s
  Installing /home/maocg/.cargo/bin/my_dev_tool
   Installed package `my_dev_tool v0.1.0` (executable `my_dev_tool`)

% my_dev_tool timestamp 1609459200
2021-01-01T08:00:00+08:00
```

### 注意事项

- 在发布之前，请确保代码和文档是清晰和完整的，这对于其他人使用你的工具非常重要。
- 你需要更新版本号（在 `Cargo.toml` 中的 `version` 字段）在每次发布新的更改时。
- 如果你的工具包含敏感或专有信息，请在发布前仔细检查。

## 0.2版本更新内容
[支持命令行提示](./add_promot.md)