use crate::sprite::Sprite;
use glam::Vec2;

#[derive(Debug)]
pub struct Paddle {
    pub speed: f32,
    pub wish_dir: Vec2,
    pub sprite: Sprite,
}

impl Paddle {
    pub fn new(sprite: Sprite, speed: f32) -> Self {
        Self {
            wish_dir: Vec2::ZERO,
            speed,
            sprite,
        }
    }

    pub fn update(&mut self, dt: f32, queue: &wgpu::Queue) {
        self.sprite.position += self.wish_dir * self.speed * dt;
        // Clamp the paddle to the screen
        self.sprite.position.y = self.sprite.position.y.min(0.9).max(-0.9);
        self.sprite.update(queue);
    }
}
