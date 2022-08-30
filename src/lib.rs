//
//  flexgrid/src/lib.rs
//
//  Created by William Bradley on 9/3/18.
//  Copyright 2022, 2021, 2018 William Bradley.
//

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Key {
    key: String,
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Direction {
    Right,
    Down,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Padding {
    start_main: f64,
    start_cross: f64,
    end_main: f64,
    end_cross: f64,
}

#[allow(dead_code)]
impl Padding {
    pub fn none() -> Self {
        Self::equal(0.0)
    }

    pub fn equal(amount: f64) -> Self {
        Self {
            start_main: amount,
            start_cross: amount,
            end_main: amount,
            end_cross: amount,
        }
    }
}

#[allow(dead_code)]
pub enum Spacing {
    Pixels(f64),
    FlexBetween,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl From<&Vector2> for Vector2 {
    fn from(v: &Vector2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[allow(dead_code)]
impl Rect {
    fn mid_x(&self) -> f64 {
        self.x + self.width * 0.5
    }
    fn mid_y(&self) -> f64 {
        self.y + self.height * 0.5
    }
    fn max_x(&self) -> f64 {
        self.x + self.width
    }
    fn max_y(&self) -> f64 {
        self.y + self.height
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct RectSize {
    pub width: f64,
    pub height: f64,
}

pub trait Lerp<NDimensionalPoint> {
    fn lerp(&self, p: NDimensionalPoint) -> NDimensionalPoint;
}

pub trait Alpha<NDimensionalPoint> {
    fn alpha(&self, p: NDimensionalPoint) -> NDimensionalPoint;
}

impl Lerp<Vector2> for Rect {
    fn lerp(&self, alpha: Vector2) -> Vector2 {
        return Vector2 {
            x: alpha.x * self.width + self.x,
            y: alpha.y * self.height + self.y,
        };
    }
}

impl Alpha<Vector2> for Rect {
    fn alpha(&self, point: Vector2) -> Vector2 {
        return Vector2 {
            x: (point.x - self.x) / self.width,
            y: (point.y - self.y) / self.height,
        };
    }
}

#[allow(dead_code)]
impl Rect {
    pub fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rect({}, {}, {}, {})",
            self.x, self.y, self.width, self.height
        )
    }
}

pub enum Size {
    Pixels(f64),
    Percent(f64),
    Flex(f64),
}

pub struct Item<'a> {
    key: Key,
    size_key: Key,
    on_layout: Box<dyn 'a + FnMut(Rect) -> ()>,
}

impl<'a> std::fmt::Debug for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Item {{ key: {}, size_key: {} }}",
            self.key, self.size_key
        )
    }
}

impl<'a> std::fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Item {{ key: {}, size_key: {} }}",
            self.key, self.size_key
        )
    }
}

impl<'a> Item<'a> {
    fn new<T>(key: Key, size_key: Key, on_layout: T) -> Self
    where
        T: 'a + FnMut(Rect) -> (),
    {
        Self {
            key: key,
            size_key: size_key,
            on_layout: Box::new(on_layout),
        }
    }

