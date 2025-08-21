#!/bin/bash

echo "基于bp-tondi示例测试Tondi Devnet节点的正确API..."
echo "节点地址: 8.210.45.192:16610 (gRPC)"
echo "=================================="

# 测试gRPC连接
echo -e "\n1. 测试gRPC连接..."
echo "注意: gRPC使用HTTP/2协议，需要特殊工具测试"

# 检查是否有grpcurl
if command -v grpcurl >/dev/null 2>&1; then
    echo "使用grpcurl测试gRPC服务..."
    
    # 列出可用的服务
    echo "列出可用的gRPC服务:"
    if grpcurl -plaintext 8.210.45.192:16610 list 2>/dev/null; then
        echo "✅ gRPC服务列表获取成功"
    else
        echo "❌ gRPC服务列表获取失败"
    fi
    
    # 尝试调用getServerInfo方法
    echo -e "\n尝试调用getServerInfo方法:"
    if grpcurl -plaintext -d '{}' 8.210.45.192:16610 tondi.rpc.RpcApi/GetServerInfo 2>/dev/null; then
        echo "✅ getServerInfo调用成功"
    else
        echo "❌ getServerInfo调用失败"
    fi
    
    # 尝试调用getBlockDagInfo方法
    echo -e "\n尝试调用getBlockDagInfo方法:"
    if grpcurl -plaintext -d '{}' 8.210.45.192:16610 tondi.rpc.RpcApi/GetBlockDagInfo 2>/dev/null; then
        echo "✅ getBlockDagInfo调用成功"
    else
        echo "❌ getBlockDagInfo调用失败"
    fi
    
else
    echo "grpcurl未安装，无法测试gRPC服务"
    echo "安装grpcurl: go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest"
fi

# 测试wRPC连接 (如果可用)
echo -e "\n2. 测试wRPC连接..."
echo "注意: wRPC使用WebSocket协议"

# 检查端口是否开放
if timeout 3 bash -c "</dev/tcp/8.210.45.192/17610" 2>/dev/null; then
    echo "✅ wRPC端口 17610 可达"
    
    # 尝试WebSocket连接
    echo "尝试WebSocket连接..."
    if command -v websocat >/dev/null 2>&1; then
        echo "使用websocat测试WebSocket..."
        # 这里可以添加WebSocket测试
    else
        echo "websocat未安装，无法测试WebSocket"
        echo "安装websocat: cargo install websocat"
    fi
else
    echo "❌ wRPC端口 17610 不可达"
fi

# 测试基本的网络连接
echo -e "\n3. 测试网络连接稳定性..."
echo "连续测试TCP连接..."

for i in {1..5}; do
    if timeout 2 bash -c "</dev/tcp/8.210.45.192/16610" 2>/dev/null; then
        echo "✅ 第 $i 次gRPC连接测试成功"
    else
        echo "❌ 第 $i 次gRPC连接测试失败"
    fi
    sleep 1
done

echo -e "\n=================================="
echo "API测试完成！"
echo ""
echo "基于bp-tondi示例，正确的API调用应该是："
echo "1. gRPC端口 16610: 使用tondi.rpc.RpcApi服务"
echo "2. 主要方法: GetServerInfo, GetBlockDagInfo, GetBlocks等"
echo "3. 连接格式: grpc://8.210.45.192:16610"
echo ""
echo "如果gRPC调用失败，可能的原因："
echo "1. 节点服务未完全启动"
echo "2. 节点还在同步中"
echo "3. 网络配置问题"
echo "4. 需要等待节点完全同步"
