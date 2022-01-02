use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::{collections::HashMap, marker::PhantomData};

use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Game, KeyState, Point, Rect, Renderer},
};

const FLOOR: i16 = 475;
const IDLE_FRAMES: u8 = 29;
const RUNNING_FRAMES: u8 = 23;
const JUMPING_FRAMES: u8 = 35;
const SLIDING_FRAMES: u8 = 14;
const RUNNING_SPEED: i16 = 3;
const IDLE_FRAME_NAME: &str = "Idle";
const RUN_FRAME_NAME: &str = "Run";
const SLIDING_FRAME_NAME: &str = "Slide";
const JUMPING_FRAME_NAME: &str = "Jump";
const JUMP_SPEED: i16 = -25;
const GRAVITY: i16 = 1;

// Things to try:
// Can I use Generics to get rid of the 'match every state' functions (hooray!)
// - You have to start by putting a trait on T that states implement
//
// Can I use PhantomData on the structs so that no memory is used.
// Can I get rid of match { Type, _ => error } - That's a big goal. That plus boilerplate
// Should I use a Box to get rid of unnecessary copies
// Can I get rid of the enum noise via a generic game struct? That's the winner
// If I can do that, I can do it on RedHatBoy
// - That would do it provided everything is generic and you constantly copy (or for efficiency transmute)
//
//
//
//

trait State {
    fn update(&self);
}

pub struct RedHatBoy {
    state_machine: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    fn new(sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state_machine: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet: sheet,
            image,
        }
    }

    fn run_right(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Run);
    }

    fn slide(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Slide);
    }

    fn jump(&mut self) {
        self.state_machine = self.state_machine.transition(Event::Jump);
    }

    fn update(&mut self) {
        self.state_machine = self.state_machine.update();
    }

    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!(
            "{} ({}).png",
            self.state_machine.state_name(),
            (self.state_machine.context().frame / 3) + 1
        );

        let sprite = self
            .sprite_sheet
            .frames
            .get(&frame_name)
            .expect("Cell not found");

        renderer.draw_image(
            &self.image,
            &Rect {
                x: sprite.frame.x.into(),
                y: sprite.frame.y.into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
            },
            &Rect {
                x: self.state_machine.context().position.x.into(),
                y: self.state_machine.context().position.y.into(),
                width: sprite.frame.w.into(),
                height: sprite.frame.h.into(),
            },
        );
    }
}

#[derive(Copy, Clone)]
enum RedHatBoyStateMachine {
    Idle(RedHatBoyState<Idle>),
    Running(RedHatBoyState<Running>),
    Sliding(RedHatBoyState<Sliding>),
    Jumping(RedHatBoyState<Jumping>),
}

enum Event {
    Run,
    Jump,
    Slide,
    Land,
    Stand,
}

impl RedHatBoyStateMachine {
    fn transition(self, event: Event) -> Self {
        match (self, event) {
            (RedHatBoyStateMachine::Idle(state), Event::Run) => {
                RedHatBoyStateMachine::Running(state.run())
            }

            (RedHatBoyStateMachine::Running(state), Event::Jump) => {
                RedHatBoyStateMachine::Jumping(state.jump())
            }

            (RedHatBoyStateMachine::Running(state), Event::Slide) => {
                RedHatBoyStateMachine::Sliding(state.slide())
            }

            (RedHatBoyStateMachine::Jumping(state), Event::Land) => {
                RedHatBoyStateMachine::Running(state.land())
            }

            (RedHatBoyStateMachine::Sliding(state), Event::Stand) => {
                RedHatBoyStateMachine::Running(state.stand())
            }

            (_, _) => self,
        }
    }

    fn state_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(_) => IDLE_FRAME_NAME,
            RedHatBoyStateMachine::Running(_) => RUN_FRAME_NAME,
            RedHatBoyStateMachine::Jumping(_) => JUMPING_FRAME_NAME,
            RedHatBoyStateMachine::Sliding(_) => SLIDING_FRAME_NAME,
        }
    }

    fn context(&self) -> &RedHatBoyContext {
        match self {
            RedHatBoyStateMachine::Idle(state) => &state.context,
            RedHatBoyStateMachine::Running(state) => &state.context,
            RedHatBoyStateMachine::Jumping(state) => &state.context,
            RedHatBoyStateMachine::Sliding(state) => &state.context,
        }
    }

    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => match state.update() {
                None => RedHatBoyStateMachine::Idle(state),
                Some(event) => RedHatBoyStateMachine::Idle(state).transition(event),
            },
            RedHatBoyStateMachine::Running(mut state) => match state.update() {
                None => RedHatBoyStateMachine::Running(state),
                Some(event) => RedHatBoyStateMachine::Running(state).transition(event),
            },
            RedHatBoyStateMachine::Jumping(mut state) => match state.update() {
                None => RedHatBoyStateMachine::Jumping(state),
                Some(event) => RedHatBoyStateMachine::Jumping(state).transition(event),
            },
            RedHatBoyStateMachine::Sliding(mut state) => match state.update() {
                None => RedHatBoyStateMachine::Sliding(state),
                Some(event) => RedHatBoyStateMachine::Sliding(state).transition(event),
            },
        }
    }
}

