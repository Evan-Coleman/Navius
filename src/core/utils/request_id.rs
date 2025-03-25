use uuid::Uuid;

pub fn get_req_id() -> String {
    Uuid::new_v4().to_string()
}
