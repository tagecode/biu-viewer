#!/usr/bin/env bash
# 手动性能测试：生成素材、跑基准、采样空闲内存。
# 用法: ./scripts/bench-manual.sh [素材目录]，默认 /tmp/biu-bench
# 环境变量: SKIP_BENCH=1 跳过 cargo bench；BIN=/path/to/biuviewer 指定被测程序。

set -euo pipefail

# shellcheck source=scripts/utils.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/utils.sh"

ASSET_DIR="${1:-/tmp/biu-bench}"
BIN="${BIN:-${ROOT_DIR}/target/release/biuviewer}"

echo "== 1. 生成测试素材 → ${ASSET_DIR}"
cargo run --release --quiet -p biuviewer-decoder --example make_assets -- "${ASSET_DIR}"

if [ "${SKIP_BENCH:-0}" != "1" ]; then
    echo
    echo "== 2. 微基准（criterion）"
    cargo bench -p biuviewer-core --bench cache -- --quiet
    cargo bench -p biuviewer-decoder --bench decode -- --quiet
fi

echo
echo "== 3. 单图解码耗时"
for asset in large-10mb.png uhd-4k.png animation.gif; do
    cargo run --release --quiet -p biuviewer-decoder --example decode_file -- "${ASSET_DIR}/${asset}"
done

if [ ! -x "${BIN}" ]; then
    echo "找不到可执行文件 ${BIN}，先执行 cargo build --release" >&2
    exit 1
fi

echo
echo "== 4. 内存采样（打开 1000 张目录，空闲 15 秒）"
"${BIN}" "${ASSET_DIR}/1000-photos" >/dev/null 2>&1 &
APP_PID=$!
trap 'kill "${APP_PID}" 2>/dev/null || true' EXIT

sleep 3
FIRST=$(ps -o rss= -p "${APP_PID}" | tr -d ' ')
sleep 12
SECOND=$(ps -o rss= -p "${APP_PID}" | tr -d ' ')

printf '3 秒 RSS:  %s KB\n' "${FIRST:-进程已退出}"
printf '15 秒 RSS: %s KB\n' "${SECOND:-进程已退出}"
if [ -n "${FIRST}" ] && [ -n "${SECOND}" ]; then
    printf '增长:      %s KB\n' "$((SECOND - FIRST))"
    echo "（空闲增长应接近 0；翻页场景请按 docs/benchmark.md 的手动步骤测试）"
fi
