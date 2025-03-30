# filtered_events API Inventory

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

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| InMemoryEventBroker | memory.rs | 44 | ✅ Complete |
| InMemoryEventBrokerFactory | memory.rs | 612 | ⚠️ Partial |
| Event<T> | event.rs | 13 | ⚠️ Partial |
| EventEnvelope | event.rs | 105 | ⚠️ Partial |
| EventFilterConfig | event.rs | 194 | ⚠️ Partial |
| SubscriptionOptions | event.rs | 315 | ⚠️ Partial |
| SubscriptionInfo | event.rs | 383 | ⚠️ Partial |
| EventBrokerConfig | broker.rs | 13 | ⚠️ Partial |
| BrokerInfo | broker.rs | 96 | ⚠️ Partial |
| TopicInfo | broker.rs | 115 | ✅ Complete |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| EventError | error.rs | 10 | ⚠️ Partial |
| EventPriority | error.rs | 78 | ⚠️ Partial |
| DeliveryStatus | error.rs | 100 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| EventBroker: | broker.rs | 134 | ✅ Complete |
| EventBrokerFactory: | broker.rs | 213 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| fn | lib.rs | 35 | ⚠️ Partial |
| fn | lib.rs | 41 | ⚠️ Partial |
| new(config: | memory.rs | 63 | ✅ Complete |
| start_cleanup_task(&self) | memory.rs | 74 | ⚠️ Partial |
| new() | memory.rs | 616 | ⚠️ Partial |
| new( | event.rs | 44 | ✅ Complete |
| with_priority( | event.rs | 64 | ⚠️ Partial |
| with_correlation_id(mut | event.rs | 85 | ⚠️ Partial |
| with_metadata(mut | event.rs | 91 | ⚠️ Partial |
| with_metadata_map(mut | event.rs | 97 | ⚠️ Partial |
| from_event<T>(event: | event.rs | 136 | ✅ Complete |
| try_into_event<T>(&self) | event.rs | 164 | ⚠️ Partial |
| new() | event.rs | 213 | ✅ Complete |
| with_event_types(mut | event.rs | 224 | ⚠️ Partial |
| with_sources(mut | event.rs | 230 | ⚠️ Partial |
| with_min_priority(mut | event.rs | 236 | ⚠️ Partial |
| with_correlation_id(mut | event.rs | 242 | ⚠️ Partial |
| with_metadata(mut | event.rs | 248 | ⚠️ Partial |
| matches(&self, | event.rs | 261 | ⚠️ Partial |
| new() | event.rs | 334 | ✅ Complete |
| with_buffer_size(mut | event.rs | 345 | ⚠️ Partial |
| with_filter(mut | event.rs | 351 | ⚠️ Partial |
| with_historical_events(mut | event.rs | 357 | ⚠️ Partial |
| with_max_retries(mut | event.rs | 363 | ⚠️ Partial |
| with_name(mut | event.rs | 369 | ⚠️ Partial |
| new() | broker.rs | 38 | ✅ Complete |
| with_max_topics(mut | broker.rs | 51 | ⚠️ Partial |
| with_max_subscribers_per_topic(mut | broker.rs | 57 | ⚠️ Partial |
| with_event_retention_seconds(mut | broker.rs | 63 | ⚠️ Partial |
| with_max_retained_events(mut | broker.rs | 69 | ⚠️ Partial |
| with_topic_validation(mut | broker.rs | 75 | ⚠️ Partial |
| with_topic_namespace(mut | broker.rs | 82 | ⚠️ Partial |

