use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// An event emitted by a mock object
#[derive(Debug, Clone)]
pub struct MockEvent {
    /// The name of the mock object
    pub mock_name: String,

    /// The name of the method that was called
    pub method: String,

    /// The arguments that were passed to the method
    pub args: Vec<String>,

    /// The result of the method call
    pub result: MockEventResult,

    /// The time at which the event occurred
    pub timestamp: Instant,

    /// The duration of the method call
    pub duration: Duration,
}

/// The result of a method call
#[derive(Debug, Clone)]
pub enum MockEventResult {
    /// The method call returned successfully
    Success,

    /// The method call returned an error
    Error(String),
}

impl fmt::Display for MockEventResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockEventResult::Success => write!(f, "Success"),
            MockEventResult::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

impl MockEvent {
    /// Create a new mock event
    pub fn new<S: Into<String>>(mock_name: S, method: S) -> Self {
        Self {
            mock_name: mock_name.into(),
            method: method.into(),
            args: Vec::new(),
            result: MockEventResult::Success,
            timestamp: Instant::now(),
            duration: Duration::from_secs(0),
        }
    }

    /// Add an argument to the event
    pub fn with_arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments to the event
    pub fn with_args<S: Into<String>>(mut self, args: Vec<S>) -> Self {
        self.args = args.into_iter().map(|a| a.into()).collect();
        self
    }

    /// Set the result of the method call
    pub fn with_result(mut self, result: MockEventResult) -> Self {
        self.result = result;
        self
    }

    /// Set the duration of the method call
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// A registry for recording and querying mock events
#[derive(Debug, Default)]
pub struct MockEventRegistry {
    /// The events that have been recorded
    events: Mutex<Vec<MockEvent>>,
}

impl MockEventRegistry {
    /// Create a new mock event registry
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    /// Record an event
    pub fn record(&self, event: MockEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    /// Get all events
    pub fn events(&self) -> Vec<MockEvent> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    /// Get events for a specific mock
    pub fn events_for_mock<S: AsRef<str>>(&self, mock_name: S) -> Vec<MockEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.mock_name == mock_name.as_ref())
            .cloned()
            .collect()
    }

    /// Get events for a specific method
    pub fn events_for_method<S: AsRef<str>>(&self, method: S) -> Vec<MockEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.method == method.as_ref())
            .cloned()
            .collect()
    }

    /// Get events that match a predicate
    pub fn events_matching<F>(&self, predicate: F) -> Vec<MockEvent>
    where
        F: Fn(&MockEvent) -> bool,
    {
        let events = self.events.lock().unwrap();
        events.iter().filter(|e| predicate(e)).cloned().collect()
    }

    /// Clear all events
    pub fn clear(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }

    /// Get the number of events
    pub fn len(&self) -> usize {
        let events = self.events.lock().unwrap();
        events.len()
    }

    /// Check if there are no events
    pub fn is_empty(&self) -> bool {
        let events = self.events.lock().unwrap();
        events.is_empty()
    }
}

/// Utility for measuring the duration of a method call
pub struct TimedCall {
    /// The start time of the call
    start: Instant,

    /// The event to be recorded
    event: MockEvent,

    /// The registry to record the event in
    registry: Arc<MockEventRegistry>,
}

impl TimedCall {
    /// Create a new timed call
    pub fn new<S: Into<String>>(
        mock_name: S,
        method: S,
        registry: &Arc<MockEventRegistry>,
    ) -> Self {
        Self {
            start: Instant::now(),
            event: MockEvent::new(mock_name, method),
            registry: Arc::clone(registry),
        }
    }

    /// Add an argument to the call
    pub fn with_arg<S: Into<String>>(&mut self, arg: S) -> &mut Self {
        self.event = self.event.clone().with_arg(arg);
        self
    }

    /// Add multiple arguments to the call
    pub fn with_args<S: Into<String>>(&mut self, args: Vec<S>) -> &mut Self {
        self.event = self.event.clone().with_args(args);
        self
    }

    /// Complete the call successfully
    pub fn success(self) {
        let duration = self.start.elapsed();
        let event = self
            .event
            .with_result(MockEventResult::Success)
            .with_duration(duration);

        self.registry.record(event);
    }

    /// Complete the call with an error
    pub fn error<S: Into<String>>(self, error: S) {
        let duration = self.start.elapsed();
        let event = self
            .event
            .with_result(MockEventResult::Error(error.into()))
            .with_duration(duration);

        self.registry.record(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_mock_event() {
        let event = MockEvent::new("UserService", "get_user")
            .with_args(vec!["user_id", "1234"])
            .with_result(MockEventResult::Success)
            .with_duration(Duration::from_millis(42));

        assert_eq!(event.mock_name, "UserService");
        assert_eq!(event.method, "get_user");
        assert_eq!(event.args, vec!["user_id", "1234"]);
        assert_eq!(event.duration, Duration::from_millis(42));

        match event.result {
            MockEventResult::Success => {}
            _ => panic!("Expected success result"),
        }
    }

    #[test]
    fn test_mock_event_registry() {
        let registry = MockEventRegistry::new();

        let event1 = MockEvent::new("UserService", "get_user")
            .with_args(vec!["user_id", "1234"])
            .with_result(MockEventResult::Success);

        let event2 = MockEvent::new("UserService", "update_user")
            .with_args(vec!["user_id", "1234", "name", "New Name"])
            .with_result(MockEventResult::Success);

        let event3 = MockEvent::new("AuthService", "authenticate")
            .with_args(vec!["username", "password"])
            .with_result(MockEventResult::Error("Invalid credentials".to_string()));

        registry.record(event1);
        registry.record(event2);
        registry.record(event3);

        assert_eq!(registry.len(), 3);
        assert!(!registry.is_empty());

        let user_events = registry.events_for_mock("UserService");
        assert_eq!(user_events.len(), 2);

        let auth_events = registry.events_for_mock("AuthService");
        assert_eq!(auth_events.len(), 1);

        let get_user_events = registry.events_for_method("get_user");
        assert_eq!(get_user_events.len(), 1);

        let error_events =
            registry.events_matching(|e| matches!(e.result, MockEventResult::Error(_)));
        assert_eq!(error_events.len(), 1);

        registry.clear();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_timed_call() {
        let registry = Arc::new(MockEventRegistry::new());

        // Successful call
        {
            let mut call = TimedCall::new("UserService", "get_user", &registry);
            call.with_arg("user_id").with_arg("1234");

            thread::sleep(Duration::from_millis(10));
            call.success();
        }

        // Error call
        {
            let mut call = TimedCall::new("AuthService", "authenticate", &registry);
            call.with_args(vec!["username", "password"]);

            thread::sleep(Duration::from_millis(10));
            call.error("Invalid credentials");
        }

        assert_eq!(registry.len(), 2);

        let user_events = registry.events_for_mock("UserService");
        assert_eq!(user_events.len(), 1);
        assert!(user_events[0].duration >= Duration::from_millis(10));

        let auth_events = registry.events_for_mock("AuthService");
        assert_eq!(auth_events.len(), 1);
        assert!(auth_events[0].duration >= Duration::from_millis(10));

        match &auth_events[0].result {
            MockEventResult::Error(err) => assert_eq!(err, "Invalid credentials"),
            _ => panic!("Expected error result"),
        }
    }
}
