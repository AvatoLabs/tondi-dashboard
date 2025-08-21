#!/bin/bash

echo "诊断Tondi Devnet节点连接问题..."
echo "节点地址: 8.210.45.192"
echo "=================================="

# 1. 检查基本网络连通性
echo -e "\n1. 基本网络连通性检查..."
if ping -c 3 8.210.45.192 > /dev/null 2>&1; then
    echo "✅ 网络连通性正常"
else
    echo "❌ 网络连通性异常"
    exit 1
fi

# 2. 检查端口状态
echo -e "\n2. 端口状态检查..."
ports=(16610 16611 17610)
for port in "${ports[@]}"; do
    if timeout 3 bash -c "</dev/tcp/8.210.45.192/$port" 2>/dev/null; then
        echo "✅ 端口 $port 可达"
    else
        echo "❌ 端口 $port 不可达"
    fi
done

# 3. 检查gRPC服务响应
echo -e "\n3. gRPC服务响应检查..."
echo "尝试连接到gRPC端口 16610..."

# 使用telnet测试端口响应
if command -v telnet >/dev/null 2>&1; then
    echo "使用telnet测试..."
    timeout 5 telnet 8.210.45.192 16610 < /dev/null 2>&1 | head -5
else
    echo "telnet不可用，使用nc测试..."
    if command -v nc >/dev/null 2>&1; then
        echo "使用nc测试..."
        timeout 5 nc -v 8.210.45.192 16610 2>&1 | head -5
    else
        echo "nc不可用，使用bash测试..."
        timeout 5 bash -c "</dev/tcp/8.210.45.192/16610" && echo "端口响应正常" || echo "端口无响应"
    fi
fi

# 4. 检查wRPC服务响应
echo -e "\n4. wRPC服务响应检查..."
echo "尝试连接到wRPC端口 17610..."

if command -v curl >/dev/null 2>&1; then
    echo "使用curl测试wRPC..."
    timeout 5 curl -v "http://8.210.45.192:17610" 2>&1 | head -10
else
    echo "curl不可用，使用bash测试..."
    timeout 5 bash -c "</dev/tcp/8.210.45.192/17610" && echo "端口响应正常" || echo "端口无响应"
fi

# 5. 检查节点是否在同步
echo -e "\n5. 节点同步状态检查..."
echo "检查是否有区块数据..."

# 尝试获取一些基本信息
if command -v curl >/dev/null 2>&1; then
    echo "尝试获取节点信息..."
    # 这里可以添加具体的API调用
    echo "注意: 需要具体的API端点来获取节点信息"
else
    echo "curl不可用，无法进行API测试"
fi

echo -e "\n=================================="
echo "诊断完成！"
echo ""
echo "可能的问题和解决方案："
echo "1. 如果端口可达但没有响应，可能是服务未启动或配置错误"
echo "2. 如果WebSocket连接失败，检查wRPC配置"
echo "3. 如果没有区块数据，节点可能还在同步中"
echo "4. 检查防火墙和网络配置"
