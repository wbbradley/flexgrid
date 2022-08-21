//
//  layout-rs/src/lib.rs {adapted from Layout.swift}
//
//  Created by William Bradley on 9/3/18.
//  Copyright © 2021, 2018 William Bradley. All rights reserved.
//
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Key<'a> {
    key: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Right,
    Down,
}

#[allow(dead_code)]
pub struct Padding {
    start_main: f32,
    start_cross: f32,
    end_main: f32,
    end_cross: f32,
}

#[allow(dead_code)]
pub enum Spacing {
    Pixels(f32),
    FlexBetween,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[allow(dead_code)]
impl Rect {
    fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[allow(dead_code)]
pub struct Item<'a> {
    key: Key<'a>,
    size_key: Key<'a>,
    frame: Rect,
}

#[allow(dead_code)]
impl<'a> Item<'a> {
    fn new(key: Key<'a>, size_key: Key<'a>) -> Self {
        Self {
            key: key,
            size_key: size_key,
            frame: Rect::empty(),
        }
    }
}

#[allow(dead_code)]
pub(crate) enum MaybeLinkedSize<'a> {
    Pixels(f32),
    Percent(f32),
    Flex(f32),
    Link(Key<'a>),
}

trait Layoutable {
    fn nest<T: Layoutable>(key: Key, direction: Direction, padding: Padding) -> T;
    fn add<'a>(size: MaybeLinkedSize) -> Item<'a>;
    fn space(size: MaybeLinkedSize) -> ();
}

#[allow(dead_code)]
pub struct LayoutInfo<'a> {
    direction: Direction,
    padding: Padding,
    items: Vec<Item<'a>>,
}

#[allow(dead_code)]
impl<'a> LayoutInfo<'a> {
    fn new(direction: Direction, padding: Padding) -> Self {
        Self {
            direction: direction,
            padding: padding,
            items: Vec::new(),
        }
    }
}

