# cfm

跨平台配置文件管理工具，通过 Git 仓库统一管理各类软件的配置文件。

## 功能特性

- **多平台支持** - 支持 Windows、Linux、macOS，可为不同平台配置不同路径
- **多种链接模式** - 支持软链接、硬链接、复制三种模式
- **Git 集成** - 通过 Git 仓库管理配置文件版本，支持同步

## 安装

```bash
git clone https://github.com/SeanSelene/cfm.git
cd cfm
cargo install --path .
```

## 使用

### 查看版本

```bash
cfm -v
# 或
cfm version
```

### 加载配置 (初始化)

```bash
cfm load <repo_url> [target_path]
```

克隆配置仓库并创建链接。`target_path` 默认为 `~/{仓库名}`。

### 应用配置

```bash
cfm apply [app...]
```

为指定软件创建链接或复制文件。不指定软件名称时，应用所有已配置的软件。若目标路径已存在，会提示确认是否覆盖。

### 列出配置

```bash
cfm list
```

列出所有已配置的软件及其状态。

### 编辑配置

```bash
cfm edit <app>
```

使用编辑器打开指定软件的配置目录。

### 清理

```bash
cfm clean
```

清理所有创建的链接、复制的文件、克隆目录和配置文件。使用 `-f` 或 `--force` 跳过确认提示。

### 取消应用

```bash
cfm unapply [-f] [app...]
```

删除指定软件已创建的链接或复制的文件。不指定软件名称时，取消应用所有软件。使用 `-f` 或 `--force` 跳过确认提示。

## 配置文件

在仓库根目录创建 `cfm.toml`：

```toml
[[apps]]
name = "nvim"
src_path = "nvim"
link_mode = "soft"
dest_path_unix = "~/.config/nvim"
dest_path_win = "$APPDATA/nvim"

[[apps]]
name = "starship"
src_path = "starship/starship.toml"
link_mode = "hard"
dest_path = "~/.config/starship.toml"

[[apps]]
name = "zed"
src_path = "zed"
link_mode = "cp"
dest_path_unix = "~/.config/zed"
dest_path_win = "$APPDATA/Zed"
```

### 配置项说明

| 字段               | 说明                                                 |
| ------------------ | ---------------------------------------------------- |
| `name`             | 软件名称                                             |
| `src_path`         | 配置文件在仓库中的相对路径                           |
| `link_mode`        | 链接模式：`soft`(软链接)、`hard`(硬链接)、`cp`(复制) |
| `dest_path`        | 通用配置路径                                         |
| `dest_path_unix`   | Unix 系统配置路径                                    |
| `dest_path_win`    | Windows 配置路径                                     |
| `dest_path_mac`    | macOS 配置路径                                       |

### 链接模式

| 模式   | 说明                                                |
| ------ | --------------------------------------------------- |
| `soft` | 软链接，Windows 目录使用 Junction（无需管理员权限） |
| `hard` | 硬链接，仅支持文件                                  |
| `cp`   | 复制文件，适合不支持链接的场景                      |

### 环境变量

支持在路径中使用环境变量：`$APPDATA`, `$HOME`, `$USERPROFILE` 等

## 许可证

MIT
