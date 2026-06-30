fn main() {
    // only compile the resource file when targeting windows
    if cfg!(target_os = "windows") {
        embed_resource::compile("client.rc", embed_resource::NONE);
    }
}
