#!/bin/bash

echo "测试tondi devnet节点的gRPC连接..."
echo "节点地址: 8.210.45.192:16610"

# 测试gRPC端口
echo -e "\n1. 测试gRPC端口连通性..."
if nc -z -w5 8.210.45.192 16610 2>/dev/null; then
    echo "✅ gRPC端口 16610 开放"
else
    echo "❌ gRPC端口 16610 关闭"
    exit 1
fi

# 测试TCP连接
echo -e "\n2. 测试TCP连接到gRPC端口..."
if timeout 3 bash -c "</dev/tcp/8.210.45.192/16610" 2>/dev/null; then
    echo "✅ TCP连接到gRPC端口成功"
else
    echo "❌ TCP连接到gRPC端口失败"
    exit 1
fi

# 尝试使用grpcurl测试gRPC服务（如果安装了）
echo -e "\n3. 测试gRPC服务..."
if command -v grpcurl >/dev/null 2>&1; then
    echo "尝试使用grpcurl测试gRPC服务..."
    if grpcurl -plaintext 8.210.45.192:16610 list 2>/dev/null; then
        echo "✅ gRPC服务响应成功"
        echo "可用的gRPC服务:"
        grpcurl -plaintext 8.210.45.192:16610 list
    else
        echo "⚠️  gRPC服务测试失败，但端口是开放的"
    fi
else
    echo "grpcurl未安装，跳过gRPC服务测试"
fi

echo -e "\n测试完成！"
echo "如果TCP连接成功，说明gRPC端口可达"
echo "要测试完整的gRPC功能，需要安装grpcurl或使用tondi dashboard"
