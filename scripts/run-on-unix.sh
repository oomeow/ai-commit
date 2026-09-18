#!/usr/bin/env bash
set -euo pipefail

# 检测是否为 Unix-like 系统
case "$(uname -s)" in
    Linux|Darwin|*BSD) ;;
    *) echo "Skipping: not a Unix system."; exit 0 ;;
esac

# 在此处执行你的实际命令
echo "Running on Unix..."
just completion
