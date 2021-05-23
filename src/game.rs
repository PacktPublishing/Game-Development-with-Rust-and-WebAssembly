use anyhow::Result;
use async_trait::async_trait;
use web_sys::HtmlImageElement;

use crate::{
    browser,
    engine::{self, Game, KeyState, Point, Rect, Renderer, Sheet},
};

const FLOOR: i16 = 475;
const IDLE_FRAMES: u8 = 29;
const RUNNING_FRAMES: u8 = 23;
const RUNNING_SPEED: i16 = 3;

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

    fn update(&mut self) {
        self.state = self.state.update();
    }

    fn draw(&self, renderer: &Renderer) {
        let frame_name = format!("Idle ({}).png", (self.state.game_object().frame / 3) + 1);

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
    fn run(mut self) -> Self {
        match self {
            RedHatBoyStateMachine::Idle(val) => RedHatBoyStateMachine::Running(val.into()),
            _ => self,
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
        machine.game_object = machine.game_object.reset_frame().start_running();
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
        self
    }

    fn reset_frame(mut self) -> Self {
        self.frame = 0;
        self
    }

    fn start_running(mut self) -> Self {
        self.velocity.x = RUNNING_SPEED;
        self
    }
}

pub struct WalkTheDog {
    image: Option<HtmlImageElement>,
    sheet: Option<Sheet>,
    frame: u8,
    position: Point,
    rhb: Option<RedHatBoy>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog {
            image: None,
            sheet: None,
            frame: 0,
            position: Point { x: 0, y: 0 },
            rhb: None,
        }
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        let json = browser::fetch_json("rhb.json").await?;
        let sheet = json.into_serde()?;
        let image = Some(engine::load_image("rhb.png").await?);

        let rhb = Some(RedHatBoy::new(
            json.into_serde::<Sheet>()?,
            engine::load_image("rhb.png").await?,
        ));

        Ok(Box::new(WalkTheDog {
            image,
            sheet,
            rhb,
            frame: self.frame,
            position: self.position,
        }))
    }

    fn update(&mut self, keystate: &KeyState) {
        let mut velocity = Point { x: 0, y: 0 };
        if keystate.is_pressed("ArrowDown") {
            velocity.y += 3;
        }

        if keystate.is_pressed("ArrowUp") {
            velocity.y -= 3;
        }

        if keystate.is_pressed("ArrowRight") {
            velocity.x += 3;
        }

        if keystate.is_pressed("ArrowLeft") {
            velocity.x -= 3;
        }

        self.position.x += velocity.x;
        self.position.y += velocity.y;

        if self.frame < 23 {
            self.frame += 1;
        } else {
            self.frame = 0;
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

        let current_sprite = (self.frame / 3) + 1;
        let frame_name = format!("Run ({}).png", current_sprite);
        let sprite = self
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.frames.get(&frame_name))
            .expect("Cell not found");

        self.image.as_ref().map(|image| {
            renderer.draw_image(
                &image,
                &Rect {
                    x: sprite.frame.x.into(),
                    y: sprite.frame.y.into(),
                    width: sprite.frame.w.into(),
                    height: sprite.frame.h.into(),
                },
                &Rect {
                    x: self.position.x.into(),
                    y: self.position.y.into(),
                    width: sprite.frame.w.into(),
                    height: sprite.frame.h.into(),
                },
            );
        });

        self.rhb.as_ref().unwrap().draw(renderer);
    }
}
