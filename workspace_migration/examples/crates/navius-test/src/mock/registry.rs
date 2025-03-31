impl MockRegistry {
    /// Create a new mock registry
    pub fn new() -> Self {
        Self {
            mocks: Default::default(),
        }
    }

    /// Register a mock component
    pub fn register<T: ?Sized + 'static, M: 'static + Send + Sync>(
        &self,
        mock: Arc<M>,
    ) -> TestResult<()> {
        let type_id = TypeId::of::<T>();
        let boxed_mock = Box::new(mock);
        let dyn_mock = boxed_mock as Box<dyn Any + Send + Sync>;

        let mut mocks = self.mocks.write().unwrap();
        mocks.insert(type_id, dyn_mock);

        Ok(())
    }

    /// Register a mock component without the additional parameters
    pub fn register_mock<T: ?Sized + 'static>(&self, mock: Arc<impl Any + Send + Sync + 'static>) {
        let type_id = TypeId::of::<T>();
        let boxed_mock = Box::new(mock);
        let dyn_mock = boxed_mock as Box<dyn Any + Send + Sync>;

        let mut mocks = self.mocks.write().unwrap();
        mocks.insert(type_id, dyn_mock);
    }

    /// Get a mock component
    pub fn get<T: ?Sized + 'static>(&self) -> TestResult<Arc<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<T>();
        let mocks = self.mocks.read().unwrap();

        if let Some(mock) = mocks.get(&type_id) {
            // SAFETY: We're downcasting to the type we registered with
            let arc_any = mock.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
            Ok(arc_any.clone())
        } else {
            Err(TestError::missing_component(format!(
                "Mock for {:?} not found",
                type_id
            )))
        }
    }
}
