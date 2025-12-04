// Lingua Web PWA - ASR 专用版本（只测试识别停顿和文本识别）
// 不包含翻译和 TTS 功能

class LinguaAsrOnlyApp {
    constructor() {
        this.audioContext = null;
        this.websocket = null;
        this.isRecording = false;
        this.serviceUrl = 'http://127.0.0.1:9000';
        this.recordStartTime = null;
        this.logEntries = [];
        this.processorNode = null;
        this.boundaryCount = 0;
        this.transcriptCount = 0;

        this.init();
    }

    init() {
        // 绑定事件
        document.getElementById('btnStart').addEventListener('click', () => this.startRecording());
        document.getElementById('btnStop').addEventListener('click', () => this.stopRecording());
        document.getElementById('serviceUrl').addEventListener('change', (e) => {
            this.serviceUrl = e.target.value;
        });

        // 检查浏览器支持
        if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
            this.showError('您的浏览器不支持音频录制功能。请使用 Chrome、Firefox 或 Edge 浏览器。');
            document.getElementById('btnStart').disabled = true;
        }
    }

    async startRecording() {
        try {
            this.logMessage('请求麦克风权限...');

            // 请求麦克风权限
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    sampleRate: 16000,
                    channelCount: 1,
                    echoCancellation: true,
                    noiseSuppression: true
                }
            });

            // 创建 Web Audio API 上下文
            this.audioContext = new (window.AudioContext || window.webkitAudioContext)({
                sampleRate: 16000
            });

            // 连接到 WebSocket
            await this.connectWebSocket();

            // 创建音频源
            const source = this.audioContext.createMediaStreamSource(stream);

            // 创建 ScriptProcessorNode 来处理音频
            const bufferSize = 4096;
            this.processorNode = this.audioContext.createScriptProcessor(bufferSize, 1, 1);

            this.processorNode.onaudioprocess = (e) => {
                if (!this.isRecording || !this.websocket || this.websocket.readyState !== WebSocket.OPEN) {
                    return;
                }

                const inputData = e.inputBuffer.getChannelData(0);

                // 转换为 16-bit PCM
                const pcmData = new Int16Array(inputData.length);
                for (let i = 0; i < inputData.length; i++) {
                    const s = Math.max(-1, Math.min(1, inputData[i]));
                    pcmData[i] = s < 0 ? s * 0x8000 : s * 0x7FFF;
                }

                // 转换为 base64
                const base64Audio = btoa(String.fromCharCode(...new Uint8Array(pcmData.buffer)));

                // 发送音频帧
                const message = {
                    type: 'audio_frame',
                    data: base64Audio,
                    timestamp_ms: Date.now() - (this.recordStartTime || Date.now()),
                    sample_rate: 16000,
                    channels: 1
                };

                this.websocket.send(JSON.stringify(message));
            };

            source.connect(this.processorNode);
            this.processorNode.connect(this.audioContext.destination);

            this.isRecording = true;
            this.recordStartTime = Date.now();
            this.boundaryCount = 0;
            this.transcriptCount = 0;

            // 更新 UI
            this.updateStatus('recording', '正在录音...');
            this.logMessage('开始实时录音（ASR 专用模式）');
            document.getElementById('btnStart').disabled = true;
            document.getElementById('btnStop').disabled = false;
            document.getElementById('serviceUrl').disabled = true;
            document.getElementById('srcLang').disabled = true;

        } catch (error) {
            console.error('Error starting recording:', error);
            this.showError('无法访问麦克风。请检查权限设置。');
            this.logMessage(`麦克风访问失败：${error.message}`, 'error');
        }
    }

    async connectWebSocket() {
        return new Promise((resolve, reject) => {
            const wsUrl = this.serviceUrl.replace('http://', 'ws://').replace('https://', 'wss://') + '/asr/stream';
            this.logMessage(`连接 ASR WebSocket: ${wsUrl}`);

            this.websocket = new WebSocket(wsUrl);

            this.websocket.onopen = () => {
                this.logMessage('ASR WebSocket 连接已建立');

                // 发送配置消息
                const config = {
                    type: 'config',
                    src_lang: document.getElementById('srcLang').value
                };
                this.websocket.send(JSON.stringify(config));

                resolve();
            };

            this.websocket.onmessage = (event) => {
                try {
                    const response = JSON.parse(event.data);
                    this.handleResponse(response);
                } catch (error) {
                    console.error('Error parsing WebSocket message:', error);
                    this.logMessage(`解析消息失败：${error.message}`, 'error');
                }
            };

            this.websocket.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.logMessage('WebSocket 连接错误', 'error');
                reject(error);
            };

            this.websocket.onclose = () => {
                this.logMessage('ASR WebSocket 连接已关闭');
                if (this.isRecording) {
                    this.showError('WebSocket 连接已断开，请重新开始录音');
                    this.stopRecording();
                }
            };
        });
    }

    handleResponse(response) {
        // 更新统计信息
        if (response.is_boundary) {
            this.boundaryCount++;
            this.logMessage(`[边界 #${this.boundaryCount}] 检测到停顿`);
        }

        // 更新转录文本
        if (response.transcript) {
            this.transcriptCount++;
            document.getElementById('transcript').textContent = response.transcript;
            this.logMessage(`[识别 #${this.transcriptCount}] ${response.transcript}`);

            // 更新统计信息
            this.updateStats();
        }
    }

    updateStats() {
        const durationMs = this.recordStartTime ? Date.now() - this.recordStartTime : 0;
        const durationSec = (durationMs / 1000).toFixed(1);

        const statsText = `总时长: ${durationSec}s | 检测到停顿: ${this.boundaryCount} 次 | 识别文本: ${this.transcriptCount} 次`;
        document.getElementById('stats').textContent = statsText;
    }

    stopRecording() {
        // 先设置 isRecording = false，防止继续发送音频帧
        this.isRecording = false;
        
        if (this.processorNode) {
            this.processorNode.disconnect();
            this.processorNode = null;
        }

        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }

        if (this.websocket) {
            // 关闭 WebSocket 连接
            this.websocket.close();
            this.websocket = null;
        }

        // 更新 UI
        this.updateStatus('idle', '已停止录音');
        const durationMs = this.recordStartTime ? Date.now() - this.recordStartTime : 0;
        this.logMessage(`停止录音。录音时长 ${(durationMs / 1000).toFixed(2)} 秒`);
        this.updateStats();
        this.resetUI();
    }

    updateStatus(type, message) {
        const statusEl = document.getElementById('status');
        statusEl.className = `status ${type}`;
        statusEl.textContent = message;
    }

    showError(message) {
        const errorEl = document.getElementById('error');
        errorEl.textContent = message;
        errorEl.classList.add('show');

        // 3 秒后自动隐藏
        setTimeout(() => {
            errorEl.classList.remove('show');
        }, 5000);
    }

    resetUI() {
        document.getElementById('btnStart').disabled = false;
        document.getElementById('btnStop').disabled = true;
        document.getElementById('serviceUrl').disabled = false;
        document.getElementById('srcLang').disabled = false;
        this.recordStartTime = null;
    }

    logMessage(message, level = 'info') {
        const timestamp = new Date().toLocaleTimeString();
        const line = `[${timestamp}] ${message}`;
        const logMethod = level === 'error' ? console.error : level === 'warn' ? console.warn : console.log;
        logMethod(line);

        this.logEntries.push(line);
        if (this.logEntries.length > 200) {
            this.logEntries.shift();
        }

        const logEl = document.getElementById('log');
        if (logEl) {
            logEl.textContent = this.logEntries.join('\n');
            logEl.scrollTop = logEl.scrollHeight;
        }
    }
}

// 初始化应用
document.addEventListener('DOMContentLoaded', () => {
    new LinguaAsrOnlyApp();
});

