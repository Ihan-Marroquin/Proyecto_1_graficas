use raylib::prelude::*;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vector2,  
    pub a: f32,        
    pub fov: f32,  
    pub has_key: bool,
    pub health: f32,
    pub health_max: f32,
    pub shield: f32,
    pub shield_max: f32,
    pub stamina: f32,
    pub stamina_max: f32,
    pub binocular_timer: f32,
    pub step_timer: f32, 
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32, fov: f32) -> Self {
        Player {
            pos: Vector2::new(x, y),
            a: angle,
            fov,
            has_key: false,
            health: 100.0,
            health_max: 100.0,
            shield: 0.0,
            shield_max: 100.0,
            stamina: 100.0,
            stamina_max: 100.0,
            binocular_timer: 0.0,
            step_timer: 0.0,
        }
    }

    pub fn update_timers(&mut self, dt: f32) {
        if self.stamina < self.stamina_max {
            self.stamina = (self.stamina + 12.0 * dt).min(self.stamina_max);
        }
        if self.binocular_timer > 0.0 {
            self.binocular_timer = (self.binocular_timer - dt).max(0.0);
        }
        if self.step_timer > 0.0 {
            self.step_timer = (self.step_timer - dt).max(0.0);
        }
    }

    pub fn apply_damage(&mut self, amount: f32) {
        let mut dmg = amount;
        if self.shield > 0.0 {
            let absorb = self.shield.min(dmg);
            self.shield -= absorb;
            dmg -= absorb;
        }
        if dmg > 0.0 {
            self.health = (self.health - dmg).max(0.0);
        }
    }

    pub fn pickup_medkit(&mut self, amount: f32) {
        if self.health < self.health_max {
            self.health = (self.health + amount).min(self.health_max);
        } else {
            self.shield = (self.shield + amount).min(self.shield_max);
        }
    }

    pub fn pickup_key(&mut self) { self.has_key = true; }

    pub fn pickup_binoculars(&mut self, seconds: f32) {
        self.binocular_timer = self.binocular_timer.max(seconds);
    }
}
