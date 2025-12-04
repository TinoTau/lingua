//! 基于 Channel 的事件总线实现
//! 
//! 使用 tokio::sync::mpsc::channel 实现真正的事件发布/订阅机制

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use async_trait::async_trait;

use crate::error::{EngineError, EngineResult};
use super::{CoreEvent, EventTopic, EventSubscription};

/// 基于 Channel 的事件总线
pub struct ChannelEventBus {
    /// 事件发布通道（发送端）
    sender: mpsc::UnboundedSender<CoreEvent>,
    /// 订阅者注册表（topic -> Vec<接收端>）
    subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::UnboundedSender<CoreEvent>>>>>,
    /// 是否已启动
    started: Arc<RwLock<bool>>,
}

impl ChannelEventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<CoreEvent>();
        let subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::UnboundedSender<CoreEvent>>>>> = 
            Arc::new(RwLock::new(HashMap::new()));
        let started = Arc::new(RwLock::new(false));
        
        // 克隆用于后台任务
        let subscribers_clone = Arc::clone(&subscribers);
        
        // 启动后台任务：分发事件到所有订阅者
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let topic = event.topic.0.clone();
                let subscribers = subscribers_clone.read().await;
                
                if let Some(subs) = subscribers.get(&topic) {
                    // 发送到所有订阅该 topic 的订阅者
                    let mut failed_subs = Vec::new();
                    for (idx, sub) in subs.iter().enumerate() {
                        if sub.send(event.clone()).is_err() {
                            // 订阅者已断开，标记为失败
                            failed_subs.push(idx);
                        }
                    }
                    
                    // 清理失败的订阅者（在释放锁后）
                    if !failed_subs.is_empty() {
                        drop(subscribers);
                        let mut subs = subscribers_clone.write().await;
                        if let Some(subs_list) = subs.get_mut(&topic) {
                            // 从后往前删除，避免索引错乱
                            for &idx in failed_subs.iter().rev() {
                                if idx < subs_list.len() {
                                    subs_list.remove(idx);
                                }
                            }
                            // 如果没有订阅者了，删除该 topic
                            if subs_list.is_empty() {
                                subs.remove(&topic);
                            }
                        }
                    }
                }
            }
        });
        
        Self {
            sender,
            subscribers,
            started,
        }
    }
    
    /// 订阅指定 topic 的事件，返回接收端
    pub fn subscribe_receiver(&self, topic: EventTopic) -> mpsc::UnboundedReceiver<CoreEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        let topic_str = topic.0.clone();
        
        let subscribers = self.subscribers.clone();
        tokio::spawn(async move {
            let mut subs = subscribers.write().await;
            subs.entry(topic_str).or_insert_with(Vec::new).push(tx);
        });
        
        rx
    }
}

impl Default for ChannelEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::EventBus for ChannelEventBus {
    async fn start(&self) -> EngineResult<()> {
        let mut started = self.started.write().await;
        *started = true;
        Ok(())
    }

    async fn stop(&self) -> EngineResult<()> {
        let mut started = self.started.write().await;
        *started = false;
        Ok(())
    }

    async fn publish(&self, event: CoreEvent) -> EngineResult<()> {
        self.sender.send(event)
            .map_err(|e| EngineError::new(format!("Failed to publish event: {}", e)))?;
        Ok(())
    }

    async fn subscribe(&self, topic: EventTopic) -> EngineResult<EventSubscription> {
        // 为了兼容 trait，返回一个订阅对象
        // 实际使用应该调用 subscribe_receiver 方法
        Ok(EventSubscription { topic })
    }
}

