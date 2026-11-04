#!/usr/bin/env bash
# 打包脚本共用的路径与版本工具。

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck disable=SC2034 # 由 source 本文件的打包脚本使用
BUNDLE_DIR="${ROOT_DIR}/target/bundle"
# shellcheck disable=SC2034
ICON_DIR="${ROOT_DIR}/resources/icons"

# 版本号以工作区 Cargo.toml 为准，避免包清单与 tag 漂移。
get_version() {
    if command -v jq >/dev/null 2>&1; then
        cargo metadata --no-deps --format-version 1 --manifest-path "${ROOT_DIR}/Cargo.toml" \
            | jq -r '.packages[] | select(.name == "biuviewer") | .version'
    else
        sed -n 's/^version = "\(.*\)"$/\1/p' "${ROOT_DIR}/Cargo.toml" | head -n 1
    fi
}

# 优先使用 CI 的交叉编译目录，本地构建时回退到默认的 target/release。
binary_path() {
    local triple="$1"
    local name="$2"
    if [ -n "${triple}" ] && [ -f "${ROOT_DIR}/target/${triple}/release/${name}" ]; then
        echo "${ROOT_DIR}/target/${triple}/release/${name}"
    else
        echo "${ROOT_DIR}/target/release/${name}"
    fi
}

make_zip() {
    local source_dir="$1"
    local output="$2"
    if command -v zip >/dev/null 2>&1; then
        (cd "${source_dir}" && zip -r -q "${output}" .)
    elif command -v 7z >/dev/null 2>&1; then
        (cd "${source_dir}" && 7z a -tzip "${output}" ./* >/dev/null)
    elif command -v powershell.exe >/dev/null 2>&1; then
        # Windows runner 上通常没有 zip，用 PowerShell 兜底。
        local win_source="${source_dir}"
        local win_output="${output}"
        if command -v cygpath >/dev/null 2>&1; then
            win_source="$(cygpath -w "${source_dir}")"
            win_output="$(cygpath -w "${output}")"
        fi
        powershell.exe -NoProfile -NonInteractive -Command \
            "Compress-Archive -Path '${win_source}\\*' -DestinationPath '${win_output}' -Force"
    else
        echo "需要 zip、7z 或 PowerShell 才能打包 ${output}" >&2
        return 1
    fi
}
