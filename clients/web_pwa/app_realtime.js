// Lingua Web PWA - 实时流式版本
// 用于验证 CoreEngine 的实时 ASR 和翻译功能

class LinguaRealtimeApp {
    constructor() {
        this.mediaRecorder = null;
        this.audioContext = null;
        this.websocket = null;
        this.isRecording = false;
        this.serviceUrl = 'http://127.0.0.1:9000';
        this.recordStartTime = null;
        this.logEntries = [];
        this.audioWorkletNode = null;
        this.processorNode = null;

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

            // 创建 ScriptProcessorNode 或 AudioWorkletNode 来处理音频
            // 使用 ScriptProcessorNode（兼容性更好）
            const bufferSize = 4096;
            this.processorNode = this.audioContext.createScriptProcessor(bufferSize, 1, 1);

            this.processorNode.onaudioprocess = (e) => {
                // 持续发送音频帧（连续模式）
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

                // 发送音频帧（每帧都会立即发送，后端会持续处理）
                const message = {
                    type: 'audio_frame',
                    data: base64Audio,
                    timestamp_ms: Date.now() - (this.recordStartTime || Date.now()),
                    sample_rate: 16000,
                    channels: 1
                };

                try {
                    this.websocket.send(JSON.stringify(message));
                } catch (error) {
                    console.error('Error sending audio frame:', error);
                }
            };

            source.connect(this.processorNode);
            this.processorNode.connect(this.audioContext.destination);

            this.isRecording = true;
            this.recordStartTime = Date.now();

            // 更新 UI
            this.updateStatus('recording', '正在转录...（持续模式：说话即转录翻译）');
            this.logMessage('开始连续转录模式：系统将持续接收语音并实时翻译输出');
            document.getElementById('btnStart').disabled = true;
            document.getElementById('btnStop').disabled = false;
            document.getElementById('serviceUrl').disabled = true;
            document.getElementById('srcLang').disabled = true;
            document.getElementById('tgtLang').disabled = true;

        } catch (error) {
            console.error('Error starting recording:', error);
            this.showError('无法访问麦克风。请检查权限设置。');
            this.logMessage(`麦克风访问失败：${error.message}`, 'error');
        }
    }

    async connectWebSocket() {
        return new Promise((resolve, reject) => {
            const wsUrl = this.serviceUrl.replace('http://', 'ws://').replace('https://', 'wss://') + '/stream';
            this.logMessage(`连接 WebSocket: ${wsUrl}`);

            this.websocket = new WebSocket(wsUrl);

            this.websocket.onopen = () => {
                this.logMessage('WebSocket 连接已建立');

                // 发送配置消息
                const config = {
                    type: 'config',
                    src_lang: document.getElementById('srcLang').value,
                    tgt_lang: document.getElementById('tgtLang').value
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
                this.logMessage('WebSocket 连接已关闭');
                if (this.isRecording) {
                    this.showError('WebSocket 连接已断开，请重新开始转录');
                    this.stopRecording();
                }
            };
        });
    }

    handleResponse(response) {
        // 更新转录文本（连续模式：每次检测到边界都会立即更新）
        if (response.transcript) {
            document.getElementById('transcript').textContent = response.transcript;
            this.logMessage(`📝 转录: ${response.transcript}`);
        }

        // 更新翻译文本（连续模式：每次检测到边界都会立即更新）
        if (response.translation) {
            document.getElementById('translation').textContent = response.translation;
            this.logMessage(`🌐 翻译: ${response.translation}`);
        }

        // 播放返回的音频（连续模式下，每句话完成后都会立即播放，无需等待停止）
        if (response.audio) {
            this.logMessage('🔊 收到音频，立即播放中...（连续模式：无需停止即可听到翻译）');
            this.playAudio(response.audio);
        }
        
        // 在连续模式下，每次收到结果都说明系统正常工作
        if (response.transcript || response.translation) {
            this.logMessage('✅ 连续模式正常：已自动处理并返回结果');
        }
    }

    stopRecording() {
        if (this.processorNode) {
            this.processorNode.disconnect();
            this.processorNode = null;
        }

        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }

        if (this.websocket) {
            this.websocket.close();
            this.websocket = null;
        }

        this.isRecording = false;

        // 更新 UI
        this.updateStatus('idle', '已停止转录');
        const durationMs = this.recordStartTime ? Date.now() - this.recordStartTime : 0;
        this.logMessage(`停止转录。转录时长 ${(durationMs / 1000).toFixed(2)} 秒`);
        this.resetUI();
    }

    async playAudio(base64Audio) {
        try {
            if (!base64Audio || base64Audio.length === 0) {
                return;
            }

            // 将 Base64 转换为 Blob
            const binaryString = atob(base64Audio);
            const bytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }

            const audioBlob = new Blob([bytes], { type: 'audio/wav' });
            const audioUrl = URL.createObjectURL(audioBlob);

            // 播放音频（不等待播放完成，允许下一段音频立即播放）
            const audio = new Audio(audioUrl);
            
            // 使用 Promise 处理播放，但不阻塞后续音频
            audio.play().catch(err => {
                console.warn('Audio play error (non-blocking):', err);
            });

            // 清理 URL
            audio.onended = () => {
                URL.revokeObjectURL(audioUrl);
            };

        } catch (error) {
            console.error('Error playing audio:', error);
            // 不显示错误，因为音频播放失败不影响主要功能
        }
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
        document.getElementById('tgtLang').disabled = false;
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
    new LinguaRealtimeApp();
});

