use std::borrow::Cow;

use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    graph::CameraDriverLabel,
    render_graph::{RenderGraph, RenderLabel},
    render_resource::{
        BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BufferInitDescriptor,
        BufferUsages, CachedComputePipelineId, ComputePassDescriptor, PipelineCache, ShaderStages,
    },
    renderer::RenderDevice,
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
            queue_bind_group.in_set(RenderSet::PrepareBindGroups),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        let label = BoidsNodeLabel("boids_compute");
        render_graph.add_node(label.clone(), BoidsNode);
        render_graph.add_node_edge(label, CameraDriverLabel);
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(BoidValue { val: 10. })
            .add_systems(Update, print_value)
            .add_plugins(ExtractResourcePlugin::<BoidValue>::default());
    }
}

#[derive(Resource)]
pub struct BoidsPipeline {
    update_pipeline: CachedComputePipelineId,
    bind_group_layout: BindGroupLayout,
}

fn print_value(value: Res<BoidValue>) {
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

#[derive(ExtractResource, Resource, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct BoidValue {
    val: f32,
}

pub fn queue_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<BoidsPipeline>,
    value: Res<BoidValue>,
) {
    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Boids buffer"),
        contents: bytemuck::cast_slice(&[value.val]),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
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
