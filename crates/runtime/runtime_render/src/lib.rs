use bevy::{
    anti_alias::AntiAliasPlugin,
    app::{PluginGroup, PluginGroupBuilder},
    camera::CameraPlugin,
    core_pipeline::CorePipelinePlugin,
    gizmos::GizmoPlugin,
    gizmos_render::GizmoRenderPlugin,
    gltf::GltfPlugin,
    image::ImagePlugin,
    light::LightPlugin,
    mesh::MeshPlugin,
    pbr::PbrPlugin,
    post_process::PostProcessPlugin,
    render::RenderPlugin,
    render::pipelined_rendering::PipelinedRenderingPlugin,
    sprite::SpritePlugin,
    sprite_render::SpriteRenderPlugin,
};

/// Must be added after `RuntimeAssetPlugins` and `RuntimeWindowPlugins`.
pub struct RuntimeRenderPlugins;

impl PluginGroup for RuntimeRenderPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RenderPlugin::default())
            // after RenderPlugin so it knows the supported compressed texture formats
            .add(ImagePlugin::default())
            .add(MeshPlugin)
            .add(CameraPlugin)
            .add(LightPlugin)
            .add(PipelinedRenderingPlugin)
            .add(CorePipelinePlugin)
            .add(PostProcessPlugin)
            .add(AntiAliasPlugin)
            .add(SpritePlugin)
            .add(SpriteRenderPlugin)
            // after RenderPlugin (compressed texture support) and before PbrPlugin (gltf material handler)
            .add(GltfPlugin::default())
            .add(PbrPlugin::default())
            .add(GizmoPlugin)
            .add(GizmoRenderPlugin)
    }
}
