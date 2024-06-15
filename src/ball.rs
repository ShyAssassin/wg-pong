use glam::Vec2;
use crate::sprite::Sprite;
use crate::paddle::Paddle;

pub struct Ball {
    pub speed: f32,
    pub velocity: Vec2,
    pub sprite: Sprite,
}

impl Ball {
    pub fn new(sprite: Sprite, speed: f32) -> Self {
        Self {
            speed,
            velocity: Vec2::ZERO,
            sprite,
        }
    }

    pub fn update(&mut self, dt: f32, queue: &wgpu::Queue, paddles: &[&Paddle]) {
        for paddle in paddles {
            if self.sprite.is_coliding(&paddle.sprite) {
                self.velocity.x = -self.velocity.x;
                self.velocity.y += paddle.wish_dir.length() * self.speed;
                self.sprite.position.x += self.velocity.x * dt;
            }
        }
        self.velocity = self.velocity.normalize() * self.speed;

        if self.sprite.position.y - self.sprite.size.y / 2.0 < -1.0 || self.sprite.position.y + self.sprite.size.y / 2.0 > 1.0 {
            self.velocity.y = -self.velocity.y;
        }

        self.sprite.position += self.velocity * dt;
        self.sprite.update(queue);
    }
}
