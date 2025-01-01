# Boids in bevy, briefly.

This repo is a demonstration of implementing a boids simulation using some modern (as of 0.15) bevy concepts such as observers, required components, as well as some good practices
such as system sets, generic systems, etc.

## What are boids?

Boids is a simulation of flocking behaviors found in birds, fish, and herd animals, first described by Craig W. Reynolds in 1987.
It uses three simple principals that a flocking entity (boid) can use to create emerging complex behaviors resembling a school of fish/flock of birds. The principles are:

1. Separation: Avoid bumping into your flockmates
2. Alignment: Try to go the same-ish way as your flockmates
3. Cohesion: try to stick with the group

This creates a seemingly complex behavior that emerges from many boids following these rules with the information they're given. Each boid is limited by it's vision range.

## Where can I find more information?

This repo aims to reproduce behaviors listed in this paper: [Steering Behaviors For Autonomous Characters]([url](https://www.red3d.com/cwr/steer/gdc99/)), which covers
more behaviors, such as seeking out a target, obstacle avoidance, roaming, etc.

The boids algorithm itself was fist described here: [Flocks, Herds, and Schools: A Distributed Behavioral Model](https://www.cs.toronto.edu/~dt/siggraph97-course/cwr87/)

## How do I run this?

After setting up a basic rust environment (mainly `cargo`), use `cargo run --release` to watch the simulation unfold. You can control the simulation with the 
settings floating menu, and use Shift + click to spawn a target for the boids to follow around. You can also drag the target.
