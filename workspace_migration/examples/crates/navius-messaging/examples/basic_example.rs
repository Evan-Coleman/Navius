use std::sync::Arc;
use std::time::Duration;

use navius_messaging::broker::TopologyBuilder;
use navius_messaging::config::BrokerConfig;
use navius_messaging::consumer::ConsumerOptions;
use navius_messaging::message::{Message, MessageAcknowledgment, MessageProcessingResult};
use navius_messaging::publisher::PublishOptions;
use navius_messaging::serialization::JsonSerializer;
use navius_messaging::util::{generate_correlation_id, generate_id};

// Our example message payload
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct UserEvent {
    id: String,
    user_id: String,
    event_type: String,
    timestamp: u64,
    data: serde_json::Value,
}

// Implement an in-memory broker for the example
mod memory_broker {
    use std::collections::{HashMap, VecDeque};
    use std::sync::Arc;
    use std::time::{Duration, Instant, SystemTime};

    use async_trait::async_trait;
    use dashmap::DashMap;
    use futures::Stream;
    use futures::stream::BoxStream;
    use tokio::sync::{Mutex, RwLock, mpsc};
    use tokio::time::sleep;
    use uuid::Uuid;

    use navius_messaging::broker::{
        BrokerMetrics, ConsumerControl, ConsumerHandle, MessageBroker, MessageBrokerFactory,
    };
    use navius_messaging::config::BrokerConfig;
    use navius_messaging::consumer::ConsumerOptions;
    use navius_messaging::error::{
        ConnectionStatus, DeliveryMode, MessagingError, MessagingResult,
    };
    use navius_messaging::message::{Message, MessageFilter, MessageHandler, ReceivedMessage};
    use navius_messaging::publisher::PublishOptions;
    use navius_messaging::serialization::MessageSerializer;
    use navius_messaging::topology::{Binding, BindingDestination, Exchange, ExchangeType, Queue};

    #[derive(Debug)]
    struct InMemoryQueue {
        queue: Queue,
        messages: Mutex<VecDeque<Vec<u8>>>,
        consumers: DashMap<String, mpsc::Sender<ConsumerControl>>,
    }

    impl InMemoryQueue {
        fn new(queue: Queue) -> Self {
            Self {
                queue,
                messages: Mutex::new(VecDeque::new()),
                consumers: DashMap::new(),
            }
        }

        async fn push(&self, message: Vec<u8>) {
            let mut messages = self.messages.lock().await;
            messages.push_back(message);
        }

        async fn pop(&self) -> Option<Vec<u8>> {
            let mut messages = self.messages.lock().await;
            messages.pop_front()
        }

        fn len(&self) -> usize {
            self.messages.try_lock().map(|q| q.len()).unwrap_or(0)
        }
    }

    #[derive(Debug)]
    struct InMemoryExchange {
        exchange: Exchange,
        bindings: RwLock<Vec<InMemoryBinding>>,
    }

    impl InMemoryExchange {
        fn new(exchange: Exchange) -> Self {
            Self {
                exchange,
                bindings: RwLock::new(Vec::new()),
            }
        }

        fn matches(&self, routing_key: &str, headers: &HashMap<String, String>) -> bool {
            match self.exchange.kind {
                ExchangeType::Direct => true,
                ExchangeType::Fanout => true,
                ExchangeType::Topic => true,
                ExchangeType::Headers => {
                    if let Some(match_type) = self.exchange.arguments.get("x-match") {
                        let match_all = match_type == "all";

                        let mut matched = 0;
                        let mut required = 0;

                        for (key, value) in &self.exchange.arguments {
                            if key != "x-match" {
                                required += 1;
                                if headers.get(key) == Some(value) {
                                    matched += 1;
                                }
                            }
                        }

                        if match_all {
                            matched == required
                        } else {
                            matched > 0
                        }
                    } else {
                        true
                    }
                }
                ExchangeType::ConsistentHash => {
                    // Simplified: just return true for the example
                    true
                }
            }
        }
    }

    #[derive(Debug)]
    struct InMemoryBinding {
        source: String,
        destination: BindingDestination,
        routing_key: String,
        arguments: HashMap<String, String>,
    }

