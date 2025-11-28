# 监控 GPU 使用情况

## Windows PowerShell

### 实时监控（每秒刷新）

```powershell
nvidia-smi -l 1
```

### 实时监控（每 0.5 秒刷新，更流畅）

```powershell
nvidia-smi -l 0.5
```

### 单次查看

```powershell
nvidia-smi
```

### 查看特定信息

```powershell
# 只查看 GPU 使用率和内存
nvidia-smi --query-gpu=index,name,utilization.gpu,utilization.memory,memory.used,memory.total --format=csv -l 1

# 查看进程信息
nvidia-smi pmon -c 1
```

## WSL2 (Linux)

### 实时监控

```bash
watch -n 1 nvidia-smi
```

### 或者使用 nvidia-smi 自带的循环模式

```bash
nvidia-smi -l 1
```

### 查看进程

```bash
nvidia-smi pmon -c 1
```

## 常用参数说明

- `-l <秒数>`: 循环模式，每 N 秒刷新一次
- `--query-gpu=...`: 查询特定字段
- `--format=csv`: CSV 格式输出
- `pmon`: 进程监控模式

## 示例输出说明

```
+-----------------------------------------------------------------------------+
| NVIDIA-SMI 566.26       Driver Version: 566.26       CUDA Version: 12.4   |
|-------------------------------+----------------------+----------------------+
| GPU  Name            TCC/WDDM | Bus-Id        Disp.A | Volatile Uncorr. ECC |
| Fan  Temp  Perf  Pwr:Usage/Cap|         Memory-Usage | GPU-Util  Compute M. |
|===============================+======================+======================|
|   0  NVIDIA GeForce ...  WDDM | 00000000:01:00.0 On |                  N/A |
|  0%   45C    P8    12W / 115W |    234MiB /  8192MiB |      5%      Default |
+-------------------------------+----------------------+----------------------+
```

关键指标：
- **GPU-Util**: GPU 使用率（0-100%）
- **Memory-Usage**: 显存使用情况
- **Temp**: GPU 温度
- **Pwr:Usage/Cap**: 功耗使用情况

## 测试 GPU 使用

1. 打开一个终端运行监控命令：
   ```powershell
   nvidia-smi -l 1
   ```

2. 在另一个终端发送 TTS 请求

3. 观察 GPU 使用率是否上升

如果 GPU 使用率上升，说明 GPU 正在工作。

