#!/bin/bash

echo "🔍 Tondi Dashboard Metrics问题诊断"
echo "=================================="
echo ""

echo "1. 检查dashboard状态..."
if pgrep -f "tondi-dashboard" > /dev/null; then
    echo "✅ Dashboard正在运行"
else
    echo "❌ Dashboard未运行"
fi

echo ""

echo "2. 检查网络连接..."
echo "   目标节点: 8.210.45.192:16610 (gRPC)"

if timeout 3 bash -c "</dev/tcp/8.210.45.192/16610" 2>/dev/null; then
    echo "✅ gRPC端口连接正常"
else
    echo "❌ gRPC端口连接失败"
    exit 1
fi

echo ""

echo "3. 检查配置文件..."
echo "请确认您的dashboard配置："
echo "   - Network: Devnet"
echo "   - Enable gRPC: ✓"
echo "   - gRPC Network Interface: Custom"
echo "   - Custom Address: 127.0.0.1:16610"
echo "   - Devnet Custom URL: 8.210.45.192:16610"

echo ""

echo "4. 可能的问题和解决方案..."
echo ""
echo "🔴 问题1: gRPC客户端未正确集成到MetricsService"
echo "   解决方案: 需要确保TondiService正确使用gRPC客户端"
echo ""
echo "🔴 问题2: Metrics数据格式不匹配"
echo "   解决方案: 需要将GetMetricsResponse转换为MetricsSnapshot"
echo ""
echo "🔴 问题3: Dashboard UI未正确显示metrics数据"
echo "   解决方案: 检查overview模块的数据绑定"

echo ""

echo "5. 调试步骤..."
echo "1. 在dashboard中查看日志输出"
echo "2. 检查是否有错误信息"
echo "3. 确认gRPC连接状态"
echo "4. 验证metrics数据是否正确获取"

echo ""

echo "6. 临时解决方案..."
echo "如果问题持续，可以尝试："
echo "1. 重启dashboard"
echo "2. 检查防火墙设置"
echo "3. 验证远程节点状态"
echo "4. 查看dashboard的详细日志"

echo ""

echo "🎯 诊断完成！请根据上述信息排查问题。"
echo "=================================="
