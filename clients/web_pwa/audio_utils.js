(function (global) {
    const MIN_DURATION_MS = 600;

    function padChannelData(channelData, minSamples) {
        if (!(channelData instanceof Float32Array)) {
            throw new TypeError('channelData must be a Float32Array');
        }
        if (channelData.length >= minSamples) {
            return channelData.slice(0);
        }
        const padded = new Float32Array(minSamples);
        padded.set(channelData);
        return padded;
    }

    function ensureMinDuration(audioContext, audioBuffer, minDurationMs = MIN_DURATION_MS) {
        if (!audioBuffer || typeof audioBuffer.sampleRate !== 'number') {
            return audioBuffer;
        }
        const minSamples = Math.ceil(audioBuffer.sampleRate * minDurationMs / 1000);
        if (audioBuffer.length >= minSamples) {
            return audioBuffer;
        }
        const context = audioContext || new (window.AudioContext || window.webkitAudioContext)({
            sampleRate: audioBuffer.sampleRate,
        });
        const paddedBuffer = context.createBuffer(
            audioBuffer.numberOfChannels,
            minSamples,
            audioBuffer.sampleRate
        );

        for (let channel = 0; channel < audioBuffer.numberOfChannels; channel++) {
            const sourceData = audioBuffer.getChannelData(channel);
            const paddedData = padChannelData(sourceData, minSamples);
            paddedBuffer.getChannelData(channel).set(paddedData);
        }

        return paddedBuffer;
    }

    const api = {
        MIN_DURATION_MS,
        padChannelData,
        ensureMinDuration,
    };

    if (global) {
        global.LinguaAudioUtils = api;
    }

    if (typeof module !== 'undefined' && module.exports) {
        module.exports = api;
    }
})(typeof window !== 'undefined' ? window : globalThis);