/*
class NestedLayout : Layoutable {
    weak var engine : Layout!
    var info : LayoutInfo

    init(engine: Layout, item: Item, direction: Direction, padding: Padding) {
        self.engine = engine
        self.info = LayoutInfo(direction: direction, padding: padding)
        self.engine.layoutMap[item.key] = self.info
    }

    func nest(key: Key, direction: Direction, padding: Padding) -> Layoutable {
        return engine.nest(key: key, direction: direction, padding: padding)
    }

    func space(size: MaybeLinkedSize) {
        let key = engine.genKey(prefix: "space_")
        let _ = self.add(key: key, size: size)
    }

    func add(size: MaybeLinkedSize) -> Item {
        let key = engine.genKey(prefix: "item_")
        return self.add(key: key, size: size)
    }

    func add(key: Key, size: MaybeLinkedSize) -> Item {
        let item = Item(key: key, size_key: engine.genSizeKey(size))
        engine.itemMap[key] = item
        info.items.append(item)
        return item
    }
}

class Layout : Layoutable {
    private static var nextKey : Int = 0
    var sizes = [Key: Size]()
    var itemMap = [Key: Item]()
    var layoutMap : [Key: LayoutInfo] = [Key: LayoutInfo]()
    var info : LayoutInfo

    init(direction: Direction, padding: Padding) {
        self.info = LayoutInfo(direction: direction, padding: padding)
    }

    func genKey(prefix: String) -> Key {
        Layout.nextKey += 1
        return "\(prefix)\(Layout.nextKey)"
    }

    func nest(key: Key, direction: Direction, padding: Padding) -> Layoutable {
        guard let item = itemMap[key] else { fatalError("Could not find item \(key)") }
        return NestedLayout(engine: self, item: item, direction: direction, padding: padding)
    }

    func genSizeKey(_ s: MaybeLinkedSize) -> Key {
        switch s {
        case .pixels(let v):
            let k = genKey(prefix: "size_")
            sizes[k] = .pixels(v)
            return k
        case .percent(let v):
            let k = genKey(prefix: "size_")
            sizes[k] = .percent(v)
            return k
        case .flex(let v):
            let k = genKey(prefix: "size_")
            sizes[k] = .flex(v)
            return k
        case .link(let k):
            if sizes.index(forKey: k) != nil {
                return k
            } else {
                fatalError("")
            }
        }
    }

    func space(size: MaybeLinkedSize) {
        let key = genKey(prefix: "space_")
        let _ = add(key: key, size: size)
    }

    func add(size: MaybeLinkedSize) -> Item {
        let key = genKey(prefix: "item_")
        return self.add(key: key, size: size)
    }

    func add(key: Key, size: MaybeLinkedSize) -> Item {
        let item = Item(key: key, size_key: genSizeKey(size))
        add(item: item)
        return item
    }

    func add(item: Item) {
        if itemMap.index(forKey: item.key) != nil {
            fatalError("Item \(item.key) already exists in engine.")
        }
        itemMap[item.key] = item
        info.items.append(item)
    }

    func solve(frame: CGRect) {
        Layout.layoutSolver(frame: frame,
                            info: self.info,
                            layoutMap: layoutMap,
                            sizes: &sizes)
    }

    static public func layoutSolver(frame: CGRect,
                                    info: LayoutInfo,
                                    layoutMap: [Key: LayoutInfo],
                                    sizes: inout [Key: Size]) {
        var flexGrowSum : CGFloat = 0

        /* track the used space in the main axis */
        var mainSum : CGFloat = 0

        for child in info.items {
            switch sizes[child.size_key]! {
            case .pixels(let px):
                mainSum += px
            case .percent(let pct):
                let pixels = getAxisLength(frame: frame, direction: info.direction) * pct / 100.0
                sizes[child.size_key] = .pixels(pixels)
                mainSum += pixels
            case .flex(let flexGrow):
                flexGrowSum += flexGrow
            }
        }

        let mainAvailable = getAxisLength(frame: frame, direction: info.direction) - info.padding.start_main - info.padding.end_main - mainSum
        if mainAvailable <= 0 {
            fatalError("Ran out of space for children because of margin overrun.")
        }

        for child in info.items {
            switch sizes[child.size_key]! {
            case .pixels(_):
                break
            case .flex(let flex):
                sizes[child.size_key] = .pixels(mainAvailable * flex/flexGrowSum)
            case .percent(_):
                fatalError("Percent should be eradicated by now")
            }
        }

        if flexGrowSum > 0 {
            if mainAvailable - mainSum - flexGrowSum < 0 {
                fatalError("Ran out of space for flex items")
            }
        } else if mainAvailable < -1 {
            fatalError("Ran out of space for non-flex items")
        }

        var mainCur : CGFloat
        switch info.direction {
        case .right:
            mainCur = frame.origin.x + info.padding.start_main
        case .down:
            mainCur = frame.origin.y + frame.size.height - info.padding.start_main
        }

        for child in info.items {
            switch sizes[child.size_key]! {
            case .pixels(let pixels):
                switch info.direction {
                case .right:
                    child.frame = CGRect(x: mainCur,
                                     y: frame.origin.y + info.padding.start_cross,
                                     width: pixels,
                                     height: frame.size.height - info.padding.end_cross - info.padding.start_cross)
                    mainCur += pixels
                case .down:
                    mainCur -= pixels
                    child.frame = CGRect(x: frame.origin.x + info.padding.start_cross,
                                     y: mainCur,
                                     width: frame.size.width - info.padding.end_cross - info.padding.start_cross,
                                     height: pixels)
                }
                if let info = layoutMap[child.key] {
                    /* recurse if there is layout info for this item */
                    Layout.layoutSolver(frame: child.frame,
                                        info: info,
                                        layoutMap: layoutMap,
                                        sizes: &sizes)
                }
                break
            case .percent(_):
                fatalError("Percent should not exist here.")
            case .flex(_):
                fatalError("item should have been converted to pixels")
            }
        }
    }
}

indirect enum Size {
    case pixels(CGFloat)
    case percent(CGFloat)
    case flex(CGFloat)
}

func getAxisLength(frame: CGRect, direction: Direction) -> CGFloat {
    switch direction {
    case .down:
        return frame.size.height
    case .right:
        return frame.size.width
    }
}

func other(direction: Direction) -> Direction {
    switch direction {
    case .right:
        return .down
    case .down:
        return .right
    }
}
*/
