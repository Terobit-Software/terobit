#[cfg(test)]
mod tests {
    use runtime::consts::shader::ROOT_SHADER_PATH;

    #[test]
    fn root_shader_path_points_at_root_wgsl() {
        assert_eq!(ROOT_SHADER_PATH, ["runtime", "shaders", "root.wgsl"]);
    }
}
