// Lingua Web PWA - 极简网页应用
// 用于验证 CoreEngine 的 S2S 翻译功能

class LinguaApp {
    constructor() {
        this.mediaRecorder = null;
        this.audioChunks = [];
        this.isRecording = false;
        this.serviceUrl = 'http://127.0.0.1:9000';
        this.recordStartTime = null;
        this.logEntries = [];
        
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

            // 创建 MediaRecorder（使用 WAV 格式）
            const options = {
                mimeType: 'audio/webm;codecs=opus', // 使用 WebM，后续转换为 WAV
                audioBitsPerSecond: 128000
            };

            // 检查浏览器支持的格式
            if (!MediaRecorder.isTypeSupported(options.mimeType)) {
                // 如果不支持，使用默认格式
                this.mediaRecorder = new MediaRecorder(stream);
            } else {
                this.mediaRecorder = new MediaRecorder(stream, options);
            }

            this.audioChunks = [];
            this.isRecording = true;
            this.recordStartTime = performance.now();

            // 监听数据可用事件
            this.mediaRecorder.ondataavailable = (event) => {
                if (event.data.size > 0) {
                    this.audioChunks.push(event.data);
                    this.logMessage(`收到音频片段：${Math.round(event.data.size / 1024)} KB (共 ${this.audioChunks.length} 个)`);
                }
            };

            // 监听停止事件
            this.mediaRecorder.onstop = () => {
                this.processAudio();
            };

            // 开始录制
            this.mediaRecorder.start(1000); // 每 1 秒收集一次数据

            // 更新 UI
            this.updateStatus('recording', '正在录音...');
            this.logMessage('开始录音');
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

    stopRecording() {
        if (this.mediaRecorder && this.isRecording) {
            this.mediaRecorder.stop();
            this.isRecording = false;

            // 停止所有音频轨道
            this.mediaRecorder.stream.getTracks().forEach(track => track.stop());

            // 更新 UI
            this.updateStatus('processing', '正在处理...');
            const durationMs = this.recordStartTime ? performance.now() - this.recordStartTime : 0;
            this.logMessage(`停止录音。录音时长 ${(durationMs / 1000).toFixed(2)} 秒，收集片段 ${this.audioChunks.length} 个`);
            document.getElementById('btnStart').disabled = true;
            document.getElementById('btnStop').disabled = true;
        }
    }

    async processAudio() {
        try {
            // 将音频块合并为 Blob
            const audioBlob = new Blob(this.audioChunks, { type: this.audioChunks[0]?.type || 'audio/webm' });
            this.logMessage(`合并音频：${this.audioChunks.length} 个片段，总大小 ${(audioBlob.size / 1024).toFixed(1)} KB`);
            
            // 检查音频时长（至少 0.5 秒）
            const duration = await this.getAudioDuration(audioBlob);
            if (duration < 0.5) {
                throw new Error('录音时间太短，请至少录制 0.5 秒');
            }
            this.logMessage(`原始音频时长：${duration.toFixed(2)} 秒`);

            // 转换为 WAV 格式
            const wavBlob = await this.convertToWav(audioBlob);
            this.logMessage(`转换 WAV 成功：${(wavBlob.size / 1024).toFixed(1)} KB`);
            
            // 转换为 Base64
            const base64Audio = await this.blobToBase64(wavBlob);
            this.logMessage(`音频 Base64 长度：${base64Audio.length} 字符`);

            // 调用 S2S 接口
            await this.callS2SAPI(base64Audio);

        } catch (error) {
            console.error('Error processing audio:', error);
            this.showError('处理音频时出错: ' + error.message);
            this.logMessage(`处理音频失败：${error.message}`, 'error');
            this.resetUI();
        }
    }

    async getAudioDuration(blob) {
        return new Promise((resolve, reject) => {
            const audio = new Audio();
            audio.onloadedmetadata = () => {
                resolve(audio.duration);
            };
            audio.onerror = reject;
            audio.src = URL.createObjectURL(blob);
        });
    }

    async convertToWav(audioBlob) {
        try {
            // 使用 Web Audio API 转换为 WAV
            const arrayBuffer = await audioBlob.arrayBuffer();
            const audioContext = new (window.AudioContext || window.webkitAudioContext)({
                sampleRate: 16000
            });
            
            // 解码音频数据
            const audioBuffer = await audioContext.decodeAudioData(arrayBuffer);
            
            // 如果采样率不是 16kHz，需要重采样
            let targetBuffer = audioBuffer;
            if (audioBuffer.sampleRate !== 16000) {
                targetBuffer = await this.resampleAudio(audioContext, audioBuffer, 16000);
                this.logMessage(`已重采样到 16kHz（原始 ${audioBuffer.sampleRate}Hz）`);
            }
            
            // 确保音频长度足够（防止过短导致后端报错）
            if (window.LinguaAudioUtils && window.LinguaAudioUtils.ensureMinDuration) {
                targetBuffer = window.LinguaAudioUtils.ensureMinDuration(audioContext, targetBuffer);
                this.logMessage(`自动补齐音频，帧数 ${targetBuffer.length}, 采样率 ${targetBuffer.sampleRate}`);
            }

            // 转换为 WAV
            const wav = this.audioBufferToWav(targetBuffer);
            return new Blob([wav], { type: 'audio/wav' });
        } catch (error) {
            console.error('Error converting to WAV:', error);
            throw new Error('音频格式转换失败: ' + error.message);
        }
    }

    async resampleAudio(audioContext, audioBuffer, targetSampleRate) {
        // 创建离线音频上下文进行重采样
        const offlineContext = new OfflineAudioContext(
            audioBuffer.numberOfChannels,
            Math.round(audioBuffer.duration * targetSampleRate),
            targetSampleRate
        );
        
        const source = offlineContext.createBufferSource();
        source.buffer = audioBuffer;
        source.connect(offlineContext.destination);
        source.start();
        
        return await offlineContext.startRendering();
    }

    audioBufferToWav(buffer) {
        const numChannels = buffer.numberOfChannels;
        const sampleRate = buffer.sampleRate;
        const format = 1; // PCM
        const bitDepth = 16;

        const bytesPerSample = bitDepth / 8;
        const blockAlign = numChannels * bytesPerSample;

        const length = buffer.length * numChannels * bytesPerSample;
        const arrayBuffer = new ArrayBuffer(44 + length);
        const view = new DataView(arrayBuffer);

        // WAV 文件头
        const writeString = (offset, string) => {
            for (let i = 0; i < string.length; i++) {
                view.setUint8(offset + i, string.charCodeAt(i));
            }
        };

        writeString(0, 'RIFF');
        view.setUint32(4, 36 + length, true);
        writeString(8, 'WAVE');
        writeString(12, 'fmt ');
        view.setUint32(16, 16, true);
        view.setUint16(20, format, true);
        view.setUint16(22, numChannels, true);
        view.setUint32(24, sampleRate, true);
        view.setUint32(28, sampleRate * blockAlign, true);
        view.setUint16(32, blockAlign, true);
        view.setUint16(34, bitDepth, true);
        writeString(36, 'data');
        view.setUint32(40, length, true);

        // 写入 PCM 数据
        let offset = 44;
        for (let i = 0; i < buffer.length; i++) {
            for (let channel = 0; channel < numChannels; channel++) {
                const sample = Math.max(-1, Math.min(1, buffer.getChannelData(channel)[i]));
                view.setInt16(offset, sample < 0 ? sample * 0x8000 : sample * 0x7FFF, true);
                offset += 2;
            }
        }

        return arrayBuffer;
    }

    blobToBase64(blob) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onloadend = () => {
                // 移除 data URL 前缀（如 data:audio/wav;base64,）
                const base64 = reader.result.split(',')[1];
                resolve(base64);
            };
            reader.onerror = reject;
            reader.readAsDataURL(blob);
        });
    }

    async callS2SAPI(base64Audio) {
        const srcLang = document.getElementById('srcLang').value;
        const tgtLang = document.getElementById('tgtLang').value;
        const serviceUrl = document.getElementById('serviceUrl').value;

        const requestBody = {
            audio: base64Audio,
            src_lang: srcLang,
            tgt_lang: tgtLang
        };

        try {
            this.logMessage(`调用 S2S API：src=${srcLang}, tgt=${tgtLang}, audio length=${base64Audio.length}`);
            const response = await fetch(`${serviceUrl}/s2s`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(requestBody)
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`HTTP ${response.status}: ${errorText}`);
            }

            const result = await response.json();
            this.logMessage(`接口返回成功。Transcript 长度 ${(result.transcript || '').length}，Translation 长度 ${(result.translation || '').length}`);

            // 显示结果
            document.getElementById('transcript').textContent = result.transcript || '-';
            document.getElementById('translation').textContent = result.translation || '-';

            // 播放返回的音频
            if (result.audio) {
                await this.playAudio(result.audio);
            }

            // 更新状态
            this.updateStatus('idle', '处理完成');
            this.resetUI();

        } catch (error) {
            console.error('Error calling S2S API:', error);
            this.showError('调用翻译服务失败: ' + error.message);
            this.logMessage(`调用 S2S 接口失败：${error.message}`, 'error');
            this.resetUI();
        }
    }

    async playAudio(base64Audio) {
        try {
            if (!base64Audio || base64Audio.length === 0) {
                console.warn('No audio data to play');
                this.logMessage('服务器返回音频为空');
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

            // 播放音频
            const audio = new Audio(audioUrl);
            
            // 等待音频加载完成
            await new Promise((resolve, reject) => {
                audio.oncanplaythrough = () => {
                    resolve();
                };
                audio.onerror = (e) => {
                    reject(new Error('音频加载失败'));
                };
                audio.load();
            });

            await audio.play();
            this.logMessage('正在播放返回音频...');

            // 清理 URL
            audio.onended = () => {
                URL.revokeObjectURL(audioUrl);
                this.logMessage('音频播放结束');
            };

        } catch (error) {
            console.error('Error playing audio:', error);
            this.logMessage(`音频播放失败：${error.message}`, 'warn');
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
    new LinguaApp();
});

