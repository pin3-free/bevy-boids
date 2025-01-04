use std::borrow::Cow;

use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    gpu_readback::{GpuReadbackPlugin, Readback, ReadbackComplete},
    graph::CameraDriverLabel,
    render_asset::RenderAssets,
    render_graph::{RenderGraph, RenderLabel},
    render_resource::{
        BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry, BufferUsages,
        CachedComputePipelineId, ComputePassDescriptor, PipelineCache, ShaderStages,
    },
    renderer::RenderDevice,
    storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
    Render, RenderApp, RenderSet,
};
use bytemuck::{Pod, Zeroable};

use super::*;

pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<BoidsPipeline>().add_systems(
            Render,
            prepare_bind_groups
                .in_set(RenderSet::PrepareBindGroups)
                .run_if(not(resource_exists::<BoidValueBindGroup>)),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        let label = BoidsNodeLabel("boids_compute");
        render_graph.add_node(label.clone(), BoidsNode);
        render_graph.add_node_edge(label, CameraDriverLabel);
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(BoidValue { val: 10. })
            // .add_systems(Update, print_value)
            .add_systems(Startup, shader_setup)
            .add_plugins((ExtractResourcePlugin::<ReadbackBuffer>::default(),));
    }
}

#[derive(Resource)]
pub struct BoidsPipeline {
    update_pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

fn _print_value(value: Res<BoidValue>) {
    info!("Value: {}", value.val);
}

impl FromWorld for BoidsPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(
            Some("Boids compute bind group layout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: bevy::render::render_resource::BindingType::Buffer {
                    ty: bevy::render::render_resource::BufferBindingType::Storage {
                        read_only: false,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        );
        let pipeline_cache = world.resource::<PipelineCache>();
        let shader = world.resource::<AssetServer>().load("shaders/boids.wgsl");

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
            update_pipeline,
            bind_group_layout,
        }
    }
}

#[derive(Resource)]
pub struct BoidValueBindGroup(pub BindGroup);

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackBuffer(pub Handle<ShaderStorageBuffer>);

#[derive(ExtractResource, Resource, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoidValue {
    val: f32,
}

pub fn shader_setup(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    value: Res<BoidValue>,
) {
    let buf = [value.val];
    let mut shader_buf = ShaderStorageBuffer::from(buf);
    shader_buf.buffer_description.usage |= BufferUsages::COPY_SRC;
    let buffer = buffers.add(shader_buf);

    commands.spawn(Readback::buffer(buffer.clone())).observe(
        |trigger: Trigger<ReadbackComplete>| {
            let data: Vec<f32> = trigger.event().to_shader_type();
            info!("Buffer after shader: {:?}", data);
        },
    );

    commands.insert_resource(ReadbackBuffer(buffer));
}

pub fn prepare_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<BoidsPipeline>,
    buffer: Res<ReadbackBuffer>,
    buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
) {
    let buffer = buffers.get(&buffer.0).expect("Found buffer");
    let bind_group = render_device.create_bind_group(
        Some("Value bind group"),
        &pipeline.bind_group_layout,
        &BindGroupEntries::sequential((buffer.buffer.as_entire_buffer_binding(),)),
    );
    commands.insert_resource(BoidValueBindGroup(bind_group));
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, RenderLabel)]
pub struct BoidsNodeLabel(&'static str);

pub struct BoidsNode;

impl bevy::render::render_graph::Node for BoidsNode {
    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let bind_group = &world.resource::<BoidValueBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<BoidsPipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, bind_group, &[]);
        let upd_pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.update_pipeline)
            .expect("A pipeline???");
        pass.set_pipeline(&upd_pipeline);
        pass.dispatch_workgroups(8, 8, 1);

        Ok(())
    }
}
