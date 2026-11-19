/// takes raw payload data (u32 - id, u32 - payload len, vec<u8> - data)
/// and splits it into each individual component in the form (id, data)
pub fn split_payloads(original: &Vec<u8>) -> Vec<(u32, Vec<u8>)> {
    let components: Vec<(u32, Vec<u8>)> = Vec::new();

    let mut byte_num = 0;
    while byte_num < original.len() {
        let mut id = 0;
        if let Some(id_slice) = original.get(1..5) {
            id = u32::from_le_bytes(id_slice);
        } else {
            return Vec::new();
        }
    }

    components
}

pub fn resolve_to_type(id: u32, data: &Vec<u8>) -> impl bevy::ecs::component::Component {
    bevy::prelude::Transform::default()
}
