use std::rc::Rc;
use web_sys::HtmlImageElement;

use crate::engine::{Image, Point, Rect, SpriteSheet};
use crate::game::{Barrier, Obstacle, Platform};

const LOW_PLATFORM: i16 = 420;
const HIGH_PLATFORM: i16 = 375;
const FIRST_PLATFORM: i16 = 370;

fn create_platform_bounding_boxes(destination_box: &Rect) -> [Rect; 3] {
    const X_OFFSET: i16 = 60;
    const END_HEIGHT: i16 = 54;
    let destination_box = destination_box;
    let bounding_box_one = Rect::new(destination_box.position, X_OFFSET, END_HEIGHT);

    let bounding_box_two = Rect::new_from_x_y(
        destination_box.x() + X_OFFSET,
        destination_box.y(),
        destination_box.width - (X_OFFSET * 2),
        destination_box.height,
    );

    let bounding_box_three = Rect::new_from_x_y(
        destination_box.x() + destination_box.width - X_OFFSET,
        destination_box.y(),
        X_OFFSET,
        END_HEIGHT,
    );

    [bounding_box_one, bounding_box_two, bounding_box_three]
}

fn create_floating_platform(sprite_sheet: Rc<SpriteSheet>, position: Point) -> Platform {
    let platform_builder = Platform::builder(sprite_sheet.clone(), position)
        .with_sprites(&["13.png", "14.png", "15.png"]);

    let bounding_boxes = create_platform_bounding_boxes(&platform_builder.destination_box());
    platform_builder
        .with_bounding_boxes(&bounding_boxes)
        .build()
}

pub fn rock_and_platform(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    vec![
        Box::new(Barrier::new(Image::new(
            stone,
            Point {
                x: offset_x + 150,
                y: 546,
            },
        ))),
        Box::new(create_floating_platform(
            sprite_sheet,
            Point {
                x: offset_x + FIRST_PLATFORM,
                y: LOW_PLATFORM,
            },
        )),
    ]
}

pub fn platform_and_rock(
    stone: HtmlImageElement,
    sprite_sheet: Rc<SpriteSheet>,
    offset_x: i16,
) -> Vec<Box<dyn Obstacle>> {
    vec![
        Box::new(Barrier::new(Image::new(
            stone,
            Point {
                x: offset_x + 400,
                y: 546,
            },
        ))),
        Box::new(create_floating_platform(
            sprite_sheet,
            Point {
                x: offset_x + 200,
                y: HIGH_PLATFORM,
            },
        )),
    ]
}
