const test = require('node:test');
const assert = require('node:assert/strict');

const { padChannelData, MIN_DURATION_MS } = require('../audio_utils.js');

test('padChannelData extends shorter buffers to required length', () => {
    const input = new Float32Array([0.2, -0.3]);
    const padded = padChannelData(input, 5);

    assert.equal(padded.length, 5);
    assert.ok(Math.abs(padded[0] - 0.2) < 1e-6);
    assert.ok(Math.abs(padded[1] + 0.3) < 1e-6);
    assert.equal(padded[2], 0);
    assert.equal(padded[3], 0);
    assert.equal(padded[4], 0);
});

test('padChannelData returns copy when buffer already long enough', () => {
    const input = new Float32Array([1, 2, 3, 4]);
    const padded = padChannelData(input, 3);

    assert.equal(padded.length, 4);
    assert.notStrictEqual(padded, input, 'should return a copy');
    assert.deepEqual(Array.from(padded), [1, 2, 3, 4]);
});

test('MIN_DURATION_MS is at least 600 ms to cover Whisper min length', () => {
    assert.ok(MIN_DURATION_MS >= 600);
});

