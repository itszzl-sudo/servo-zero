#!/bin/bash
# 下载DejaVu Sans字体文件
# 这个字体用于文本渲染

mkdir -p fonts

echo "正在下载 DejaVu Sans 字体..."
curl -L "https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2" -o /tmp/dejavu-fonts.tar.bz2

echo "解压字体文件..."
cd /tmp
tar -xjf dejavu-fonts.tar.bz2

echo "复制字体文件到项目..."
cp /tmp/dejavu-fonts-ttf-2.37/ttf/DejaVuSans.ttf "$PWD/fonts/"

echo "清理临时文件..."
rm -rf /tmp/dejavu-fonts*

echo "✓ 字体文件已下载到 fonts/DejaVuSans.ttf"
