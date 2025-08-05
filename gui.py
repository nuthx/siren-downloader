#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import tkinter as tk
from tkinter import ttk, messagebox
import json
import os
import threading
from datetime import datetime

from src.fetch_data import fetch_all_songs, fetch_album_data, fetch_song_data
from src.update_data import update_data
from src.download_music import download_music
from src.check_download import need_download


class MusicDownloaderGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("明日方舟塞壬唱片专辑下载器")
        self.root.geometry("900x600")
        
        # 数据存储
        self.song_list = {"count": 0, "instrumental": 0, "songs": []}
        self.is_downloading = False
        
        self.setup_ui()
        self.load_local_data()
        
    def setup_ui(self):
        # 主框架
        main_frame = ttk.Frame(self.root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # 配置网格权重
        self.root.columnconfigure(0, weight=1)
        self.root.rowconfigure(0, weight=1)
        main_frame.columnconfigure(1, weight=1)
        main_frame.rowconfigure(1, weight=1)
        
        # 按钮框架
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=0, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 10))
        
        # 三个功能按钮
        self.update_list_btn = ttk.Button(
            button_frame, 
            text="更新歌曲列表", 
            command=self.update_song_list
        )
        self.update_list_btn.pack(side=tk.LEFT, padx=(0, 10))
        
        self.update_urls_btn = ttk.Button(
            button_frame, 
            text="更新下载地址", 
            command=self.update_download_urls
        )
        self.update_urls_btn.pack(side=tk.LEFT, padx=(0, 10))
        
        self.download_btn = ttk.Button(
            button_frame, 
            text="下载音乐", 
            command=self.download_music
        )
        self.download_btn.pack(side=tk.LEFT)
        
        # 状态标签
        self.status_label = ttk.Label(main_frame, text="就绪")
        self.status_label.grid(row=0, column=1, sticky=tk.E)
        
        # 表格框架
        table_frame = ttk.Frame(main_frame)
        table_frame.grid(row=1, column=0, columnspan=2, sticky=(tk.W, tk.E, tk.N, tk.S))
        table_frame.columnconfigure(0, weight=1)
        table_frame.rowconfigure(0, weight=1)
        
        # 创建Treeview表格
        columns = ("title", "album", "publish", "instrumental", "status")
        self.tree = ttk.Treeview(table_frame, columns=columns, show="headings", height=20)
        
        # 定义列标题和宽度
        self.tree.heading("title", text="音乐名称")
        self.tree.heading("album", text="专辑")
        self.tree.heading("publish", text="发布日期")
        self.tree.heading("instrumental", text="伴奏")
        self.tree.heading("status", text="下载状态")
        
        self.tree.column("title", width=250)
        self.tree.column("album", width=180)
        self.tree.column("publish", width=100)
        self.tree.column("instrumental", width=60)
        self.tree.column("status", width=100)
        
        # 滚动条
        scrollbar = ttk.Scrollbar(table_frame, orient=tk.VERTICAL, command=self.tree.yview)
        self.tree.configure(yscrollcommand=scrollbar.set)
        
        # 布局表格和滚动条
        self.tree.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        scrollbar.grid(row=0, column=1, sticky=(tk.N, tk.S))
        
        # 进度条
        self.progress = ttk.Progressbar(main_frame, mode='determinate')
        self.progress.grid(row=2, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(10, 0))
        
    def load_local_data(self):
        """加载本地数据"""
        try:
            if os.path.exists("conf/data.json"):
                with open("conf/data.json", "r", encoding="utf-8") as f:
                    self.song_list = json.load(f)
                self.refresh_table()
                self.update_status(f"已加载 {self.song_list['count']} 首音乐")
            else:
                self.update_status("未找到本地数据文件")
        except Exception as e:
            messagebox.showerror("错误", f"加载本地数据失败: {str(e)}")
            
    def refresh_table(self):
        """刷新表格显示"""
        # 清空现有数据
        for item in self.tree.get_children():
            self.tree.delete(item)
            
        # 添加新数据
        for song in self.song_list.get("songs", []):
            status = self.get_download_status(song)
            instrumental_text = "是" if song.get("instrumental", 0) == 1 else "否"
            publish_date = song.get("publish", "")
            
            self.tree.insert("", tk.END, values=(
                song.get("title", ""),
                song.get("album", ""),
                publish_date,
                instrumental_text,
                status
            ))
            
    def get_download_status(self, song):
        """获取下载状态显示文本"""
        download = song.get("download", "")
        if download:
            return download.upper()
        else:
            return ""
            
    def need_download_check(self, song):
        """检查是否需要下载"""
        # 已下载
        download = song.get("download", "")
        if download and download != "":
            return False
        
        # 伴奏（这里可以根据配置决定是否跳过）
        # if song.get("instrumental", 0) == 1:
        #     return False
            
        return True
            
    def update_status(self, message):
        """更新状态显示"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        self.status_label.config(text=f"[{timestamp}] {message}")
        self.root.update_idletasks()
        
    def set_progress(self, value, maximum=100):
        """设置进度条值"""
        self.progress.config(maximum=maximum, value=value)
        self.root.update_idletasks()
        
    def reset_progress(self):
        """重置进度条"""
        self.progress.config(value=0)
        self.root.update_idletasks()
        
    def disable_buttons(self):
        """禁用所有按钮"""
        self.update_list_btn.config(state="disabled")
        self.update_urls_btn.config(state="disabled")
        self.download_btn.config(state="disabled")
        
    def enable_buttons(self):
        """启用所有按钮"""
        self.update_list_btn.config(state="normal")
        self.update_urls_btn.config(state="normal")
        self.download_btn.config(state="normal")
        
    def update_song_list(self):
        """更新歌曲列表"""
        def task():
            try:
                self.disable_buttons()
                self.reset_progress()
                self.update_status("正在获取歌曲列表...")
                
                # 获取在线歌曲列表
                song_list = fetch_all_songs()
                self.update_status(f"获取到 {song_list['count']} 首音乐，其中 {song_list['instrumental']} 首伴奏")
                
                # 计算专辑数量（去重后的）
                album_ids = list(set(song["album_id"] for song in song_list["songs"]))
                album_count = len(album_ids)
                total_operations = 1 + album_count  # 1个fetch_all_songs + N个专辑
                
                # 第一步完成（fetch_all_songs）
                self.set_progress(1, total_operations)
                
                self.update_status("正在获取专辑信息...")
                # 获取专辑数据，带进度回调
                def album_progress(current, total):
                    # 当前进度 = 1（已完成fetch_all_songs）+ current（当前专辑进度）
                    overall_progress = 1 + current
                    self.set_progress(overall_progress, total_operations)
                    self.update_status(f"正在获取专辑信息... ({current}/{total})")
                
                fetch_album_data(song_list, album_progress)
                
                # 保存到本地
                self.song_list = song_list
                with open("conf/data.json", "w", encoding="utf-8") as json_file:
                    json.dump(self.song_list, json_file, ensure_ascii=False, indent=4)
                
                self.refresh_table()
                self.set_progress(total_operations, total_operations)
                self.update_status("歌曲列表和专辑信息更新完成")
                
            except Exception as e:
                messagebox.showerror("错误", f"更新歌曲列表失败: {str(e)}")
                self.update_status("更新失败")
            finally:
                self.enable_buttons()
                
        threading.Thread(target=task, daemon=True).start()
        
    def update_download_urls(self):
        """更新下载地址"""
        def task():
            try:
                self.disable_buttons()
                self.reset_progress()
                
                # 检查是否有歌曲数据
                if not self.song_list.get("songs"):
                    self.update_status("请先更新歌曲列表")
                    messagebox.showwarning("警告", "请先点击'更新歌曲列表'获取歌曲数据")
                    return
                
                self.update_status("正在获取下载地址...")
                # 获取歌曲下载地址，带进度回调
                def song_progress(current, total):
                    self.set_progress(current, total)
                    self.update_status(f"正在获取下载地址... ({current}/{total})")
                
                fetch_song_data(self.song_list, song_progress)
                
                # 保存到本地
                with open("conf/data.json", "w", encoding="utf-8") as json_file:
                    json.dump(self.song_list, json_file, ensure_ascii=False, indent=4)
                
                self.refresh_table()
                self.set_progress(100, 100)
                self.update_status("下载地址更新完成")
                
            except Exception as e:
                messagebox.showerror("错误", f"更新下载地址失败: {str(e)}")
                self.update_status("更新失败")
            finally:
                self.enable_buttons()
                
        threading.Thread(target=task, daemon=True).start()      
  
    def download_music(self):
        """下载音乐"""
        if self.is_downloading:
            messagebox.showwarning("警告", "正在下载中，请等待完成")
            return
            
        # 检查是否有需要下载的音乐
        need_download_songs = [song for song in self.song_list.get("songs", []) if self.need_download_check(song)]
        if not need_download_songs:
            messagebox.showinfo("提示", "没有需要下载的音乐")
            return
            
        def task():
            try:
                self.is_downloading = True
                self.disable_buttons()
                self.reset_progress()
                
                downloaded_count = 0
                total_count = len(need_download_songs)
                
                for i, song in enumerate(need_download_songs):
                    current = i + 1
                    self.set_progress(current, total_count)
                    self.update_status(f"正在下载 ({current}/{total_count}): {song['title']}")
                    
                    try:
                        # 下载音乐
                        download_result = download_music(song)
                        song["download"] = download_result
                        downloaded_count += 1
                        
                        # 保存到本地文件
                        with open("conf/data.json", "w", encoding="utf-8") as json_file:
                            json.dump(self.song_list, json_file, ensure_ascii=False, indent=4)
                            
                        # 更新表格中对应行的状态
                        self.refresh_table()
                        
                    except Exception as e:
                        print(f"下载 {song['title']} 失败: {str(e)}")
                        continue
                
                self.set_progress(100, 100)
                self.update_status(f"下载完成，成功下载 {downloaded_count} 首音乐")
                if downloaded_count > 0:
                    messagebox.showinfo("完成", f"成功下载 {downloaded_count} 首音乐")
                    
            except Exception as e:
                messagebox.showerror("错误", f"下载过程中出现错误: {str(e)}")
                self.update_status("下载失败")
            finally:
                self.is_downloading = False
                self.enable_buttons()
                
        threading.Thread(target=task, daemon=True).start()


def main():
    root = tk.Tk()
    app = MusicDownloaderGUI(root)
    root.mainloop()


if __name__ == "__main__":
    main()