#[cfg(test)]
mod tests {
    use runtime::ecs::Mesh;

    fn assert_is_component<T: bevy_ecs::component::Component>() {}

    #[test]
    fn mesh_is_a_bevy_ecs_component() {
        assert_is_component::<Mesh>();
    }
}