    #[derive(Debug)]
    pub struct InMemoryBroker {
        config: BrokerConfig,
        exchanges: DashMap<String, Arc<InMemoryExchange>>,
        queues: DashMap<String, Arc<InMemoryQueue>>,
        status: RwLock<ConnectionStatus>,
        metrics: RwLock<BrokerMetrics>,
        json_serializer: Arc<navius_messaging::serialization::JsonSerializer>,
    }

    impl InMemoryBroker {
        pub fn new(config: BrokerConfig) -> Self {
            let mut broker = Self {
                config,
                exchanges: DashMap::new(),
                queues: DashMap::new(),
                status: RwLock::new(ConnectionStatus::Disconnected),
                metrics: RwLock::new(BrokerMetrics::default()),
                json_serializer: Arc::new(navius_messaging::serialization::JsonSerializer),
            };

            // Create default exchange
            let default_exchange = Exchange::direct("");
            broker.exchanges.insert(
                "".to_string(),
                Arc::new(InMemoryExchange::new(default_exchange)),
            );

            broker
        }

        async fn route_message(
            &self,
            exchange_name: &str,
            routing_key: &str,
            headers: HashMap<String, String>,
            data: Vec<u8>,
        ) -> MessagingResult<()> {
            let exchange = self
                .exchanges
                .get(exchange_name)
                .ok_or_else(|| MessagingError::ExchangeNotFound(exchange_name.to_string()))?;

            if !exchange.matches(routing_key, &headers) {
                return Err(MessagingError::PublishError("No matching exchange".into()));
            }

            let bindings = exchange.bindings.read().await.clone();

            for binding in bindings {
                if binding.routing_key == routing_key
                    || binding.routing_key.is_empty()
                    || exchange.exchange.kind == ExchangeType::Fanout
                {
                    match &binding.destination {
                        BindingDestination::Queue(queue_name) => {
                            if let Some(queue) = self.queues.get(queue_name) {
                                queue.push(data.clone()).await;
                            }
                        }
                        BindingDestination::Exchange(ex_name) => {
                            // Route to another exchange
                            self.route_message(ex_name, routing_key, headers.clone(), data.clone())
                                .await?;
                        }
                    }
                }
            }

            Ok(())
        }
    }

    #[async_trait]
    impl MessageBroker for InMemoryBroker {
        fn id(&self) -> &str {
            &self.config.id
        }

        fn name(&self) -> &str {
            &self.config.name
        }

        fn broker_type(&self) -> &str {
            &self.config.broker_type
        }

        fn config(&self) -> &BrokerConfig {
            &self.config
        }

        async fn connect(&self) -> MessagingResult<()> {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Connected;
            Ok(())
        }

        async fn disconnect(&self) -> MessagingResult<()> {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Disconnected;
            Ok(())
        }

        async fn is_connected(&self) -> bool {
            let status = self.status.read().await;
            *status == ConnectionStatus::Connected
        }

        async fn connection_status(&self) -> ConnectionStatus {
            *self.status.read().await
        }

        async fn metrics(&self) -> BrokerMetrics {
            self.metrics.read().await.clone()
        }

        async fn declare_queue(&self, queue: &Queue) -> MessagingResult<Queue> {
            let queue_obj = Arc::new(InMemoryQueue::new(queue.clone()));
            self.queues.insert(queue.name.clone(), queue_obj);

            // Bind to default exchange for direct routing
            let binding = InMemoryBinding {
                source: "".to_string(),
                destination: BindingDestination::Queue(queue.name.clone()),
                routing_key: queue.name.clone(),
                arguments: HashMap::new(),
            };

            if let Some(exchange) = self.exchanges.get("") {
                let mut bindings = exchange.bindings.write().await;
                bindings.push(binding);
            }

            Ok(queue.clone())
        }

        async fn delete_queue(
            &self,
            name: &str,
            _if_unused: bool,
            _if_empty: bool,
        ) -> MessagingResult<()> {
            self.queues.remove(name);
            Ok(())
        }

        async fn purge_queue(&self, name: &str) -> MessagingResult<()> {
            if let Some(queue) = self.queues.get(name) {
                let mut messages = queue.messages.lock().await;
                messages.clear();
            }
            Ok(())
        }

