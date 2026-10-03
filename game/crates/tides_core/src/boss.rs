//! 黑腕队长·格罗姆's brain: chase → telegraphed wind-up → lunge or slam →
//! recover. At half HP he enters phase 2: faster, and calls two dwarves.
//! Engine-free: it takes the distance to the player and returns intents.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Chase,
    WindupLunge,
    Lunge,
    WindupSlam,
    Recover,
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Move {
    Stop,
    /// Walk toward the player at this speed.
    Chase(f32),
    /// Dash along the direction locked at the end of the wind-up.
    Lunge(f32),
}

/// What the body should do this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intent {
    pub movement: Move,
    /// Flash to warn the player (true during wind-ups).
    pub telegraph: bool,
    /// Lock the lunge direction toward the player now.
    pub aim_lunge: bool,
    /// Damage to deal if the player is within `radius` this tick.
    pub strike: Option<Strike>,
    pub summon: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strike {
    pub damage: f32,
    pub radius: f32,
    /// Slams make a shockwave; lunges hit on contact.
    pub shockwave: bool,
}

pub const MAX_HP: f32 = 160.0;
const MOVE_SPEED: f32 = 56.0;
const LUNGE_SPEED: f32 = 380.0;
const LUNGE_DAMAGE: f32 = 14.0;
const LUNGE_REACH: f32 = 44.0;
const SLAM_DAMAGE: f32 = 18.0;
pub const SLAM_RADIUS: f32 = 130.0;
const SLAM_TRIGGER: f32 = 80.0;

#[derive(Clone, Debug)]
pub struct Grom {
    pub hp: f32,
    pub phase: u8,
    pub state: State,
    t: f32,
    attack_cd: f32,
    /// A lunge only lands once per dash.
    lunge_hit: bool,
}

impl Default for Grom {
    fn default() -> Self {
        Self { hp: MAX_HP, phase: 1, state: State::Chase, t: 0.0, attack_cd: 1.2, lunge_hit: false }
    }
}

impl Grom {
    pub fn is_dead(&self) -> bool {
        self.state == State::Dead
    }

    /// Returns true on the hit that kills him.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.is_dead() {
            return false;
        }
        self.hp = (self.hp - amount).max(0.0);
        if self.hp <= 0.0 {
            self.state = State::Dead;
            return true;
        }
        false
    }

    fn enter(&mut self, s: State, t: f32) {
        self.state = s;
        self.t = t;
    }

    fn fast(&self) -> bool {
        self.phase == 2
    }

    pub fn tick(&mut self, dt: f32, dist: f32) -> Intent {
        let mut out = Intent { movement: Move::Stop, telegraph: false, aim_lunge: false, strike: None, summon: false };
        self.t -= dt;
        match self.state {
            State::Dead => {}
            State::Chase => self.chase(dt, dist, &mut out),
            State::WindupLunge => {
                out.telegraph = true;
                if self.t <= 0.0 {
                    out.aim_lunge = true;
                    self.lunge_hit = false;
                    self.enter(State::Lunge, 0.3);
                }
            }
            State::Lunge => self.lunge(dist, &mut out),
            State::WindupSlam => {
                out.telegraph = true;
                if self.t <= 0.0 {
                    out.strike = Some(Strike { damage: SLAM_DAMAGE, radius: SLAM_RADIUS, shockwave: true });
                    self.attack_cd = if self.fast() { 1.6 } else { 2.1 };
                    self.enter(State::Recover, 0.6);
                }
            }
            State::Recover => {
                if self.t <= 0.0 {
                    self.enter(State::Chase, 0.0);
                }
            }
        }
        out
    }

    fn chase(&mut self, dt: f32, dist: f32, out: &mut Intent) {
        self.attack_cd -= dt;
        let speed = if self.fast() { MOVE_SPEED * 1.45 } else { MOVE_SPEED };
        out.movement = Move::Chase(speed);
        if self.phase == 1 && self.hp <= MAX_HP * 0.5 {
            self.phase = 2;
            out.summon = true;
        }
        if self.attack_cd <= 0.0 {
            if dist < SLAM_TRIGGER {
                self.enter(State::WindupSlam, 0.55);
            } else {
                self.enter(State::WindupLunge, 0.5);
            }
        }
    }

    fn lunge(&mut self, dist: f32, out: &mut Intent) {
        out.movement = Move::Lunge(LUNGE_SPEED);
        if !self.lunge_hit && dist < LUNGE_REACH {
            self.lunge_hit = true;
            out.strike = Some(Strike { damage: LUNGE_DAMAGE, radius: LUNGE_REACH, shockwave: false });
        }
        if self.t <= 0.0 {
            self.attack_cd = if self.fast() { 1.5 } else { 1.9 };
            self.enter(State::Recover, 0.45);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_until(g: &mut Grom, dist: f32, pred: impl Fn(&Intent) -> bool) -> Option<Intent> {
        (0..600).map(|_| g.tick(1.0 / 60.0, dist)).find(|i| pred(i))
    }

    #[test]
    fn far_player_gets_telegraphed_lunge() {
        let mut g = Grom::default();
        assert!(run_until(&mut g, 300.0, |i| i.telegraph).is_some());
        assert_eq!(g.state, State::WindupLunge);
        assert!(run_until(&mut g, 300.0, |i| i.aim_lunge).is_some());
    }

    #[test]
    fn close_player_gets_slammed() {
        let mut g = Grom::default();
        let hit = run_until(&mut g, 40.0, |i| i.strike.is_some()).unwrap();
        assert!(hit.strike.unwrap().shockwave);
    }

    #[test]
    fn lunge_hits_once_per_dash() {
        let mut g = Grom::default();
        run_until(&mut g, 300.0, |i| i.aim_lunge);
        let hits = (0..20).filter(|_| g.tick(1.0 / 60.0, 10.0).strike.is_some()).count();
        assert_eq!(hits, 1);
    }

    #[test]
    fn phase_two_summons_once() {
        let mut g = Grom::default();
        g.take_damage(MAX_HP * 0.6);
        let summons = (0..600).filter(|_| g.tick(1.0 / 60.0, 300.0).summon).count();
        assert_eq!(summons, 1);
        assert_eq!(g.phase, 2);
    }

    #[test]
    fn dies_once() {
        let mut g = Grom::default();
        assert!(!g.take_damage(100.0));
        assert!(g.take_damage(100.0));
        assert!(!g.take_damage(100.0));
        assert_eq!(g.tick(0.1, 10.0).movement, Move::Stop);
    }
}
