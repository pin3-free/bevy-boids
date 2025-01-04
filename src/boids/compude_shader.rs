use std::borrow::Cow;

use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    gpu_readback::{Readback, ReadbackComplete},
    graph::CameraDriverLabel,
    render_asset::RenderAssets,
    render_graph::{RenderGraph, RenderLabel},
    render_resource::{
        binding_types::storage_buffer, BindGroup, BindGroupEntries, BindGroupLayout,
        BindGroupLayoutEntries, BufferUsages, CachedComputePipelineId, ComputePassDescriptor,
        PipelineCache, ShaderStages, ShaderType,
    },
    renderer::RenderDevice,
    storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
    Render, RenderApp, RenderSet,
};

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
        app
            // .add_systems(Update, print_value)
            .add_systems(Startup, shader_setup)
            .add_systems(Update, sync_positions)
            .add_plugins((ExtractResourcePlugin::<ReadbackBuffer>::default(),));
    }
}

#[derive(Resource)]
pub struct BoidsPipeline {
    update_pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for BoidsPipeline {
    fn from_world(world: &mut World) -> Self {
        let bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(
            Some("Boids compute bind group layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (storage_buffer::<Vec<BoidInfo>>(false),),
            ),
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

#[derive(ShaderType, Debug, Clone, Copy)]
pub struct BoidInfo {
    position: Vec2,
}

#[derive(Resource)]
pub struct BoidValueBindGroup(pub BindGroup);

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackBuffer(pub Handle<ShaderStorageBuffer>);

fn sync_positions(
    q_boids: Query<&Transform, With<Boid>>,
    buffer: Res<ReadbackBuffer>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let positions = q_boids
        .iter()
        .map(|t| BoidInfo {
            position: t.translation.xy(),
        })
        .collect::<Vec<_>>();
    let buf = buffers.get_mut(&buffer.0).expect("Buffer");
    buf.set_data(positions);
}

fn print_readback(trigger: Trigger<ReadbackComplete>) {
    let data: Vec<BoidInfo> = trigger.event().to_shader_type();
    info!("Buffer after shader: {:?}", data);
}

pub fn shader_setup(mut commands: Commands, mut buffers: ResMut<Assets<ShaderStorageBuffer>>) {
    let mut shader_buf = ShaderStorageBuffer::from(Vec::<BoidInfo>::new());
    shader_buf.buffer_description.usage |= BufferUsages::COPY_SRC;
    let buffer = buffers.add(shader_buf);

    commands
        .spawn(Readback::buffer(buffer.clone()))
        .observe(print_readback);

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
