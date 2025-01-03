use std::borrow::Cow;

use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_resource::{
        BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BufferInitDescriptor,
        BufferUsages, CachedComputePipelineId, PipelineCache, ShaderStages,
    },
    renderer::RenderDevice,
    Extract, RenderApp, RenderSet,
};
use bytemuck::{Pod, Zeroable};

use super::*;

pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<BoidsPipeline>()
            .add_systems(ExtractSchedule, queue_bind_group.in_set(RenderSet::Queue));
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(BoidUniform { val: 10. })
            .add_plugins(ExtractResourcePlugin::<BoidUniform>::default());
    }
}

#[derive(Resource)]
pub struct BoidsPipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for BoidsPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(
            Some("Boids compute bind group layout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: bevy::render::render_resource::BindingType::Buffer {
                    ty: bevy::render::render_resource::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );
        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world.resource::<AssetServer>().load("shaders/boids.wgsl");

        let init_pipeline = pipeline_cache.queue_compute_pipeline(
            bevy::render::render_resource::ComputePipelineDescriptor {
                label: Some(Cow::Borrowed("Boids init pipeline")),
                layout: vec![bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("init"),
                zero_initialize_workgroup_memory: true,
            },
        );

        let update_pipeline = pipeline_cache.queue_compute_pipeline(
            bevy::render::render_resource::ComputePipelineDescriptor {
                label: Some(Cow::Borrowed("Boids update pipeline")),
                layout: vec![bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("update"),
                zero_initialize_workgroup_memory: true,
            },
        );

        BoidsPipeline {
            init_pipeline,
            update_pipeline,
            bind_group_layout,
        }
    }
}

#[derive(Resource)]
pub struct BoidValueBindGroup(pub BindGroup);

#[derive(ExtractResource, Resource, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoidUniform {
    val: f32,
}

pub fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<BoidsPipeline>,
    value: Res<BoidUniform>,
) {
    info!("Queue bind group!");
    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Boids buffer"),
        contents: bytemuck::cast_slice(&[value.val]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let bind_group = render_device.create_bind_group(
        Some("Value bind group"),
        &pipeline.bind_group_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    );
    commands.insert_resource(BoidValueBindGroup(bind_group));
}
