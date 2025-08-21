#!/bin/bash

echo "测试tondi devnet节点的正确端口..."
echo "节点地址: 8.210.45.192"

# 正确的devnet端口
ports=(
    16610  # gRPC接口
    16611  # P2P接口  
    17610  # wRPC接口
)

echo -e "\n1. 测试端口连通性..."
for port in "${ports[@]}"; do
    if nc -z -w5 8.210.45.192 $port 2>/dev/null; then
        echo "✅ 端口 $port 开放"
    else
        echo "❌ 端口 $port 关闭"
    fi
done

echo -e "\n2. 测试TCP连接..."
for port in "${ports[@]}"; do
    if timeout 3 bash -c "</dev/tcp/8.210.45.192/$port" 2>/dev/null; then
        echo "✅ TCP连接到端口 $port 成功"
    else
        echo "❌ TCP连接到端口 $port 失败"
    fi
done

echo -e "\n3. 测试HTTP连接..."
for port in "${ports[@]}"; do
    echo "测试端口 $port..."
    if curl -s --connect-timeout 5 "http://8.210.45.192:$port" > /dev/null 2>&1; then
        echo "✅ HTTP连接到端口 $port 成功"
    else
        echo "⚠️  HTTP连接到端口 $port 失败（这可能是正常的，如果节点只支持gRPC/wRPC）"
    fi
done

echo -e "\n测试完成！"
