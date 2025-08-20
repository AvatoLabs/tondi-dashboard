#!/bin/bash

echo "开始测试tondi dev节点网络连接..."
echo "节点地址: 8.210.45.192"

# 测试基本网络连通性
echo -e "\n1. 测试基本网络连通性..."
if ping -c 3 8.210.45.192 > /dev/null 2>&1; then
    echo "✅ ping测试成功 - 节点网络可达"
else
    echo "❌ ping测试失败 - 节点网络不可达"
fi

# 测试端口连通性
echo -e "\n2. 测试端口连通性..."
echo "测试端口 17110 (gRPC)..."
if nc -z -w5 8.210.45.192 17110 2>/dev/null; then
    echo "✅ 端口 17110 开放 - gRPC服务可用"
else
    echo "❌ 端口 17110 关闭或不可达"
fi

echo "测试端口 17111 (wRPC)..."
if nc -z -w5 8.210.45.192 17111 2>/dev/null; then
    echo "✅ 端口 17111 开放 - wRPC服务可用"
else
    echo "❌ 端口 17111 关闭或不可达"
fi

echo "测试端口 17112 (wRPC Borsh)..."
if nc -z -w5 8.210.45.192 17112 2>/dev/null; then
    echo "✅ 端口 17112 开放 - wRPC Borsh服务可用"
else
    echo "❌ 端口 17112 关闭或不可达"
fi

# 测试HTTP连接
echo -e "\n3. 测试HTTP连接..."
echo "测试HTTP连接到端口 17110..."
if curl -s --connect-timeout 5 "http://8.210.45.192:17110" > /dev/null 2>&1; then
    echo "✅ HTTP连接成功"
else
    echo "⚠️  HTTP连接失败（这可能是正常的，如果节点只支持gRPC）"
fi

# 测试TLS连接
echo -e "\n4. 测试TLS连接..."
echo "测试TLS连接到端口 17110..."
if openssl s_client -connect 8.210.45.192:17110 -servername 8.210.45.192 < /dev/null > /dev/null 2>&1; then
    echo "✅ TLS连接成功"
else
    echo "⚠️  TLS连接失败（这可能是正常的，如果节点只支持gRPC）"
fi

echo -e "\n测试完成！"
echo "如果ping成功且端口开放，说明节点网络可达"
echo "如果HTTP/TLS失败，这通常是正常的，因为tondi节点主要使用gRPC协议"
