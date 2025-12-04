# Speaker Embedding 模型集成方案

## ✅ 模型下载状态

根据你的日志，模型已成功下载：

- **模型位置**：`D:\work\pure_python310\core\engine\models\speaker_embedding\cache`
- **模型格式**：PyTorch（SpeechBrain）
- **模型类型**：ECAPA-TDNN
- **输出维度**：`[batch, 1, 192]`（这是正常的，ECAPA-TDNN 输出 192 维）

## ⚠️ 重要发现

### 1. 输出维度说明

**日志显示**：`Output: embeddings of shape [batch, 1, 192]`

**说明**：
- ECAPA-TDNN 的实际输出是 **192 维**，不是 512 维
- 代码中使用的 512 维是占位符，需要修改为 192 维
- 这是正常的，ECAPA-TDNN 模型就是输出 192 维特征向量

### 2. ONNX 导出不支持

**原因**：
- SpeechBrain 的预处理模块包含数据依赖操作（dynamic slicing）
- 这些操作与 ONNX 不兼容
- 这是 SpeechBrain 的已知限制

**解决方案**：
- ✅ 使用 PyTorch 模型（已下载）
- ✅ 通过 Python HTTP 服务包装（推荐）
- ❌ 不能直接使用 ONNX Runtime

## 🔧 集成方案

### 方案 1：Python HTTP 服务（推荐）

参考项目中已有的 `PiperHttpTts` 模式，创建 Python HTTP 服务：

**步骤 1**：创建 Python HTTP 服务脚本

```python
# core/engine/scripts/speaker_embedding_service.py
from flask import Flask, request, jsonify
from speechbrain.inference.speaker import EncoderClassifier
import numpy as np
import base64

app = Flask(__name__)
classifier = None

def load_model():
    global classifier
    model_path = "core/engine/models/speaker_embedding/cache"
    classifier = EncoderClassifier.from_hparams(source=model_path)
    print("✅ Speaker Embedding model loaded")

@app.route('/extract', methods=['POST'])
def extract_embedding():
    data = request.json
    audio_data = np.array(data['audio'], dtype=np.float32)
    
    # 转换为 tensor [batch, samples]
    audio_tensor = torch.from_numpy(audio_data).unsqueeze(0)
    
    # 提取 embedding
    embeddings = classifier.encode_batch(audio_tensor)
    
    # 转换为列表 [192]
    embedding = embeddings.squeeze().cpu().numpy().tolist()
    
    return jsonify({
        'embedding': embedding,
        'dimension': len(embedding)
    })

if __name__ == '__main__':
    load_model()
    app.run(host='127.0.0.1', port=5003)
```

**步骤 2**：创建 Rust HTTP 客户端

参考 `PiperHttpTts`，创建 `SpeakerEmbeddingHttpClient`：

```rust
// core/engine/src/speaker_identifier/speaker_embedding_client.rs
pub struct SpeakerEmbeddingHttpClient {
    base_url: String,
    client: reqwest::Client,
}

impl SpeakerEmbeddingHttpClient {
    pub async fn extract_embedding(&self, audio: &[f32]) -> Result<Vec<f32>> {
        let response = self.client
            .post(&format!("{}/extract", self.base_url))
            .json(&json!({ "audio": audio }))
            .send()
            .await?;
        
        let result: EmbeddingResponse = response.json().await?;
        Ok(result.embedding)
    }
}
```

**步骤 3**：修改 `EmbeddingBasedSpeakerIdentifier`

```rust
// 使用 HTTP 客户端而不是直接加载 ONNX
async fn extract_embedding(&self, audio_segment: &[AudioFrame]) -> EngineResult<Vec<f32>> {
    // 合并音频帧
    let audio_data = merge_audio_frames(audio_segment);
    
    // 调用 HTTP 服务
    let embedding = self.http_client.extract_embedding(&audio_data).await?;
    
    Ok(embedding)  // 返回 192 维向量
}
```

### 方案 2：直接使用 PyTorch（不推荐）

需要集成 PyTorch C++ API，复杂度较高。

## 📝 需要修改的代码

### 1. 修改输出维度

**文件**：`core/engine/src/speaker_identifier/embedding_based.rs`

```rust
// 修改占位符维度从 512 改为 192
Ok(vec![0.0; 192])  // ECAPA-TDNN 输出 192 维
```

### 2. 更新配置

**文件**：`core/engine/src/speaker_identifier/mod.rs`

```rust
pub struct EmbeddingBasedSpeakerIdentifierConfig {
    pub similarity_threshold: f32,
    pub max_speakers: usize,
    pub embedding_dim: usize,  // 添加：192
    pub service_url: Option<String>,  // 添加：HTTP 服务 URL
}
```

## ✅ 模型可用性确认

**结论**：✅ **模型可以支持后续开发**

**理由**：
1. ✅ 模型已成功下载
2. ✅ 模型可以正常加载和使用
3. ✅ 输出维度明确（192 维）
4. ✅ 可以通过 HTTP 服务集成

**下一步**：
1. 创建 Python HTTP 服务脚本
2. 创建 Rust HTTP 客户端
3. 修改 `EmbeddingBasedSpeakerIdentifier` 使用 HTTP 客户端
4. 更新输出维度为 192

## 🚀 快速开始

```bash
# 1. 启动 Python HTTP 服务
python core/engine/scripts/speaker_embedding_service.py

# 2. 在 Rust 代码中配置服务 URL
let config = EmbeddingBasedSpeakerIdentifierConfig {
    similarity_threshold: 0.7,
    max_speakers: 5,
    embedding_dim: 192,
    service_url: Some("http://127.0.0.1:5003".to_string()),
};
```

