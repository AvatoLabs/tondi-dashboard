#!/bin/bash

echo "🔍 Tondi Dashboard深度诊断"
echo "=========================="
echo ""

echo "1. 检查dashboard进程状态..."
if pgrep -f "tondi-dashboard" > /dev/null; then
    echo "✅ Dashboard正在运行"
    echo "   进程ID: $(pgrep -f 'tondi-dashboard')"
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
fi

echo ""

echo "3. 检查配置文件..."
echo "请确认您的dashboard配置："
echo "   - Network: Devnet"
echo "   - Enable gRPC: ✓"
echo "   - Devnet Custom URL: 8.210.45.192:16610"

echo ""

echo "4. 检查dashboard日志..."
echo "请查看dashboard启动时的日志输出，寻找以下信息："
echo "   - '[gRPC]' 相关的连接信息"
echo "   - 'Connected to' 或 'Connection established'"
echo "   - 任何错误信息"
echo "   - 'Metrics' 相关的信息"

echo ""

echo "5. 可能的问题分析..."
echo "🔴 问题1: gRPC客户端未正确启动"
echo "   检查: 日志中是否有gRPC连接信息"
echo ""
echo "🔴 问题2: 配置未正确应用"
echo "   检查: 是否保存了配置并重启了dashboard"
echo ""
echo "🔴 问题3: metrics服务未正确启动"
echo "   检查: 日志中是否有metrics相关错误"
echo ""
echo "🔴 问题4: 数据转换失败"
echo "   检查: 是否有类型转换错误"

echo ""

echo "6. 调试步骤..."
echo "1. 完全关闭dashboard"
echo "2. 检查配置文件是否正确保存"
echo "3. 重新启动dashboard"
echo "4. 观察启动日志"
echo "5. 检查是否有错误信息"

echo ""

echo "7. 请提供以下信息..."
echo "请告诉我："
echo "1. dashboard启动时显示了什么日志？"
echo "2. 是否有任何错误信息？"
echo "3. 在Settings中看到的具体配置是什么？"
echo "4. 重启后是否看到任何变化？"

echo ""

echo "🎯 深度诊断完成！请根据上述信息排查问题。"
echo "=========================="
