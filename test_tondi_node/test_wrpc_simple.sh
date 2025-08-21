#!/bin/bash

echo "简化测试Tondi Devnet节点的wRPC连接..."
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

# 测试HTTP连接（快速测试）
echo -e "\n2. 测试HTTP连接..."
if timeout 3 curl -s "http://8.210.45.192:17610" > /dev/null 2>&1; then
    echo "✅ HTTP连接成功"
else
    echo "⚠️  HTTP连接失败（这可能是正常的，如果节点只支持WebSocket）"
fi

# 测试RPC调用（快速测试）
echo -e "\n3. 测试RPC调用..."
echo "尝试调用getServerInfo..."

rpc_request='{"jsonrpc":"2.0","method":"getServerInfo","params":[],"id":1}'

response=$(timeout 5 curl -s \
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
echo "wRPC简化测试完成！"
echo ""
echo "总结："
echo "1. 如果wRPC端口可达，说明服务正在运行"
echo "2. 如果RPC调用成功，说明wRPC服务可用"
echo "3. 建议在dashboard中启用wRPC而不是gRPC"