#[derive(Copy, Clone)]
struct RedHatBoyState<S> {
    context: RedHatBoyContext,
    _state: S,
}

impl<S> RedHatBoyState<S> {
    fn update_context(&mut self, frames: u8) {
        self.context = self.context.update(frames);
    }
}

#[derive(Copy, Clone)]
struct Idle;

impl RedHatBoyState<Idle> {
    fn new() -> Self {
        RedHatBoyState {
            context: RedHatBoyContext {
                frame: 0,
                position: Point { x: 0, y: FLOOR },
                velocity: Point { x: 0, y: 0 },
            },
            _state: Idle {},
        }
    }

    fn update(&mut self) -> Option<Event> {
        self.update_context(IDLE_FRAMES);
        None
    }

    fn run(mut self) -> RedHatBoyState<Running> {
        self.context = self.context.reset_frame().run_right();
        RedHatBoyState {
            context: self.context,
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
struct Running;

impl RedHatBoyState<Running> {
    fn update(&mut self) -> Option<Event> {
        self.update_context(RUNNING_FRAMES);
        None
    }

    fn jump(mut self) -> RedHatBoyState<Jumping> {
        self.context = self.context.reset_frame().set_vertical_velocity(JUMP_SPEED);
        RedHatBoyState {
            context: self.context,
            _state: Jumping {},
        }
    }

    fn slide(mut self) -> RedHatBoyState<Sliding> {
        self.context = self.context.reset_frame();
        RedHatBoyState {
            context: self.context,
            _state: Sliding {},
        }
    }
}

#[derive(Copy, Clone)]
struct Jumping;

impl RedHatBoyState<Jumping> {
    fn update(&mut self) -> Option<Event> {
        self.update_context(JUMPING_FRAMES);

        if self.context.position.y >= FLOOR {
            Some(Event::Land)
        } else {
            None
        }
    }

    fn land(mut self) -> RedHatBoyState<Running> {
        self.context = self.context.reset_frame();
        RedHatBoyState {
            context: self.context,
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
struct Sliding;

impl RedHatBoyState<Sliding> {
    fn update(&mut self) -> Option<Event> {
        self.update_context(SLIDING_FRAMES);

        if self.context.frame >= SLIDING_FRAMES {
            Some(Event::Stand)
        } else {
            None
        }
    }

    fn stand(mut self) -> RedHatBoyState<Running> {
        self.context = self.context.reset_frame();
        RedHatBoyState {
            context: self.context,
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
struct RedHatBoyContext {
    frame: u8,
    position: Point,
    velocity: Point,
}

impl RedHatBoyContext {
    fn update(mut self, frame_count: u8) -> Self {
        self.velocity.y += GRAVITY;

        if self.frame < frame_count {
            self.frame += 1;
        } else {
            self.frame = 0;
        }

        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;

        if self.position.y > FLOOR {
            self.position.y = FLOOR;
        }

        self
    }

    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        self
    }

    fn set_vertical_velocity(mut self, y: i16) -> Self {
        self.velocity.y = y;
        self
    }

    fn run_right(mut self) -> Self {
        self.velocity.x += RUNNING_SPEED;
        self
    }
}

#[derive(Deserialize, Debug)]
struct SheetRect {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

#[derive(Deserialize, Debug)]
struct Cell {
    frame: SheetRect,
}

#[derive(Deserialize, Debug)]
pub struct Sheet {
    frames: HashMap<String, Cell>,
}

pub enum WalkTheDog {
    Loading,
    Loaded(RedHatBoy),
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog::Loading {}
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self {
            WalkTheDog::Loading => {
                let sheet = browser::fetch_json("rhb.json").await?.into_serde()?;

                let rhb = RedHatBoy::new(sheet, engine::load_image("rhb.png").await?);
                Ok(Box::new(WalkTheDog::Loaded(rhb)))
            }
            WalkTheDog::Loaded(_) => Err(anyhow!("Error: Game is already initialized")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let WalkTheDog::Loaded(rhb) = self {
            if keystate.is_pressed("ArrowRight") {
                rhb.run_right();
            }

            if keystate.is_pressed("Space") {
                rhb.jump();
            }

            if keystate.is_pressed("ArrowDown") {
                rhb.slide();
            }

            rhb.update();
        }
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        });

        if let WalkTheDog::Loaded(rhb) = self {
            rhb.draw(renderer);
        }
    }
}
