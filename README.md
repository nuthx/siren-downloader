# 明日方舟塞壬唱片专辑下载器

从塞壬唱片官网下载原汁原味的音乐专辑，并借助网易云音乐 API 补全缺失的 ID3 信息

## 安装

需要 Python 环境

```
git clone https://github.com/nuthx/siren-downloader.git
cd siren-downloader
pip install requirements.txt
```

## 设置

打开 `conf` 文件下的 `config.ini` 文件

修改 `download_path` 为你的音乐保存路径

默认跳过伴奏曲目下载。若需要下载伴奏，修改 `skip_instrumental` 为 `false`

其他选项保持默认即可

## 运行

在终端中执行

```
python main.py
```

等待专辑下载完成

## 致谢

[塞壬唱片-MSR](https://monster-siren.hypergryph.com/)

[网易云音乐API](https://gitlab.com/Binaryify/neteasecloudmusicapi)

## 免责

本项目代码仅供学习交流，不得用于商业用途，请在下载后 24 小时内删除，若侵权请联系
