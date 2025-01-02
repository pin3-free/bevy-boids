use super::*;

pub struct ObstaclesPlugin;

impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnObstacle>()
            .add_observer(spawn_obstacle);
    }
}

#[derive(Component)]
#[require(Collider, Transform, RigidBody, ViewVisibility, Mesh2d, MeshMaterial2d<ColorMaterial>)]
pub struct Obstacle;

pub enum ObstacleType {
    /// Circle defined by its width
    Circle(f32),
    /// Rectangle defined by its width and height
    Rectangle(f32, f32),
}

impl Default for ObstacleType {
    fn default() -> Self {
        Self::Circle(10.)
    }
}

#[derive(Event)]
pub struct SpawnObstacle {
    pos: Vec2,
    angle: f32,
    obstacle_type: ObstacleType,
    color: Color,
}

impl Default for SpawnObstacle {
    fn default() -> Self {
        Self {
            color: Color::hsl(0.3, 0.3, 0.3),
            pos: Default::default(),
            angle: Default::default(),
            obstacle_type: Default::default(),
        }
    }
}

impl SpawnObstacle {
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self {
            obstacle_type: ObstacleType::Rectangle(width, height),
            ..Default::default()
        }
    }

    pub fn circle(radius: f32) -> Self {
        Self {
            obstacle_type: ObstacleType::Circle(radius),
            ..Default::default()
        }
    }

    pub fn with_pos(self, pos: Vec2) -> Self {
        Self { pos, ..self }
    }

    pub fn with_angle(self, angle: f32) -> Self {
        Self { angle, ..self }
    }

    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

fn spawn_obstacle(
    trigger: Trigger<SpawnObstacle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let obstacle_material = materials.add(ColorMaterial::from_color(trigger.color));
    let (obstacle_shape, obstacle_collider) = match trigger.obstacle_type {
        ObstacleType::Circle(radius) => (meshes.add(Circle::new(radius)), Collider::circle(radius)),
        ObstacleType::Rectangle(width, height) => (
            meshes.add(Rectangle::new(width, height)),
            Collider::rectangle(width, height),
        ),
    };

    commands.spawn((
        Obstacle,
        Transform::from_translation(trigger.pos.extend(-1.))
            .with_rotation(Quat::from_rotation_z(trigger.angle)),
        MeshMaterial2d(obstacle_material),
        Mesh2d(obstacle_shape),
        obstacle_collider,
        RigidBody::Static,
        CollisionLayers::new(
            GameCollisionLayer::Obstacles,
            [GameCollisionLayer::VisionCones],
        ),
    ));
}