        async fn declare_exchange(&self, exchange: &Exchange) -> MessagingResult<Exchange> {
            let exchange_obj = Arc::new(InMemoryExchange::new(exchange.clone()));
            self.exchanges.insert(exchange.name.clone(), exchange_obj);
            Ok(exchange.clone())
        }

        async fn delete_exchange(&self, name: &str, _if_unused: bool) -> MessagingResult<()> {
            self.exchanges.remove(name);
            Ok(())
        }

        async fn bind_queue(
            &self,
            queue: &str,
            exchange: &str,
            routing_key: &str,
            arguments: Option<HashMap<String, String>>,
        ) -> MessagingResult<Binding> {
            if !self.queues.contains_key(queue) {
                return Err(MessagingError::QueueNotFound(queue.to_string()));
            }

            let exchange_obj = self
                .exchanges
                .get(exchange)
                .ok_or_else(|| MessagingError::ExchangeNotFound(exchange.to_string()))?;

            let binding = InMemoryBinding {
                source: exchange.to_string(),
                destination: BindingDestination::Queue(queue.to_string()),
                routing_key: routing_key.to_string(),
                arguments: arguments.unwrap_or_default(),
            };

            let mut bindings = exchange_obj.bindings.write().await;
            bindings.push(binding);

            Ok(Binding::queue_binding(exchange, queue, routing_key))
        }

        async fn unbind_queue(
            &self,
            queue: &str,
            exchange: &str,
            routing_key: &str,
        ) -> MessagingResult<()> {
            if let Some(exchange_obj) = self.exchanges.get(exchange) {
                let mut bindings = exchange_obj.bindings.write().await;
                bindings.retain(|b| {
                    if let BindingDestination::Queue(q) = &b.destination {
                        !(q == queue && b.routing_key == routing_key)
                    } else {
                        true
                    }
                });
            }
            Ok(())
        }

        async fn bind_exchange(
            &self,
            destination: &str,
            source: &str,
            routing_key: &str,
            arguments: Option<HashMap<String, String>>,
        ) -> MessagingResult<Binding> {
            if !self.exchanges.contains_key(destination) {
                return Err(MessagingError::ExchangeNotFound(destination.to_string()));
            }

            let source_obj = self
                .exchanges
                .get(source)
                .ok_or_else(|| MessagingError::ExchangeNotFound(source.to_string()))?;

            let binding = InMemoryBinding {
                source: source.to_string(),
                destination: BindingDestination::Exchange(destination.to_string()),
                routing_key: routing_key.to_string(),
                arguments: arguments.unwrap_or_default(),
            };

            let mut bindings = source_obj.bindings.write().await;
            bindings.push(binding);

            Ok(Binding::exchange_binding(source, destination, routing_key))
        }

        async fn unbind_exchange(
            &self,
            destination: &str,
            source: &str,
            routing_key: &str,
        ) -> MessagingResult<()> {
            if let Some(source_obj) = self.exchanges.get(source) {
                let mut bindings = source_obj.bindings.write().await;
                bindings.retain(|b| {
                    if let BindingDestination::Exchange(e) = &b.destination {
                        !(e == destination && b.routing_key == routing_key)
                    } else {
                        true
                    }
                });
            }
            Ok(())
        }

        async fn publish<T: serde::Serialize + Send + Sync>(
            &self,
            message: &Message<T>,
            options: Option<PublishOptions>,
        ) -> MessagingResult<()> {
            let options = options.unwrap_or_default();
            let exchange = options.exchange;
            let routing_key = options.routing_key.unwrap_or_else(|| message.topic.clone());

            // Serialize the message
            let data = self.json_serializer.serialize(message)?;

            // Route the message
            let headers = message.headers.all().clone();
            self.route_message(&exchange, &routing_key, headers, data)
                .await?;

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.published_messages += 1;

            Ok(())
        }

        async fn publish_with_confirm<T: serde::Serialize + Send + Sync>(
            &self,
            message: &Message<T>,
            options: Option<PublishOptions>,
            _timeout: Option<Duration>,
        ) -> MessagingResult<()> {
            // For the simple example, just call publish
            self.publish(message, options).await
        }

