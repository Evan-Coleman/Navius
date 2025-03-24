#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode},
        response::Response,
    };
    use chrono::Utc;
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        app::{
            api::pet_db::{CreatePetRequest, UpdatePetRequest, configure},
            database::repositories::pet_repository::{Pet, PetRepository},
            services::{
                ServiceRegistry,
                pet_service::{CreatePetDto, DefaultPetService, PetService, UpdatePetDto},
            },
        },
        core::{
            database::{MockDatabaseConnection, PgPool, generate_uuid},
            error::AppError,
            router::AppState,
        },
    };

    // Mock implementation of PetService for testing
    struct MockPetService {
        // Control what the service returns for each method
        get_all_result: Result<Vec<Pet>, AppError>,
        get_by_id_result: Result<Option<Pet>, AppError>,
        create_result: Result<Pet, AppError>,
        update_result: Result<Pet, AppError>,
        delete_result: Result<bool, AppError>,
    }

    impl MockPetService {
        fn new() -> Self {
            // Default mock with successful responses
            let now = Utc::now();
            let pet = Pet {
                id: generate_uuid(),
                name: "Test Pet".to_string(),
                species: "Test Species".to_string(),
                age: 5,
                created_at: now,
                updated_at: now,
            };

            Self {
                get_all_result: Ok(vec![pet.clone()]),
                get_by_id_result: Ok(Some(pet.clone())),
                create_result: Ok(pet.clone()),
                update_result: Ok(pet.clone()),
                delete_result: Ok(true),
            }
        }

        // Helper to configure mock to return an error
        fn with_error(error: AppError) -> Self {
            let mut mock = Self::new();
            mock.get_all_result = Err(error.clone());
            mock.get_by_id_result = Err(error.clone());
            mock.create_result = Err(error.clone());
            mock.update_result = Err(error.clone());
            mock.delete_result = Err(error);
            mock
        }

        // Helper to configure mock to return not found
        fn with_not_found() -> Self {
            let mut mock = Self::new();
            mock.get_by_id_result = Ok(None);
            mock.delete_result = Ok(false);
            mock
        }
    }

    #[async_trait::async_trait]
    impl PetService for MockPetService {
        async fn get_all_pets(&self) -> Result<Vec<Pet>, AppError> {
            self.get_all_result.clone()
        }

        async fn get_pet_by_id(&self, _id: Uuid) -> Result<Option<Pet>, AppError> {
            self.get_by_id_result.clone()
        }

        async fn create_pet(&self, _pet_data: CreatePetDto) -> Result<Pet, AppError> {
            self.create_result.clone()
        }

        async fn update_pet(&self, _id: Uuid, _pet_data: UpdatePetDto) -> Result<Pet, AppError> {
            self.update_result.clone()
        }

        async fn delete_pet(&self, _id: Uuid) -> Result<bool, AppError> {
            self.delete_result.clone()
        }

        async fn find_pets_by_species(&self, _species: &str) -> Result<Vec<Pet>, AppError> {
            self.get_all_result.clone()
        }
    }

    // Mock implementation of ServiceRegistry for testing
    struct MockServiceRegistry {
        pet_service: Box<dyn PetService>,
    }

    impl MockServiceRegistry {
        fn new(pet_service: Box<dyn PetService>) -> Self {
            Self { pet_service }
        }
    }

    impl ServiceRegistry for MockServiceRegistry {
        fn pet_service(&self) -> &dyn PetService {
            self.pet_service.as_ref()
        }
    }

    // Helper to create a test AppState with the mock services
    fn create_test_state(pet_service: Box<dyn PetService>) -> Arc<AppState> {
        let service_registry =
            Arc::new(MockServiceRegistry::new(pet_service)) as Arc<dyn ServiceRegistry>;

        Arc::new(AppState {
            client: reqwest::Client::new(),
            config: Default::default(),
            start_time: std::time::SystemTime::now(),
            cache_registry: None,
            metrics_handle: metrics_exporter_prometheus::PrometheusBuilder::new()
                .build_recorder()
                .handle(),
            token_client: None,
            resource_registry: crate::utils::api_resource::ApiResourceRegistry::new(),
            db_pool: None,
            service_registry: Some(service_registry),
        })
    }

    // Helper to make a test request to an endpoint
    async fn make_request(
        state: Arc<AppState>,
        method: &str,
        uri: &str,
        body: Option<Value>,
    ) -> Response {
        let app = configure().with_state(state);

        let mut req_builder = Request::builder()
            .uri(uri)
            .method(method)
            .header("Content-Type", "application/json");

        let body = match body {
            Some(json_body) => Body::from(json_body.to_string()),
            None => Body::empty(),
        };

        let request = req_builder.body(body).unwrap();
        app.oneshot(request).await.unwrap()
    }

    // Helper to read response body into JSON
    async fn read_body_json(response: Response) -> Value {
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn test_get_all_pets_success() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::new()));

        // Act
        let response = make_request(state, "GET", "/petdb", None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body = read_body_json(response).await;
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0]["name"], "Test Pet");
        assert_eq!(body[0]["species"], "Test Species");
    }

    #[tokio::test]
    async fn test_get_all_pets_error() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::with_error(
            AppError::DatabaseError("Database error".to_string()),
        )));

        // Act
        let response = make_request(state, "GET", "/petdb", None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_get_pet_by_id_success() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::new()));
        let uuid = Uuid::new_v4();

        // Act
        let response = make_request(state, "GET", &format!("/petdb/{}", uuid), None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body = read_body_json(response).await;
        assert_eq!(body["name"], "Test Pet");
        assert_eq!(body["species"], "Test Species");
    }

    #[tokio::test]
    async fn test_get_pet_by_id_not_found() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::with_not_found()));
        let uuid = Uuid::new_v4();

        // Act
        let response = make_request(state, "GET", &format!("/petdb/{}", uuid), None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_pet_by_id_invalid_uuid() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::new()));

        // Act - Use an invalid UUID
        let response = make_request(state, "GET", "/petdb/not-a-uuid", None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_pet_success() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::new()));
        let request_body = json!({
            "name": "New Pet",
            "species": "Dog",
            "age": 3
        });

        // Act
        let response = make_request(state, "POST", "/petdb", Some(request_body)).await;

        // Assert
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = read_body_json(response).await;
        assert_eq!(body["name"], "Test Pet"); // MockPetService returns a test pet
    }

    #[tokio::test]
    async fn test_update_pet_success() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::new()));
        let uuid = Uuid::new_v4();
        let request_body = json!({
            "name": "Updated Pet",
            "species": "Cat"
        });

        // Act
        let response = make_request(
            state,
            "PUT",
            &format!("/petdb/{}", uuid),
            Some(request_body),
        )
        .await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let body = read_body_json(response).await;
        assert_eq!(body["name"], "Test Pet"); // MockPetService returns a test pet
    }

    #[tokio::test]
    async fn test_delete_pet_success() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::new()));
        let uuid = Uuid::new_v4();

        // Act
        let response = make_request(state, "DELETE", &format!("/petdb/{}", uuid), None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_pet_not_found() {
        // Arrange
        let state = create_test_state(Box::new(MockPetService::with_not_found()));
        let uuid = Uuid::new_v4();

        // Act
        let response = make_request(state, "DELETE", &format!("/petdb/{}", uuid), None).await;

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