    pub(crate) fn callback(&mut self, rect: Rect) {
        (self.on_layout)(rect)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Spec {
    Pixels(f64),
    Percent(f64),
    Flex(f64),
}

#[allow(dead_code)]
pub struct LayoutInfo {
    direction: Direction,
    padding: Padding,
    items: Vec<Key>,
}

#[allow(dead_code)]
impl LayoutInfo {
    fn new(direction: Direction, padding: Padding) -> Self {
        Self {
            direction: direction,
            padding: padding,
            items: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct LayoutError {
    message: String,
}

impl LayoutError {
    fn error<T>(message: T) -> LayoutError
    where
        T: Into<String>,
    {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[layout error] {}", self.message)
    }
}

impl std::error::Error for LayoutError {}

pub struct Layout<'a> {
    next_key: i64,
    sizes: HashMap<Key, Size>,
    item_map: HashMap<Key, Item<'a>>,
    info: LayoutInfo,
}

impl<'a> Layout<'a> {
    pub fn new(direction: Direction, padding: Padding) -> Self {
        Self {
            next_key: 1,
            sizes: HashMap::new(),
            item_map: HashMap::new(),
            info: LayoutInfo::new(direction, padding),
        }
    }

    fn generate_key(&mut self, prefix: &'a str) -> Key {
        self.next_key += 1;
        Key {
            key: format!("{}{}", prefix, self.next_key),
        }
    }

    #[allow(dead_code)]
    pub fn space(&mut self, size: Spec) -> Result<(), LayoutError> {
        let key = self.generate_key("space_");
        self.add_with_key(key, size, move |_| ())?;
        Ok(())
    }

    pub fn add<T>(&mut self, size: Spec, on_layout: T) -> Result<Key, LayoutError>
    where
        T: 'a + FnMut(Rect) -> (),
    {
        let key = self.generate_key("item_");
        self.add_with_key(key.clone(), size, on_layout)?;
        Ok(key)
    }

    fn add_with_key<T>(&mut self, key: Key, size: Spec, on_layout: T) -> Result<(), LayoutError>
    where
        T: 'a + FnMut(Rect) -> (),
    {
        let item = Item::new(key, self.generate_size_key(size)?, Box::new(on_layout));
        let _key = self.add_item(item);
        Ok(())
    }

    fn add_item(&mut self, item: Item<'a>) -> Result<(), LayoutError> {
        if self.item_map.contains_key(&item.key) {
            Err(LayoutError::error(format!(
                "Item {} already exists in engine.",
                item.key
            )))
        } else {
            self.info.items.push(item.key.clone());
            self.item_map.insert(item.key.clone(), item);
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn nest<F>(
        &mut self,
        spec: Spec,
        direction: Direction,
        padding: Padding,
        mut f: F,
    ) -> Result<Key, LayoutError>
    where
        F: 'a + FnMut(&mut Self) -> (),
    {
        let key = self.add(spec, move |_rect: Rect| -> () {
            let mut layout = Self::new(direction, padding);
            f(&mut layout);
        })?;
        Ok(key)
    }

    fn generate_size_key(&mut self, s: Spec) -> Result<Key, LayoutError> {
        match s {
            Spec::Pixels(v) => {
                let k = self.generate_key("size_");
                self.sizes.insert(k.clone(), Size::Pixels(v));
                Ok(k)
            }
            Spec::Percent(v) => {
                let k = self.generate_key("size_");
                self.sizes.insert(k.clone(), Size::Percent(v));
                Ok(k)
            }
            Spec::Flex(v) => {
                let k = self.generate_key("size_");
                self.sizes.insert(k.clone(), Size::Flex(v));
                Ok(k)
            }
        }
    }
    pub fn solve(self, frame: Rect) -> Result<HashMap<Key, Rect>, LayoutError> {
        layout_solver(frame, self.info, self.item_map, self.sizes)
    }
}

fn layout_solver<'a>(
    frame: Rect,
    info: LayoutInfo,
    mut item_map: HashMap<Key, Item<'a>>,
    mut sizes: HashMap<Key, Size>,
) -> Result<HashMap<Key, Rect>, LayoutError> {
    let mut flex_grow_sum: f64 = 0.0;

    /* track the used space in the main axis */
    let mut main_sum: f64 = 0.0;

    for child_key in &info.items {
        let child: &Item<'a> = item_map.get(&child_key).unwrap();
        match sizes.get(&child.size_key) {
            Some(Size::Pixels(px)) => {
                main_sum += px;
            }
            Some(Size::Percent(pct)) => {
                let pixels = get_axis_length(&frame, info.direction) * pct / 100.0;
                sizes.insert(child.size_key.clone(), Size::Pixels(pixels));
                main_sum += pixels;
            }
            Some(Size::Flex(flex_grow)) => {
                flex_grow_sum += flex_grow;
            }
            None => {
                return Err(LayoutError::error(format!(
                    "missing size_key {}",
                    child.size_key
                )));
            }
        }
    }

    let main_available = get_axis_length(&frame, info.direction)
        - info.padding.start_main
        - info.padding.end_main
        - main_sum;
    if main_available <= 0.0 {
        return Err(LayoutError::error(
            "Ran out of space for children because of margin overrun.",
        ));
    }
    let mut final_sizes: HashMap<Key, f64> = HashMap::new();
    for child_key in &info.items {
        let child: &Item<'a> = item_map.get(&child_key).unwrap();
        match sizes.get(&child.size_key) {
            Some(Size::Pixels(pixels)) => {
                final_sizes.insert(child.size_key.clone(), *pixels);
            }
            Some(Size::Flex(flex)) => {
                final_sizes.insert(
                    child.size_key.clone(),
                    main_available * flex / flex_grow_sum,
                );
            }
            Some(Size::Percent(_)) => {
                return Err(LayoutError::error("Percent should be eradicated by now"))
            }
            None => {
                return Err(LayoutError::error(format!(
                    "Unexpected missing size [size_key={}]",
                    child.size_key
                )))
            }
        }
    }

    if flex_grow_sum > 0.0 {
        if main_available - flex_grow_sum < 0.0 {
            return Err(LayoutError::error(format!(
                "Ran out of space for flex items [flex_grow_sum={}, main_sum={}, main_available={}]",
                flex_grow_sum, main_sum, main_available
            )));
        }
    } else if main_available < -1.0 {
        return Err(LayoutError::error(format!(
            "Ran out of space for non-flex items"
        )));
    }

    let mut main_cur: f64 = match &info.direction {
        Direction::Right => frame.x + info.padding.start_main,
        Direction::Down => frame.y + info.padding.start_main,
    };
    let mut final_rects: HashMap<Key, Rect> = HashMap::new();
    for child_key in info.items {
        let mut child: Item<'a> = item_map.remove(&child_key).unwrap();
        match final_sizes.get(&child.size_key) {
            Some(pixels) => match info.direction {
                Direction::Right => {
                    let rect = Rect {
                        x: main_cur,
                        y: frame.y + info.padding.start_cross,
                        width: *pixels,
                        height: frame.height - info.padding.end_cross - info.padding.start_cross,
                    };
                    final_rects.insert(child_key.clone(), rect.clone());
                    child.callback(rect);
                    main_cur += pixels;
                }
                Direction::Down => {
                    let rect = Rect {
                        x: frame.x + info.padding.start_cross,
                        y: main_cur,
                        width: frame.width - info.padding.end_cross - info.padding.start_cross,
                        height: *pixels,
                    };
                    main_cur += pixels;
                    child.callback(rect);
                    final_rects.insert(child_key.clone(), rect);
                }
            },
            None => {
                return Err(LayoutError::error(format!(
                    "missing size for size_key {}",
                    child.size_key
                )))
            }
        }
    }
    Ok(final_rects)
}

fn get_axis_length(frame: &Rect, direction: Direction) -> f64 {
    match direction {
        Direction::Down => frame.height,
        Direction::Right => frame.width,
    }
}

#[allow(dead_code)]
pub fn grid(
    frame: Rect,
    cols: i32,
    rows: i32,
    cell_aspect_ratio: f64,
    spacing_ratio: f64,
) -> Vec<Rect> {
    let min_size = if cell_aspect_ratio >= 1.0 {
        RectSize {
            width: f64::from(cols) + f64::from(cols - 1) * spacing_ratio,
            height: f64::from(rows) * cell_aspect_ratio + f64::from(rows - 1) * spacing_ratio,
        }
    } else {
        RectSize {
            width: f64::from(cols) / cell_aspect_ratio
                + f64::from(cols - 1) * spacing_ratio / cell_aspect_ratio,
            height: f64::from(rows) + f64::from(rows - 1) * spacing_ratio,
        }
    };

    let m = f64::from(rows);
    let n = f64::from(cols);
    assert!(min_size.width >= n);
    assert!(min_size.height >= m);

    let grid_aspect_ratio = min_size.height / min_size.width;
    let frame_aspect_ratio = frame.height / frame.width;
    let mut frames = Vec::new();

    if frame_aspect_ratio > grid_aspect_ratio {
        /* frame is taller than the grid is tall, grid should be vertically centered and touch sides */
        let spacing_pixels = (spacing_ratio * frame.width + n + 1.0).ceil();
        let cell_width = (frame.width - spacing_pixels) / n;
        let cell_height = cell_width * cell_aspect_ratio;
        let spacing = (spacing_pixels / (n + 1.0)).ceil();
        let grid_height = f64::from(rows) * cell_height + f64::from(rows - 1) * spacing;
        for y in 0..rows {
            let yf = f64::from(y);
            for x in 0..cols {
                let xf = f64::from(x);
                let cell_frame = Rect {
                    x: (frame.x + xf * cell_width + (xf + 1.0) * spacing).floor(),
                    y: (frame.mid_y() - grid_height / 2.0 + yf * (cell_height + spacing)).floor(),
                    width: cell_width.floor(),
                    height: cell_height.floor(),
                };
                frames.push(cell_frame);
            }
        }
    } else {
        /* frame is wider than the grid is wide, grid should be horizontally centered and touch top/bottom */
        let spacing_pixels = (spacing_ratio * frame.height + m + 1.0).ceil();

        let cell_height = (frame.height - spacing_pixels) / m;
        let cell_width = cell_height / cell_aspect_ratio;
        let spacing = (spacing_pixels / (m + 1.0)).ceil();
        let grid_width = f64::from(cols) * cell_width + f64::from(cols - 1) * spacing;
        for y in 0..rows {
            let yf = f64::from(y);
            for x in 0..cols {
                let xf = f64::from(x);
                let cell_frame = Rect {
                    x: (frame.mid_x() - grid_width / 2.0 + xf * (cell_width + spacing)).floor(),
                    y: (frame.y + yf * cell_height + (yf + 1.0) * spacing).floor(),
                    width: cell_width.floor(),
                    height: cell_height.floor(),
                };
                frames.push(cell_frame);
            }
        }
    }
    frames
}
