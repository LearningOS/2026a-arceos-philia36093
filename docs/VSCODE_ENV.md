# VS Code 开发环境配置

本仓库的六道练习都在 RISC-V 上运行，评测环境是课程 Docker 镜像
`1301182193/arceos-env`（内置 Rust `nightly-2024-09-04`、RISC-V musl GCC
`11.2.1`、QEMU `8.2.2`）。因此**所有编译和测试都在容器里执行**，本机的 Windows
只负责编辑源码和启动 VS Code。

仓库里已经配好以下内容，直接使用即可：

| 文件 | 作用 |
| --- | --- |
| `.devcontainer/devcontainer.json` | 用课程镜像打开整个仓库，自动装好 rust-analyzer |
| `.vscode/settings.json` | rust-analyzer 指向 `arceos/Cargo.toml`，目标 `riscv64gc-unknown-none-elf` |
| `.vscode/extensions.json` | 推荐扩展：Dev Containers + rust-analyzer |
| `.vscode/tasks.json` | 六道题的单题测试任务和 `total-test.sh` 一键测试任务 |
| `.gitattributes` | 强制检出为 LF，避免 Windows 的 CRLF 让容器里的 shell 脚本 / Makefile 失效 |

## 一次性准备

1. 启动 Docker Desktop，等右下角显示引擎已启动。
2. 在 PowerShell 中确认镜像存在：

   ```powershell
   docker images | Select-String arceos
   ```

   如果没有 `1301182193/arceos-env:latest`，可以导入课程镜像或从 Docker Hub 拉取。
   已有本地镜像时，补一个与本仓库配置一致的标签：

   ```powershell
   docker tag 1301182193/arceos-env:latest docker.io/1301182193/arceos-env:latest
   ```

3. VS Code 安装扩展：**Dev Containers**（`ms-vscode-remote.remote-containers`）和
   **rust-analyzer**（`rust-lang.rust-analyzer`）。

## 用 VS Code 做题

1. 用 VS Code 打开仓库根目录（`2026a-arceos-<你的 GitHub 登录名>`）。
2. `Ctrl+Shift+P` → **Dev Containers: Reopen in Container**，等待容器启动。
   首次启动会自动安装 rust-analyzer 并执行 `postCreateCommand` 打印工具版本。
3. 在容器里打开终端，或用 **Terminal → Run Task**：
   - `ArceOS: test print_with_color`
   - `ArceOS: test ramfs_rename`
   - `ArceOS: test alt_alloc`
   - `ArceOS: test support_hashmap`
   - `ArceOS: test sys_map`
   - `ArceOS: test simple_hv`
   - `ArceOS: test all (total-test.sh)`（默认测试任务，`Ctrl+Shift+B`）
4. 修改源码后重新运行对应任务，末行出现 `... pass` 即为通过。

## 只用终端时

在容器外的仓库根目录运行：

```sh
./scripts/docker.sh
```

进入容器后（提示符 `root@arceos:/workspace#`）直接执行：

```sh
./scripts/test-print.sh      # 第 1 题
./scripts/test-ramfs_rename.sh
./scripts/test-alt_alloc.sh
./scripts/test-support_hashmap.sh
./scripts/test-sys_map.sh
./scripts/test-simple_hv.sh
./scripts/total-test.sh      # 六题总分，末行是分数
```

也可以不用交互 shell，一条命令跑完某道题：

```sh
./scripts/docker.sh ./scripts/test-sys_map.sh
```

## 常见问题

- **`failed to connect to the docker API`**：Docker Desktop 没启动，启动后重试。
- **拉取镜像超时**：本机已有镜像时不需要联网；用上面的 `docker tag` 命令补标签，
  `docker.sh` 和任务都会直接使用本地镜像。
- **容器里提示 `bash: ./scripts/xxx.sh: cannot execute: required file not found`**：
  脚本被检出成了 CRLF。仓库根目录已有 `.gitattributes` 强制 LF，重新检出即可：

  ```powershell
  git config core.autocrlf false
  git rm --cached -r . ; git reset --hard
  ```

  若只想修当前工作区，可在容器里执行：
  `find /workspace -name '*.sh' -exec sed -i 's/\r$//' {} +`。
- **rust-analyzer 在容器外报找不到 `riscv64gc-unknown-none-elf`**：容器外不需要
  rust-analyzer 分析内核，按上面的步骤在 Dev Container 里打开即可；容器里已装好
  `rust-src`、`llvm-tools` 和 RISC-V 目标。
- **测试输出里有 `qemu.log`、`*.img`、`*.bin`**：这些是生成物，已被 `.gitignore`
  忽略，不要提交。
