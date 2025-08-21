#!/bin/bash

echo "测试Tondi Devnet节点的wRPC连接..."
echo "节点地址: 8.210.45.192:17610 (wRPC)"
echo "=================================="

# 测试wRPC端口连通性
echo -e "\n1. 测试wRPC端口连通性..."
if timeout 3 bash -c "</dev/tcp/8.210.45.192/17610" 2>/dev/null; then
    echo "✅ wRPC端口 17610 可达"
else
    echo "❌ wRPC端口 17610 不可达"
    exit 1
fi

# 测试WebSocket连接
echo -e "\n2. 测试WebSocket连接..."
echo "尝试建立WebSocket连接..."

# 使用curl测试WebSocket升级
if curl -s --include --no-buffer \
    --header "Connection: Upgrade" \
    --header "Upgrade: websocket" \
    --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
    --header "Sec-WebSocket-Version: 13" \
    "http://8.210.45.192:17610" > /dev/null 2>&1; then
    echo "✅ WebSocket升级请求成功"
else
    echo "❌ WebSocket升级请求失败"
fi

# 测试HTTP连接
echo -e "\n3. 测试HTTP连接..."
if curl -s --connect-timeout 5 "http://8.210.45.192:17610" > /dev/null 2>&1; then
    echo "✅ HTTP连接成功"
else
    echo "⚠️  HTTP连接失败（这可能是正常的，如果节点只支持WebSocket）"
fi

# 测试RPC调用
echo -e "\n4. 测试RPC调用..."
echo "尝试调用getServerInfo..."

# 构造一个简单的RPC请求
rpc_request='{"jsonrpc":"2.0","method":"getServerInfo","params":[],"id":1}'

echo "发送RPC请求到端口 17610..."
response=$(curl -s --connect-timeout 5 \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$rpc_request" \
    "http://8.210.45.192:17610" 2>/dev/null)

if [ ! -z "$response" ]; then
    echo "✅ RPC调用成功"
    echo "   响应: ${response:0:200}..."
else
    echo "❌ RPC调用失败或无响应"
fi

echo -e "\n=================================="
echo "wRPC连接测试完成！"
echo ""
echo "建议："
echo "1. 如果wRPC测试成功，在dashboard中启用wRPC而不是gRPC"
echo "2. 如果wRPC也失败，可能是节点服务未完全启动"
echo "3. 当前gRPC客户端实现不完整，不建议使用"
