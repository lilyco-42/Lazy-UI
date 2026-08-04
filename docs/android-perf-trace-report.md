# Android 性能监控与 Trace 分析报告

> 目标：分析 `demo.apk`（包名 `rust.demo`）在设备 PJD110 (OPPO, Android 15) 上的
> CPU / 内存占用及渲染瓶颈。

## 1. 环境

| 项 | 值 |
|---|---|
| 设备 | PJD110 (OPPO), Android 15 (SDK 36) |
| 连接 | 无线调试 `192.168.1.6:36485` |
| 包名 | `rust.demo` |
| 采样时 PID | 30255 |
| Perfetto trace | `perfetto -t 8s -b 64mb sched freq idle gfx view` (31 MB) |

## 2. 内存占用（`dumpsys meminfo`）

| 项 | 值 |
|---|---|
| Total PSS | 782 MB |
| RSS | 507 MB |
| SwapPss | 398 MB |
| Native Heap | 164 MB（分配 545 MB，峰值 587 MB） |
| Graphics（GPU 内存） | 188 MB |
| 设备总内存 | 11 GB，应用占比约 4.4% |

结论：内存大头是 Native Heap（164 MB）与 Graphics（188 MB）。应用处于前台，
`altUiHidden = true`（后台）时 retention 50%。

## 3. CPU 占用（`top` 快照）

- 单核 85.7%，累计 CPU 时间 3:03.88。

## 4. 图形渲染（`dumpsys gfxinfo`）

- **Pipeline = Skia (OpenGL)**，确认走 GPU 渲染。
- **GPU 绘制时间中位数仅 2ms**，P99 4ms —— GPU 非常空闲。
- 帧耗时（主线程侧）中位数 **32ms**，远超 16ms vsync 目标。
- Janky frames: 12/42 (28.57%)，UI 线程慢帧 12 次。

## 5. Trace 分析（Perfetto）

### 5.1 各线程 CPU 占用（8s 窗口）

| 线程 | CPU 时间 | 说明 |
|---|---|---|
| Thread-3 | 2.62s | **渲染线程** |
| rust.demo（主线程） | 1.00s | 主线程 |
| binder:30255_3 | 0.34s | binder |
| AudioTrack | 0.11s | 音频 |
| RenderThread | 0.05s | 系统渲染线程 |

### 5.2 渲染线程瓶颈（Thread-3）

最耗时 slice 全部是：

```
dequeueBuffer - SurfaceView[rust.demo/rust.demo.MainActivity]#23(BLAST Consumer)23
每次 10 ~ 14ms
```

### 5.3 主线程瓶颈

主线程繁忙于 `deliverInputEvent`，每个 ~1ms，trace 期间持续出现（输入流密集）。

## 6. 结论：瓶颈在 BufferQueue，不在 GPU

关键证据链：

1. **GPU 绘制仅 2ms**（gfxinfo GPU 直方图），渲染能力充足。
2. **渲染线程几乎全部时间阻塞在 `dequeueBuffer`**（每帧 10-14ms），等待
   SurfaceView 的 BufferQueue 归还可用 buffer。
3. 帧耗时 32ms ≈ dequeueBuffer 阻塞（~13ms）+ 主线程输入处理 + 提交，而非 GPU。

### 根因

`rust.demo` 使用 **双缓冲 SurfaceView**。当应用在 SurfaceView 上持续绘制而
SurfaceFlinger/合成端 buffer 归还不及时，渲染线程只能空等，导致：

- 有效渲染率被锁在 ~30fps（33ms/帧），尽管 GPU 有大量余量。
- 28.57% jank 帧全部源于此。

## 7. 建议

1. **确认 buffer 数量**：cargo-quad-apk 的 SurfaceView 是否可配置为三缓冲
   (`setBufferCount(3)`)。增加一个 buffer 通常可直接消除 dequeueBuffer 等待。
2. **降低每帧 CPU 工作**：主线程 deliverInputEvent + 布局/构建耗时需 profile，
   目标 <8ms，留出 buffer 等待余量。
3. **核对合成端**：确认 SurfaceFlinger 是否因分辨率 1080x2124 或格式 RGB_8888
   放大导致合成变慢（可查 `dumpsys SurfaceFlinger --latency`）。
4. 内存侧：Native Heap 分配峰值 545 MB 偏高，建议检查泄漏（用
   `dumpsys meminfo --unreachable` 或 heapprofd 抓 native heap profile）。

## 8. 优化后验证（2026-08-04，APK v2）

### 改动

1. **MSAA 采样数 4 → 1**（`src/main.rs` `window_conf`）——减少 4x 像素填充带宽。
2. **空闲降帧**：无触摸/按键时主循环 sleep 到 ~15fps，交互后 5 帧恢复到 60fps。
3. **移除未引用字体** LXGWWenKaiMono-Medium.ttf（25MB），APK 从 ~51MB 降至 26.3MB。

### 对比（8s Perfetto trace，干净空闲态）

| 指标 | 优化前 | 优化后 | 变化 |
|---|---|---|---|
| 帧数（8s） | 721 | 280 | **-61%** |
| 平均帧间隔 | 11.0ms | 28.1ms | 空闲降帧生效 |
| 最大帧间隔 | 22.2ms | 78.7ms* | *66ms 为故意降帧，非 jank |
| Thread-3 总耗时 | 13.46s | 4.28s | **-68%** |
| dequeueBuffer 总耗时 | 4748ms | 1225ms | **-74%** |
| dequeueBuffer 平均 | 6.59ms | 4.36ms | **-34%** |

### 剩余观察

- 设备仍持续注入 ~90Hz 触摸流（`src=0x1002`，OPPO 曲面边缘防误触 daemon），
  空闲降帧只能在其间隙生效，无法完全降到 15fps。
- 空闲态 `deliverInputEvent` 已从基线 620 次降至 ~121 次，主线程从 2.27s 降至 0.41s。

## 9. 复现与再抓取

```bash
# 抓 trace（设备端）
adb shell "perfetto -o /data/local/tmp/demo_trace.pb -t 8s -b 64mb sched freq idle gfx view"
adb pull /data/local/tmp/demo_trace.pb .

# 分析
trace_processor_shell -Q "select t.tid, t.name, printf('%.2f', sum(s.dur)/1e9) from sched s join thread t on t.utid=s.utid where t.upid=(select upid from process where pid=<PID>) group by t.utid order by 3 desc" demo_trace.pb

# 查看渲染线程阻塞
trace_processor_shell -Q "select printf('%.3f', ts/1e9), printf('%.2f', dur/1e3), name from slice where track_id=72 and dur>0 order by dur desc limit 15" demo_trace.pb
```