        async fn subscribe<T, F>(
            &self,
            queue_name: &str,
            handler: F,
            options: Option<ConsumerOptions>,
        ) -> MessagingResult<ConsumerHandle>
        where
            T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
            F: MessageHandler<T> + 'static,
        {
            let options = options.unwrap_or_default();
            let tag = options
                .consumer_tag
                .unwrap_or_else(|| format!("consumer-{}", Uuid::new_v4()));

            let queue = self
                .queues
                .get(queue_name)
                .ok_or_else(|| MessagingError::QueueNotFound(queue_name.to_string()))?;

            let (control_tx, mut control_rx) = mpsc::channel(10);
            let queue_clone = queue.clone();
            let json_serializer = self.json_serializer.clone();
            let metrics_clone = self.metrics.clone();

            // Spawn a task to consume messages
            tokio::spawn(async move {
                let mut running = true;
                let mut delivery_tag: u64 = 0;

                while running {
                    tokio::select! {
                        control_cmd = control_rx.recv() => {
                            match control_cmd {
                                Some(ConsumerControl::Cancel) => {
                                    running = false;
                                },
                                Some(ConsumerControl::Pause) => {
                                    // Wait for Resume control command
                                    while let Some(cmd) = control_rx.recv().await {
                                        if let ConsumerControl::Resume = cmd {
                                            break;
                                        }
                                        if let ConsumerControl::Cancel = cmd {
                                            running = false;
                                            break;
                                        }
                                    }
                                },
                                Some(ConsumerControl::Resume) => {
                                    // Already running, do nothing
                                },
                                Some(ConsumerControl::SetPrefetch(_)) => {
                                    // Not implemented for in-memory broker
                                },
                                None => {
                                    running = false;
                                }
                            }
                        },

                        _ = sleep(Duration::from_millis(100)), if running => {
                            if let Some(data) = queue_clone.pop().await {
                                delivery_tag += 1;

                                // Deserialize and process
                                match json_serializer.deserialize::<Message<T>>(&data) {
                                    Ok(msg) => {
                                        let received = ReceivedMessage {
                                            message: msg.clone(),
                                            delivery_tag,
                                            redelivered: false,
                                            exchange: "".to_string(),
                                            routing_key: msg.topic.clone(),
                                            consumer_tag: tag.clone(),
                                        };

                                        // Update metrics
                                        let mut metrics = metrics_clone.write().await;
                                        metrics.consumed_messages += 1;

                                        // Process the message
                                        let result = handler.handle(&received);
                                        if let Ok(ack) = result {
                                            match ack {
                                                MessageAcknowledgment::Ack => {
                                                    metrics.acknowledged_messages += 1;
                                                },
                                                MessageAcknowledgment::Reject { requeue } => {
                                                    metrics.rejected_messages += 1;
                                                    if requeue {
                                                        queue_clone.push(data).await;
                                                    }
                                                },
                                                MessageAcknowledgment::Nack { requeue, .. } => {
                                                    metrics.rejected_messages += 1;
                                                    if requeue {
                                                        queue_clone.push(data).await;
                                                    }
                                                },
                                            }
                                        } else {
                                            metrics.consume_errors += 1;
                                        }
                                    },
                                    Err(err) => {
                                        let mut metrics = metrics_clone.write().await;
                                        metrics.consume_errors += 1;
                                        eprintln!("Error deserializing message: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }

                // Remove consumer from queue
                queue_clone.consumers.remove(&tag);
            });

            // Register consumer
            queue.consumers.insert(tag.clone(), control_tx.clone());

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_consumers += 1;

            Ok(ConsumerHandle::new(tag, queue_name, control_tx))
        }

        async fn subscribe_filtered<T, F, M>(
            &self,
            queue_name: &str,
            handler: F,
            filter: M,
            options: Option<ConsumerOptions>,
        ) -> MessagingResult<ConsumerHandle>
        where
            T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
            F: MessageHandler<T> + 'static,
            M: MessageFilter<T> + 'static,
        {
            let options = options.unwrap_or_default();
            let tag = options
                .consumer_tag
                .unwrap_or_else(|| format!("consumer-{}", Uuid::new_v4()));

            let queue = self
                .queues
                .get(queue_name)
                .ok_or_else(|| MessagingError::QueueNotFound(queue_name.to_string()))?;

            let (control_tx, mut control_rx) = mpsc::channel(10);
            let queue_clone = queue.clone();
            let json_serializer = self.json_serializer.clone();
            let metrics_clone = self.metrics.clone();

            // Spawn a task to consume messages
            tokio::spawn(async move {
                let mut running = true;
                let mut delivery_tag: u64 = 0;

                while running {
                    tokio::select! {
                        control_cmd = control_rx.recv() => {
                            match control_cmd {
                                Some(ConsumerControl::Cancel) => {
                                    running = false;
                                },
                                Some(ConsumerControl::Pause) => {
                                    // Wait for Resume control command
                                    while let Some(cmd) = control_rx.recv().await {
                                        if let ConsumerControl::Resume = cmd {
                                            break;
                                        }
                                        if let ConsumerControl::Cancel = cmd {
                                            running = false;
                                            break;
                                        }
                                    }
                                },
                                Some(ConsumerControl::Resume) => {
                                    // Already running, do nothing
                                },
                                Some(ConsumerControl::SetPrefetch(_)) => {
                                    // Not implemented for in-memory broker
                                },
                                None => {
                                    running = false;
                                }
                            }
                        },

                        _ = sleep(Duration::from_millis(100)), if running => {
                            if let Some(data) = queue_clone.pop().await {
                                delivery_tag += 1;

                                // Deserialize and process
                                match json_serializer.deserialize::<Message<T>>(&data) {
                                    Ok(msg) => {
                                        // Apply the filter
                                        if filter.matches(&msg) {
                                            let received = ReceivedMessage {
                                                message: msg.clone(),
                                                delivery_tag,
                                                redelivered: false,
                                                exchange: "".to_string(),
                                                routing_key: msg.topic.clone(),
                                                consumer_tag: tag.clone(),
                                            };

                                            // Update metrics
                                            let mut metrics = metrics_clone.write().await;
                                            metrics.consumed_messages += 1;

                                            // Process the message
                                            let result = handler.handle(&received);
                                            if let Ok(ack) = result {
                                                match ack {
                                                    MessageAcknowledgment::Ack => {
                                                        metrics.acknowledged_messages += 1;
                                                    },
                                                    MessageAcknowledgment::Reject { requeue } => {
                                                        metrics.rejected_messages += 1;
                                                        if requeue {
                                                            queue_clone.push(data).await;
                                                        }
                                                    },
                                                    MessageAcknowledgment::Nack { requeue, .. } => {
                                                        metrics.rejected_messages += 1;
                                                        if requeue {
                                                            queue_clone.push(data).await;
                                                        }
                                                    },
                                                }
                                            } else {
                                                metrics.consume_errors += 1;
                                            }
                                        } else {
                                            // Message doesn't match the filter, put it back
                                            queue_clone.push(data).await;
                                        }
                                    },
                                    Err(err) => {
                                        let mut metrics = metrics_clone.write().await;
                                        metrics.consume_errors += 1;
                                        eprintln!("Error deserializing message: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }

                // Remove consumer from queue
                queue_clone.consumers.remove(&tag);
            });

            // Register consumer
            queue.consumers.insert(tag.clone(), control_tx.clone());

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_consumers += 1;

            Ok(ConsumerHandle::new(tag, queue_name, control_tx))
        }

        async fn consume<T>(
            &self,
            queue_name: &str,
            options: Option<ConsumerOptions>,
        ) -> MessagingResult<
            Box<dyn Stream<Item = Result<ReceivedMessage<T>, MessagingError>> + Send + Unpin>,
        >
        where
            T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
        {
            let options = options.unwrap_or_default();
            let tag = options
                .consumer_tag
                .unwrap_or_else(|| format!("consumer-{}", Uuid::new_v4()));

            let queue = self
                .queues
                .get(queue_name)
                .ok_or_else(|| MessagingError::QueueNotFound(queue_name.to_string()))?;

            let (control_tx, mut control_rx) = mpsc::channel(10);
            let (message_tx, message_rx) = mpsc::channel(100);

            let queue_clone = queue.clone();
            let json_serializer = self.json_serializer.clone();
            let metrics_clone = self.metrics.clone();

            // Spawn a task to consume messages
            tokio::spawn(async move {
                let mut running = true;
                let mut delivery_tag: u64 = 0;

                while running {
                    tokio::select! {
                        control_cmd = control_rx.recv() => {
                            match control_cmd {
                                Some(ConsumerControl::Cancel) => {
                                    running = false;
                                },
                                Some(ConsumerControl::Pause) => {
                                    // Wait for Resume control command
                                    while let Some(cmd) = control_rx.recv().await {
                                        if let ConsumerControl::Resume = cmd {
                                            break;
                                        }
                                        if let ConsumerControl::Cancel = cmd {
                                            running = false;
                                            break;
                                        }
                                    }
                                },
                                Some(ConsumerControl::Resume) => {
                                    // Already running, do nothing
                                },
                                Some(ConsumerControl::SetPrefetch(_)) => {
                                    // Not implemented for in-memory broker
                                },
                                None => {
                                    running = false;
                                }
                            }
                        },

                        _ = sleep(Duration::from_millis(100)), if running => {
                            if let Some(data) = queue_clone.pop().await {
                                delivery_tag += 1;

                                // Deserialize and process
                                match json_serializer.deserialize::<Message<T>>(&data) {
                                    Ok(msg) => {
                                        let received = ReceivedMessage {
                                            message: msg.clone(),
                                            delivery_tag,
                                            redelivered: false,
                                            exchange: "".to_string(),
                                            routing_key: msg.topic.clone(),
                                            consumer_tag: tag.clone(),
                                        };

                                        // Update metrics
                                        let mut metrics = metrics_clone.write().await;
                                        metrics.consumed_messages += 1;

                                        // Send to the stream
                                        if message_tx.send(Ok(received)).await.is_err() {
                                            // Receiver dropped, stop consuming
                                            running = false;
                                        }
                                    },
                                    Err(err) => {
                                        let mut metrics = metrics_clone.write().await;
                                        metrics.consume_errors += 1;

                                        // Send error to the stream
                                        if message_tx.send(Err(err)).await.is_err() {
                                            // Receiver dropped, stop consuming
                                            running = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Remove consumer from queue
                queue_clone.consumers.remove(&tag);
            });

            // Register consumer
            queue.consumers.insert(tag.clone(), control_tx);

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_consumers += 1;

            // Convert channel to stream
            let stream = Box::pin(futures::stream::unfold(message_rx, |mut rx| async move {
                rx.recv().await.map(|msg| (msg, rx))
            }));

            Ok(Box::new(stream)
                as Box<
                    dyn Stream<Item = Result<ReceivedMessage<T>, MessagingError>> + Send + Unpin,
                >)
        }

        async fn ack(&self, _delivery_tag: u64, _multiple: bool) -> MessagingResult<()> {
            // For simplicity, ACK is handled in the consume loop
            Ok(())
        }

        async fn reject(&self, _delivery_tag: u64, _requeue: bool) -> MessagingResult<()> {
            // For simplicity, reject is handled in the consume loop
            Ok(())
        }

        async fn nack(
            &self,
            _delivery_tag: u64,
            _multiple: bool,
            _requeue: bool,
        ) -> MessagingResult<()> {
            // For simplicity, nack is handled in the consume loop
            Ok(())
        }

        async fn message_count(&self, queue_name: &str) -> MessagingResult<u32> {
            let queue = self
                .queues
                .get(queue_name)
                .ok_or_else(|| MessagingError::QueueNotFound(queue_name.to_string()))?;

            Ok(queue.len() as u32)
        }

        async fn consumer_count(&self, queue_name: &str) -> MessagingResult<u32> {
            let queue = self
                .queues
                .get(queue_name)
                .ok_or_else(|| MessagingError::QueueNotFound(queue_name.to_string()))?;

            Ok(queue.consumers.len() as u32)
        }

        async fn create_reply_queue(&self) -> MessagingResult<Queue> {
            let queue_name = format!("reply-{}", Uuid::new_v4());
            let queue = Queue::new(&queue_name)
                .exclusive(true)
                .auto_delete(true)
                .durable(false);

            self.declare_queue(&queue).await
        }

        async fn ping(&self) -> MessagingResult<Duration> {
            // Simulate a ping by measuring the time it takes to execute
            let start = Instant::now();
            // Dummy operation
            let _status = self.is_connected().await;
            Ok(start.elapsed())
        }
    }

    pub struct InMemoryBrokerFactory;

    #[async_trait]
    impl MessageBrokerFactory for InMemoryBrokerFactory {
        async fn create_broker(
            &self,
            config: BrokerConfig,
        ) -> MessagingResult<Arc<dyn MessageBroker>> {
            let broker = InMemoryBroker::new(config);
            let broker = Arc::new(broker);
            broker.connect().await?;
            Ok(broker)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a broker config
    let config = BrokerConfig::new("in-memory-broker", "InMemory Broker Example", "in-memory");

    // Create a broker factory
    let factory = memory_broker::InMemoryBrokerFactory;

    // Create the broker
    let broker = factory.create_broker(config).await?;

    // Connect to the broker
    println!("Connecting to the broker...");
    broker.connect().await?;
    println!("Connected!");

    // Create the basic topology
    println!("Setting up messaging topology...");
    let builder = TopologyBuilder::new(broker.clone());

    // Create a topic exchange with topics
    let user_events_exchange = "user-events";
    let topics = vec![
        ("user-logins", "user.login"),
        ("user-signups", "user.signup"),
        ("user-updates", "user.update"),
    ];

    builder.build_topic(user_events_exchange, &topics).await?;
    println!("Topology created!");

    // Create a consumer handler
    let user_login_handler = |message: &ReceivedMessage<UserEvent>| -> MessageProcessingResult {
        let event = &message.message.payload;
        println!(
            "Received user login event: user_id={}, event_type={}, timestamp={}",
            event.user_id, event.event_type, event.timestamp
        );
        Ok(MessageAcknowledgment::Ack)
    };

    // Create a consumer
    println!("Setting up a consumer for user logins...");
    let consumer_options = ConsumerOptions::default().with_consumer_tag("user-login-consumer");

    let consumer_handle = broker
        .subscribe("user-logins", user_login_handler, Some(consumer_options))
        .await?;

    println!("Consumer ready: {}", consumer_handle.tag);

    // Publish some messages
    println!("Publishing some user events...");

    // Create and publish a login event
    let login_event = UserEvent {
        id: generate_id(),
        user_id: "user123".to_string(),
        event_type: "login".to_string(),
        timestamp: 1619712000,
        data: serde_json::json!({
            "ip": "192.168.1.1",
            "device": "mobile",
            "success": true
        }),
    };

    let login_message =
        Message::new(login_event, "user.login").with_correlation_id(generate_correlation_id());

    let publish_options = PublishOptions::new(user_events_exchange)
        .with_routing_key("user.login")
        .with_confirm(true, Some(Duration::from_secs(5)));

    // Publish the message
    broker
        .publish(&login_message, Some(publish_options))
        .await?;
    println!("Published login event");

    // Create and publish a signup event
    let signup_event = UserEvent {
        id: generate_id(),
        user_id: "newuser456".to_string(),
        event_type: "signup".to_string(),
        timestamp: 1619712100,
        data: serde_json::json!({
            "email": "newuser@example.com",
            "referral": "website"
        }),
    };

    let signup_message =
        Message::new(signup_event, "user.signup").with_correlation_id(generate_correlation_id());

    let publish_options = PublishOptions::new(user_events_exchange).with_routing_key("user.signup");

    // Publish the message
    broker
        .publish(&signup_message, Some(publish_options))
        .await?;
    println!("Published signup event");

    // Wait a bit for messages to be processed
    println!("Waiting for messages to be processed...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Cancel the consumer
    println!("Cancelling consumer...");
    consumer_handle.cancel().await?;

    // Get metrics
    let metrics = broker.metrics().await;
    println!("Broker metrics:");
    println!("- Messages published: {}", metrics.published_messages);
    println!("- Messages consumed: {}", metrics.consumed_messages);
    println!("- Messages acknowledged: {}", metrics.acknowledged_messages);

    // Disconnect from the broker
    println!("Disconnecting from the broker...");
    broker.disconnect().await?;
    println!("Disconnected!");

    println!("Example completed successfully!");
    Ok(())
}
