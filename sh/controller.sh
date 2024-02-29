#!/bin/bash

# echo "参数1: $1"
# echo "参数2: $2"
# echo "参数3: $3"
# echo "参数4: $4"
# echo "参数5: $5"

# 获取当前文件所在目录的绝对路径
#current_dir=$(dirname "$(realpath "$0")")

# 执行命令，并显示输出结果
eval "$(which expect) sh/script.ex $1 $2 $3 $4 $5"

