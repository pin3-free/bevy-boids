use super::*;

pub struct ConfigurationPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigurationSet;

impl Plugin for ConfigurationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationConfig::default())
            .insert_resource(VisionRadius::default())
            .insert_resource(MaxSpeed::default())
            .insert_resource(MaxForce::default())
            .register_type::<SimulationConfig>()
            .add_plugins(ResourceInspectorPlugin::<SimulationConfig>::default())
            .add_systems(
                FixedUpdate,
                (update_max_speed, update_max_force, update_vision_radius)
                    .run_if(resource_changed::<SimulationConfig>)
                    .in_set(ConfigurationSet),
            )
            .add_systems(
                FixedUpdate,
                update_vision_colliders
                    .run_if(resource_changed::<VisionRadius>)
                    .in_set(ConfigurationSet),
            );
    }
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct SimulationConfig {
    /// The maximum force that can be applied to a boid
    pub max_force: f32,
    /// The maximum speed of a boid
    pub max_speed: f32,
    /// The radius, at which the boid sees its flockmates
    pub vision_radius: f32,
    /// The weight of the separation instinct
    pub separation_strength: f32,
    /// The weight of the cohesion instinct
    pub cohesion_strength: f32,
    /// The weight of the alignment instinct
    pub alignment_strength: f32,
    /// The weight of the seek instinct
    pub seek_strength: f32,
    /// The number of rays that will be uniformly cast on _one side_ during
    /// the obstacle detection process
    pub obstacle_detection_density: i32,
    /// The radius at which the boid starts taking evasive action, expressed as
    /// a fraction of the vision radius
    pub obstacle_detection_radius_rel: f32,
    /// The weight of the obstacle avoidance instinct
    pub obstacle_avoidance_strength: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            max_force: 0.75,
            max_speed: 150.,
            vision_radius: 1.5,
            separation_strength: 1.05,
            cohesion_strength: 1.,
            alignment_strength: 0.2,
            seek_strength: 0.1,
            obstacle_detection_radius_rel: 0.5,
            obstacle_detection_density: 10,
            obstacle_avoidance_strength: 2.,
        }
    }
}

#[derive(Resource, Default)]
pub struct VisionRadius(pub f32);

#[derive(Resource, Default)]
pub struct MaxSpeed(pub f32);

#[derive(Resource, Default)]
pub struct MaxForce(pub f32);

fn update_max_speed(config: Res<SimulationConfig>, mut max_speed: ResMut<MaxSpeed>) {
    max_speed.0 = config.max_speed;
}

fn update_max_force(config: Res<SimulationConfig>, mut max_force: ResMut<MaxForce>) {
    max_force.0 = config.max_speed * config.max_force;
}

fn update_vision_radius(config: Res<SimulationConfig>, mut vision_radius: ResMut<VisionRadius>) {
    vision_radius.0 = config.max_speed * config.vision_radius;
}

fn update_vision_colliders(
    vision_radius: Res<VisionRadius>,
    q_cones: Populated<Entity, With<BoidVisionCone>>,
    mut commands: Commands,
) {
    for ent in q_cones.iter() {
        commands
            .entity(ent)
            .insert(Collider::circle(vision_radius.0));
    }
}
