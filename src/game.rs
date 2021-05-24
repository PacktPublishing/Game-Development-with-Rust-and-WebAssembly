use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Game, KeyState, Point, Rect, Renderer},
};

const FLOOR: i16 = 475;
const IDLE_FRAMES: u8 = 29;
const RUNNING_FRAMES: u8 = 23;
const RUNNING_SPEED: i16 = 3;
const IDLE_FRAME_NAME: &str = "Idle";
const RUN_FRAME_NAME: &str = "Run";

struct RedHatBoy {
    state: RedHatBoyStateMachine,
    sprite_sheet: Sheet,
    image: HtmlImageElement,
}

impl RedHatBoy {
    fn new(sheet: Sheet, image: HtmlImageElement) -> Self {
        RedHatBoy {
            state: RedHatBoyStateMachine::Idle(RedHatBoyState::new()),
            sprite_sheet: sheet,
            image,
        }
    }

    fn run_right(&mut self) {
        self.state = self.state.run();
    }

    fn update(&mut self) {
        self.state = self.state.update();
    }

    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!(
            "{} ({}).png",
            self.state.state_name(),
            (self.state.game_object().frame / 3) + 1
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
                x: self.state.game_object().position.x.into(),
                y: self.state.game_object().position.y.into(),
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
}

impl RedHatBoyStateMachine {
    fn run(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(val) => RedHatBoyStateMachine::Running(val.into()),
            _ => self,
        }
    }

    fn state_name(&self) -> &str {
        match self {
            RedHatBoyStateMachine::Idle(_) => IDLE_FRAME_NAME,
            RedHatBoyStateMachine::Running(_) => RUN_FRAME_NAME,
        }
    }

    fn game_object(&self) -> &GameObject {
        match self {
            RedHatBoyStateMachine::Idle(state) => &state.game_object,
            RedHatBoyStateMachine::Running(state) => &state.game_object,
        }
    }

    fn update(self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(mut state) => {
                state.game_object = state.game_object.update(IDLE_FRAMES);
                RedHatBoyStateMachine::Idle(state)
            }
            RedHatBoyStateMachine::Running(mut state) => {
                state.game_object = state.game_object.update(RUNNING_FRAMES);
                RedHatBoyStateMachine::Running(state)
            }
        }
    }
}

#[derive(Copy, Clone)]
struct RedHatBoyState<S> {
    game_object: GameObject,
    _state: S,
}

#[derive(Copy, Clone)]
struct Idle;

impl RedHatBoyState<Idle> {
    fn new() -> Self {
        RedHatBoyState {
            game_object: GameObject {
                frame: 0,
                position: Point { x: 0, y: FLOOR },
                velocity: Point { x: 0, y: 0 },
            },
            _state: Idle {},
        }
    }
}

#[derive(Copy, Clone)]
struct Running;

impl From<RedHatBoyState<Idle>> for RedHatBoyState<Running> {
    fn from(mut machine: RedHatBoyState<Idle>) -> Self {
        machine.game_object = machine.game_object.reset_frame().run_right();
        RedHatBoyState {
            game_object: machine.game_object,
            _state: Running {},
        }
    }
}

#[derive(Copy, Clone)]
struct GameObject {
    frame: u8,
    position: Point,
    velocity: Point,
}

impl GameObject {
    fn update(mut self, frame_count: u8) -> Self {
        if self.frame < frame_count {
            self.frame += 1;
        } else {
            self.frame = 0;
        }

        self.position.x += self.velocity.x;
        self.position.y += self.velocity.y;
        self
    }

    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        self
    }

    fn run_right(mut self) -> Self {
        self.velocity.x += RUNNING_SPEED;
        self
    }
}

#[derive(Deserialize, Debug)]
struct SheetRect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize, Debug)]
struct Cell {
    frame: SheetRect,
}

#[derive(Deserialize, Debug)]
pub struct Sheet {
    frames: HashMap<String, Cell>,
}

pub struct WalkTheDog {
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { rhb: None }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&mut self) -> Result<Box<dyn Game>> {
        let json = browser::fetch_json("rhb.json").await?;

        let rhb = Some(RedHatBoy::new(
            json.into_serde::<Sheet>()?,
            engine::load_image("rhb.png").await?,
        ));

        Ok(Box::new(WalkTheDog { rhb }))
    }

    fn update(&mut self, keystate: &KeyState) {
        if keystate.is_pressed("ArrowRight") {
            self.rhb.as_mut().unwrap().run_right();
        }
        self.rhb.as_mut().unwrap().update();
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect {
            x: 0.0,
            y: 0.0,
            width: 600.0,
            height: 600.0,
        });

        self.rhb.as_ref().unwrap().draw(renderer);
    }
}
