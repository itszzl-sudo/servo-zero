@echo off
REM 下载DejaVu Sans字体文件的PowerShell脚本
REM 这个字体用于文本渲染

echo 创建fonts目录...
if not exist fonts mkdir fonts

echo 正在下载 DejaVu Sans 字体...
powershell -Command "Invoke-WebRequest -Uri 'https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2' -OutFile 'dejavu-fonts.tar.bz2'"

echo 请使用7-Zip或其他工具解压dejavu-fonts.tar.bz2...
echo 然后将 DejaVuSans.ttf 复制到 fonts\ 目录

echo 或者手动下载：
echo 1. 访问 https://dejavu-fonts.github.io/Download.html
echo 2. 下载 dejavu-fonts-ttf-2.37.tar.bz2
echo 3. 解压并将 ttf/DejaVuSans.ttf 复制到项目的 fonts\ 目录
