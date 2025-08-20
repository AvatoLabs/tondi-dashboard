#!/bin/bash

echo "测试常见的tondi节点端口..."
echo "节点地址: 8.210.45.192"

# 常见的tondi节点端口
ports=(
    16110  # 主网gRPC
    16111  # 主网wRPC
    16112  # 主网wRPC Borsh
    17110  # 测试网gRPC
    17111  # 测试网wRPC
    17112  # 测试网wRPC Borsh
    18110  # devnet gRPC
    18111  # devnet wRPC
    18112  # devnet wRPC Borsh
    8080   # 常见HTTP端口
    8443   # 常见HTTPS端口
    9000   # 常见gRPC端口
    9090   # 常见gRPC端口
)

echo -e "\n测试端口连通性..."
for port in "${ports[@]}"; do
    if nc -z -w3 8.210.45.192 $port 2>/dev/null; then
        echo "✅ 端口 $port 开放"
    else
        echo "❌ 端口 $port 关闭"
    fi
done

echo -e "\n测试完成！"
