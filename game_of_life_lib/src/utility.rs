use uuid::Uuid;

pub fn generate_id() -> String {
    let mut buf = Uuid::encode_buffer();
    let id = Uuid::new_v4().to_simple().encode_upper(&mut buf);

    String::from(id)
}
