# request_reply API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- async-trait
- chrono
- futures
- serde
- serde_json
- thiserror
- tokio
- tokio-stream
- tracing
- uuid
- metrics
- navius-core
- navius-event
- navius-plugin

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Message<T> | message.rs | 16 | ⚠️ Partial |
| MessageHeaders | message.rs | 219 | ⚠️ Partial |
| ReceivedMessage<T> | message.rs | 289 | ⚠️ Partial |
| TopicFilter | message.rs | 395 | ✅ Complete |
| HeaderFilter | message.rs | 425 | ⚠️ Partial |
| AndFilter<T> | message.rs | 454 | ⚠️ Partial |
| OrFilter<T> | message.rs | 473 | ⚠️ Partial |
| RequestReply<T> | util.rs | 39 | ⚠️ Partial |
| RateLimiter | util.rs | 190 | ⚠️ Partial |
| MessageBuffer<T> | util.rs | 249 | ⚠️ Partial |
| DeduplicationFilter | util.rs | 319 | ⚠️ Partial |
| StandardHeaders; | util.rs | 401 | ⚠️ Partial |
| MessageTracker | util.rs | 436 | ✅ Complete |
| MessageInfo | util.rs | 520 | ⚠️ Partial |
| BrokerConfig | config.rs | 9 | ⚠️ Partial |
| ConnectionConfig | config.rs | 123 | ⚠️ Partial |
| TlsConfig | config.rs | 234 | ⚠️ Partial |
| RetryConfig | config.rs | 301 | ⚠️ Partial |
| PoolConfig | config.rs | 379 | ⚠️ Partial |
| RecoveryConfig | config.rs | 458 | ⚠️ Partial |
| ConsumerDefaultConfig | config.rs | 539 | ⚠️ Partial |
| PublisherDefaultConfig | config.rs | 624 | ⚠️ Partial |
| EventConfig | config.rs | 726 | ⚠️ Partial |
| MessagingMetrics | metrics.rs | 10 | ⚠️ Partial |
| QueueMetrics | metrics.rs | 302 | ⚠️ Partial |
| ExchangeMetrics | metrics.rs | 333 | ✅ Complete |
| MetricsSummary | metrics.rs | 355 | ✅ Complete |
| LatencyMeasurer | metrics.rs | 464 | ⚠️ Partial |
| JsonSerializer; | serialization.rs | 20 | ✅ Complete |
| CborSerializer; | serialization.rs | 41 | ⚠️ Partial |
| MessagePackSerializer; | serialization.rs | 62 | ⚠️ Partial |
| BincodeSerializer; | serialization.rs | 86 | ⚠️ Partial |
| BinaryData | serialization.rs | 157 | ⚠️ Partial |
| AnyMessage | serialization.rs | 202 | ⚠️ Partial |
| TypedMessageDeserializer<T> | serialization.rs | 251 | ⚠️ Partial |
| PublishOptions | publisher.rs | 14 | ⚠️ Partial |
| PublishResult | publisher.rs | 130 | ⚠️ Partial |
| BrokerPublisher | publisher.rs | 167 | ⚠️ Partial |
| BatchPublisher | publisher.rs | 299 | ⚠️ Partial |
| BrokerMetrics | broker.rs | 176 | ✅ Complete |
| ConsumerHandle | broker.rs | 220 | ✅ Complete |
| TopologyBuilder | broker.rs | 301 | ✅ Complete |
| Exchange | topology.rs | 44 | ⚠️ Partial |
| Queue | topology.rs | 131 | ⚠️ Partial |
| Binding | topology.rs | 274 | ⚠️ Partial |
| TopologyInfo | topology.rs | 355 | ⚠️ Partial |
| ExchangeInfo | topology.rs | 368 | ✅ Complete |
| QueueInfo | topology.rs | 390 | ✅ Complete |
| BindingInfo | topology.rs | 415 | ✅ Complete |
| ConsumerOptions | consumer.rs | 10 | ⚠️ Partial |
| ConsumerInfo | consumer.rs | 257 | ⚠️ Partial |
| RetryingHandler<T, | consumer.rs | 300 | ✅ Complete |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| MessageAcknowledgment | message.rs | 358 | ⚠️ Partial |
| MessageStatus | util.rs | 567 | ⚠️ Partial |
| MessagingError | error.rs | 11 | ⚠️ Partial |
| DeliveryMode | error.rs | 120 | ⚠️ Partial |
| AcknowledgmentMode | error.rs | 139 | ⚠️ Partial |
| ConnectionStatus | error.rs | 158 | ⚠️ Partial |
| MessagingEventType | config.rs | 774 | ⚠️ Partial |
| ConsumerControl | broker.rs | 286 | ⚠️ Partial |
| ExchangeType | topology.rs | 7 | ⚠️ Partial |
| QueueOverflowBehavior | topology.rs | 251 | ⚠️ Partial |
| BindingDestination | topology.rs | 290 | ✅ Complete |
| BindingDestinationType | topology.rs | 434 | ✅ Complete |
| MessageOffset | consumer.rs | 168 | ⚠️ Partial |
| RetryStrategy | consumer.rs | 184 | ✅ Complete |
| ConsumerStatus | consumer.rs | 285 | ✅ Complete |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| MessageHandler<T>: | message.rs | 373 | ✅ Complete |
| MessageFilter<T>: | message.rs | 388 | ⚠️ Partial |
| MessageSerializer: | serialization.rs | 8 | ⚠️ Partial |
| StringSerializer | serialization.rs | 110 | ⚠️ Partial |
| MessagePublisher: | publisher.rs | 149 | ✅ Complete |
| MessageBroker: | broker.rs | 20 | ⚠️ Partial |
| MessageBrokerFactory: | broker.rs | 213 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(payload: | message.rs | 53 | ✅ Complete |
| with_id(mut | message.rs | 70 | ⚠️ Partial |
| with_topic(mut | message.rs | 76 | ⚠️ Partial |
| with_header(mut | message.rs | 82 | ⚠️ Partial |
| with_headers(mut | message.rs | 88 | ⚠️ Partial |
| with_expiration(mut | message.rs | 94 | ⚠️ Partial |
| with_ttl(mut | message.rs | 100 | ⚠️ Partial |
| with_priority(mut | message.rs | 106 | ⚠️ Partial |
| with_delivery_mode(mut | message.rs | 112 | ⚠️ Partial |
| with_correlation_id(mut | message.rs | 118 | ⚠️ Partial |
| with_reply_to( | message.rs | 124 | ⚠️ Partial |
| is_expired(&self) | message.rs | 135 | ⚠️ Partial |
| time_to_expiration(&self) | message.rs | 147 | ⚠️ Partial |
| convert<U>(&self, | message.rs | 153 | ⚠️ Partial |
| create_reply<R>(&self, | message.rs | 180 | ⚠️ Partial |
| new() | message.rs | 225 | ⚠️ Partial |
| insert<K, | message.rs | 232 | ⚠️ Partial |
| get(&self, | message.rs | 241 | ⚠️ Partial |
| remove(&mut | message.rs | 246 | ⚠️ Partial |
| contains_key(&self, | message.rs | 251 | ⚠️ Partial |
| all(&self) | message.rs | 256 | ⚠️ Partial |
| extend<K, | message.rs | 261 | ⚠️ Partial |
| clear(&mut | message.rs | 272 | ⚠️ Partial |
| len(&self) | message.rs | 277 | ⚠️ Partial |
| is_empty(&self) | message.rs | 282 | ⚠️ Partial |
| new( | message.rs | 311 | ✅ Complete |
| inner(&self) | message.rs | 330 | ⚠️ Partial |
| convert<U>( | message.rs | 335 | ⚠️ Partial |
| new(topics: | message.rs | 401 | ⚠️ Partial |
| matches_topic(&self, | message.rs | 406 | ⚠️ Partial |
| new(required_headers: | message.rs | 432 | ✅ Complete |
| new(filters: | message.rs | 460 | ⚠️ Partial |
| new(filters: | message.rs | 479 | ⚠️ Partial |
| generate_id() | util.rs | 18 | ⚠️ Partial |
| generate_correlation_id() | util.rs | 23 | ⚠️ Partial |
| generate_consumer_tag(prefix: | util.rs | 28 | ⚠️ Partial |
| create_reply_topic(original_topic: | util.rs | 33 | ⚠️ Partial |
| fn | util.rs | 61 | ✅ Complete |
| fn | util.rs | 124 | ⚠️ Partial |
| reply_queue(&self) | util.rs | 171 | ⚠️ Partial |
| fn | util.rs | 176 | ⚠️ Partial |
| new(rate: | util.rs | 203 | ✅ Complete |
| fn | util.rs | 212 | ⚠️ Partial |
| fn | util.rs | 231 | ⚠️ Partial |
| fn | util.rs | 242 | ⚠️ Partial |
| new(max_size: | util.rs | 265 | ✅ Complete |
| add(&mut | util.rs | 275 | ⚠️ Partial |
| should_flush(&self) | util.rs | 284 | ⚠️ Partial |
| flush(&mut | util.rs | 301 | ⚠️ Partial |
| len(&self) | util.rs | 308 | ⚠️ Partial |
| is_empty(&self) | util.rs | 313 | ⚠️ Partial |
| new(ttl: | util.rs | 335 | ✅ Complete |
| fn | util.rs | 345 | ⚠️ Partial |
| fn | util.rs | 351 | ⚠️ Partial |
| fn | util.rs | 394 | ⚠️ Partial |
| new() | util.rs | 443 | ✅ Complete |
| track_publish(&self, | util.rs | 450 | ⚠️ Partial |
| track_delivery(&self, | util.rs | 467 | ⚠️ Partial |
| track_ack(&self, | util.rs | 476 | ⚠️ Partial |
| get_message_info(&self, | util.rs | 485 | ⚠️ Partial |
| get_in_flight_messages(&self) | util.rs | 490 | ⚠️ Partial |
| remove(&self, | util.rs | 501 | ⚠️ Partial |
| cleanup_old(&self, | util.rs | 506 | ⚠️ Partial |
| delivery_latency(&self) | util.rs | 545 | ✅ Complete |
| processing_time(&self) | util.rs | 551 | ⚠️ Partial |
| total_latency(&self) | util.rs | 559 | ⚠️ Partial |
| new( | config.rs | 46 | ✅ Complete |
| with_connection(mut | config.rs | 67 | ⚠️ Partial |
| with_pool(mut | config.rs | 73 | ⚠️ Partial |
| with_recovery(mut | config.rs | 79 | ⚠️ Partial |
| with_consumer(mut | config.rs | 85 | ⚠️ Partial |
| with_publisher(mut | config.rs | 91 | ⚠️ Partial |
| with_default_exchange(mut | config.rs | 97 | ⚠️ Partial |
| with_events(mut | config.rs | 103 | ⚠️ Partial |
| with_option(mut | config.rs | 109 | ⚠️ Partial |
| with_options(mut | config.rs | 115 | ⚠️ Partial |
| new(uri: | config.rs | 154 | ✅ Complete |
| with_credentials( | config.rs | 169 | ⚠️ Partial |
| with_vhost(mut | config.rs | 180 | ⚠️ Partial |
| with_timeout(mut | config.rs | 186 | ⚠️ Partial |
| with_heartbeat(mut | config.rs | 192 | ⚠️ Partial |
| with_tls(mut | config.rs | 198 | ⚠️ Partial |
| with_retry(mut | config.rs | 204 | ⚠️ Partial |
| with_property(mut | config.rs | 210 | ⚠️ Partial |
| new(ca_cert_path: | config.rs | 265 | ⚠️ Partial |
| with_client_cert( | config.rs | 276 | ⚠️ Partial |
| with_verify_peer(mut | config.rs | 287 | ⚠️ Partial |
| with_verify_hostname(mut | config.rs | 293 | ⚠️ Partial |
| new( | config.rs | 332 | ⚠️ Partial |
| with_jitter(mut | config.rs | 348 | ⚠️ Partial |
| calculate_delay(&self, | config.rs | 354 | ⚠️ Partial |
| new(min_size: | config.rs | 418 | ⚠️ Partial |
| with_acquire_timeout(mut | config.rs | 431 | ⚠️ Partial |
| with_idle_timeout(mut | config.rs | 437 | ⚠️ Partial |
| with_max_lifetime(mut | config.rs | 443 | ⚠️ Partial |
| with_testing(mut | config.rs | 449 | ⚠️ Partial |
| new(enabled: | config.rs | 497 | ⚠️ Partial |
| with_recovery_types( | config.rs | 510 | ⚠️ Partial |
| with_max_attempts(mut | config.rs | 525 | ⚠️ Partial |
| with_backoff(mut | config.rs | 531 | ⚠️ Partial |
| new(prefetch_count: | config.rs | 578 | ⚠️ Partial |
| with_ack_mode(mut | config.rs | 591 | ⚠️ Partial |
| with_tag_prefix(mut | config.rs | 597 | ⚠️ Partial |
| with_timeout(mut | config.rs | 603 | ⚠️ Partial |
| with_auto_recover(mut | config.rs | 609 | ⚠️ Partial |
| with_retry(mut | config.rs | 615 | ⚠️ Partial |
| new(default_exchange: | config.rs | 671 | ⚠️ Partial |
| with_confirms(mut | config.rs | 686 | ⚠️ Partial |
| with_mandatory(mut | config.rs | 693 | ⚠️ Partial |
| with_immediate(mut | config.rs | 699 | ⚠️ Partial |
| with_expiration(mut | config.rs | 705 | ⚠️ Partial |
| with_priority(mut | config.rs | 711 | ⚠️ Partial |
| with_retry(mut | config.rs | 717 | ⚠️ Partial |
| new(enabled: | config.rs | 753 | ⚠️ Partial |
| with_event_types(mut | config.rs | 766 | ⚠️ Partial |
| new() | metrics.rs | 89 | ⚠️ Partial |
| shared() | metrics.rs | 94 | ⚠️ Partial |
| uptime(&self) | metrics.rs | 99 | ⚠️ Partial |
| record_publish(&self, | metrics.rs | 104 | ⚠️ Partial |
| record_consume(&self, | metrics.rs | 119 | ⚠️ Partial |
| record_ack(&self) | metrics.rs | 134 | ⚠️ Partial |
| record_reject(&self) | metrics.rs | 139 | ⚠️ Partial |
| record_connection_error(&self) | metrics.rs | 144 | ⚠️ Partial |
| record_publish_error(&self) | metrics.rs | 149 | ⚠️ Partial |
| record_consume_error(&self) | metrics.rs | 154 | ⚠️ Partial |
| set_active_connections(&self, | metrics.rs | 159 | ⚠️ Partial |
| set_active_channels(&self, | metrics.rs | 164 | ⚠️ Partial |
| set_active_publishers(&self, | metrics.rs | 169 | ⚠️ Partial |
| set_active_consumers(&self, | metrics.rs | 174 | ⚠️ Partial |
| increment_custom(&self, | metrics.rs | 179 | ⚠️ Partial |
| get_custom(&self, | metrics.rs | 187 | ⚠️ Partial |
| set_custom(&self, | metrics.rs | 195 | ⚠️ Partial |
| avg_publish_latency(&self, | metrics.rs | 203 | ⚠️ Partial |
| avg_consume_latency(&self, | metrics.rs | 214 | ⚠️ Partial |
| fn | metrics.rs | 225 | ⚠️ Partial |
| fn | metrics.rs | 231 | ⚠️ Partial |
| fn | metrics.rs | 237 | ⚠️ Partial |
| fn | metrics.rs | 243 | ⚠️ Partial |
| fn | metrics.rs | 249 | ⚠️ Partial |
| fn | metrics.rs | 279 | ⚠️ Partial |
| calculate_throughput(&self) | metrics.rs | 401 | ✅ Complete |
| calculate_error_rate(&self) | metrics.rs | 410 | ⚠️ Partial |
| calculate_success_rate(&self) | metrics.rs | 420 | ⚠️ Partial |
| format(&self) | metrics.rs | 432 | ⚠️ Partial |
| new() | metrics.rs | 471 | ✅ Complete |
| end(&self) | metrics.rs | 478 | ⚠️ Partial |
| create_serializer(content_type: | serialization.rs | 142 | ⚠️ Partial |
| new(data: | serialization.rs | 167 | ✅ Complete |
| from_value<T: | serialization.rs | 175 | ⚠️ Partial |
| to_value<T: | serialization.rs | 186 | ⚠️ Partial |
| deserialize<T: | serialization.rs | 192 | ⚠️ Partial |
| new<T: | serialization.rs | 218 | ✅ Complete |
| deserialize<T: | serialization.rs | 234 | ⚠️ Partial |
| with_header(mut | serialization.rs | 239 | ⚠️ Partial |
| can_deserialize<T: | serialization.rs | 245 | ⚠️ Partial |
| new(serializer: | serialization.rs | 261 | ✅ Complete |
| deserialize(&self, | serialization.rs | 269 | ⚠️ Partial |
| content_type(&self) | serialization.rs | 274 | ⚠️ Partial |
| new(exchange: | publisher.rs | 65 | ⚠️ Partial |
| with_routing_key(mut | publisher.rs | 73 | ⚠️ Partial |
| with_mandatory(mut | publisher.rs | 79 | ⚠️ Partial |
| with_immediate(mut | publisher.rs | 85 | ⚠️ Partial |
| with_delivery_mode(mut | publisher.rs | 91 | ⚠️ Partial |
| with_expiration(mut | publisher.rs | 97 | ⚠️ Partial |
| with_priority(mut | publisher.rs | 103 | ⚠️ Partial |
| with_confirm(mut | publisher.rs | 109 | ⚠️ Partial |
| with_header(mut | publisher.rs | 116 | ⚠️ Partial |
| with_headers(mut | publisher.rs | 122 | ⚠️ Partial |
| new(broker: | publisher.rs | 177 | ✅ Complete |
| with_options(broker: | publisher.rs | 186 | ⚠️ Partial |
| set_default_options(&mut | publisher.rs | 194 | ⚠️ Partial |
| new( | publisher.rs | 350 | ⚠️ Partial |
| fn | publisher.rs | 419 | ⚠️ Partial |
| fn | publisher.rs | 447 | ⚠️ Partial |
| new( | broker.rs | 233 | ✅ Complete |
| fn | broker.rs | 246 | ⚠️ Partial |
| fn | broker.rs | 255 | ⚠️ Partial |
| fn | broker.rs | 264 | ⚠️ Partial |
| fn | broker.rs | 273 | ⚠️ Partial |
| new(broker: | broker.rs | 307 | ⚠️ Partial |
| fn | broker.rs | 312 | ⚠️ Partial |
| fn | broker.rs | 334 | ⚠️ Partial |
| fn | broker.rs | 356 | ⚠️ Partial |
| fn | broker.rs | 382 | ⚠️ Partial |
| new(name: | topology.rs | 66 | ✅ Complete |
| direct(name: | topology.rs | 78 | ⚠️ Partial |
| fanout(name: | topology.rs | 83 | ⚠️ Partial |
| topic(name: | topology.rs | 88 | ⚠️ Partial |
| headers(name: | topology.rs | 93 | ⚠️ Partial |
| durable(mut | topology.rs | 98 | ⚠️ Partial |
| auto_delete(mut | topology.rs | 104 | ⚠️ Partial |
| internal(mut | topology.rs | 110 | ⚠️ Partial |
| with_argument(mut | topology.rs | 116 | ⚠️ Partial |
| with_alternate_exchange(mut | topology.rs | 122 | ⚠️ Partial |
| new(name: | topology.rs | 150 | ✅ Complete |
| durable(mut | topology.rs | 161 | ⚠️ Partial |
| exclusive(mut | topology.rs | 167 | ⚠️ Partial |
| auto_delete(mut | topology.rs | 173 | ⚠️ Partial |
| with_argument(mut | topology.rs | 179 | ⚠️ Partial |
| with_message_ttl(mut | topology.rs | 185 | ⚠️ Partial |
| with_queue_ttl(mut | topology.rs | 192 | ⚠️ Partial |
| with_dead_letter( | topology.rs | 199 | ⚠️ Partial |
| with_max_length(mut | topology.rs | 216 | ⚠️ Partial |
| with_max_length_bytes(mut | topology.rs | 223 | ⚠️ Partial |
| with_overflow(mut | topology.rs | 230 | ⚠️ Partial |
| with_max_priority(mut | topology.rs | 237 | ⚠️ Partial |
| is_temporary(&self) | topology.rs | 244 | ⚠️ Partial |
| queue_binding( | topology.rs | 300 | ✅ Complete |
| exchange_binding( | topology.rs | 314 | ⚠️ Partial |
| with_argument(mut | topology.rs | 328 | ⚠️ Partial |
| with_header_match( | topology.rs | 334 | ⚠️ Partial |
| with_match_type(mut | topology.rs | 344 | ⚠️ Partial |
| new() | consumer.rs | 82 | ⚠️ Partial |
| with_consumer_tag(mut | consumer.rs | 87 | ⚠️ Partial |
| with_exclusive(mut | consumer.rs | 93 | ⚠️ Partial |
| with_auto_delete(mut | consumer.rs | 99 | ⚠️ Partial |
| with_prefetch_count(mut | consumer.rs | 105 | ⚠️ Partial |
| with_ack_mode(mut | consumer.rs | 111 | ⚠️ Partial |
| with_argument(mut | consumer.rs | 117 | ⚠️ Partial |
| with_offset(mut | consumer.rs | 123 | ⚠️ Partial |
| with_retry(mut | consumer.rs | 129 | ⚠️ Partial |
| with_forward_errors(mut | consumer.rs | 136 | ⚠️ Partial |
| with_no_local(mut | consumer.rs | 142 | ⚠️ Partial |
| with_timeout(mut | consumer.rs | 148 | ⚠️ Partial |
| with_auto_start(mut | consumer.rs | 154 | ⚠️ Partial |
| with_priority(mut | consumer.rs | 160 | ⚠️ Partial |
| calculate_delay(&self, | consumer.rs | 218 | ✅ Complete |
| new(inner: | consumer.rs | 316 | ✅ Complete |
| fn | consumer.rs | 332 | ⚠️ Partial |

