use ncurses::*;
use std::cmp;
use std::ops::{Add, Mul};

#[derive(Copy, Clone)]
struct Point(i32, i32);

impl Add for Point {
    type Output = Self;

    fn add(self, Self(x, y): Self) -> Self {
        Self(self.0 + x, self.1 + y)
    }
}

impl Mul for Point {
    type Output = Self;

    fn mul(self, Self(x, y): Self) -> Self {
        Self(self.0 * x, self.1 * y)
    }
}

enum LayoutType {
    Horz,
    Vert,
}

struct Layout {
    typ: LayoutType,
    pos: Point,
    size: Point,
    pad: i32,
}

impl Layout {
    fn new(typ: LayoutType, pos: Point, pad: i32) -> Self {
        Self {
            typ,
            pos,
            size: Point(0, 0),
            pad
        }
    }

    fn free_pos(&self) -> Point {
        match self.typ {
            LayoutType::Horz => self.pos + self.size * Point(1, 0),
            LayoutType::Vert => self.pos + self.size * Point(0, 1),
        }
    }

    fn add_size(&mut self, size: Point) {
        match self.typ {
            LayoutType::Horz => {
                self.size.0 += size.0 + self.pad;
                self.size.1 = cmp::max(self.size.1, size.1);
            },
            LayoutType::Vert => {
                self.size.0 = cmp::max(self.size.0, size.0);
                self.size.1 += size.1 + self.pad;
            }
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
struct Id(usize);

struct ImTui {
    hot: Option<Id>,
    active: Option<Id>,
    layouts: Vec<Layout>,
    key: Option<i32>,
}

impl ImTui {
    fn new() -> Self {
        Self {
            active: None,
            hot: None,
            layouts: Vec::new(),
            key: None,
        }
    }

    fn begin(&mut self, pos: Point) {
        self.layouts.push(Layout::new(LayoutType::Vert, pos, 0));
    }

    fn begin_layout(&mut self, typ: LayoutType, pad: i32) {
        let pos = self.layouts.last().unwrap().free_pos();
        self.layouts.push(Layout::new(typ, pos, pad));
    }

    fn end_layout(&mut self) {
        let layout = self.layouts.pop().unwrap();
        self.layouts.last_mut().unwrap().add_size(layout.size);
    }

    fn end(&mut self) {
        self.layouts.pop().unwrap();
        self.key = None;
    }

    fn feed_key(&mut self, key: i32) {
        self.key = Some(key)
    }
}

fn label(imtui: &mut ImTui, text: &str) {
    let pos = imtui.layouts.last().unwrap().free_pos();
    mv(pos.1, pos.0);
    attron(COLOR_PAIR(INACTIVE_PAIR));
    addstr(&text);
    attroff(COLOR_PAIR(INACTIVE_PAIR));
    imtui.layouts.last_mut().unwrap().add_size(Point(text.len() as i32, 1));
}

fn checkbox(imtui: &mut ImTui, text: &str, state: &mut bool, my_id: Id) -> bool {
    let mut clicked = false;
    let mut pair = INACTIVE_PAIR;
    if imtui.active == Some(my_id) {
        imtui.active = None;
        clicked = true;
    } else if imtui.hot == Some(my_id) {
        pair = HOT_PAIR;
        if imtui.active.is_none() {
            if imtui.key == Some(10) {
                imtui.active = Some(my_id);
                pair = ACTIVE_PAIR;
            }
        }
    }

    if clicked {
        *state = !*state;
    }

    let pos = imtui.layouts.last().unwrap().free_pos();

    attron(COLOR_PAIR(pair));
    mv(pos.1, pos.0);

    let s = format!("[{}] {}", if *state {"X"} else {" "}, text);
    addstr(&s);

    imtui.layouts.last_mut().unwrap().add_size(Point(s.len() as i32, 1));

    attroff(COLOR_PAIR(pair));

    return clicked;
}

fn button(imtui: &mut ImTui, label: &str, id: Id) -> bool {
    let mut clicked = false;
    let mut pair = INACTIVE_PAIR;

    if imtui.active == Some(id) {
        imtui.active = None;
        clicked = true;
    } else if imtui.hot == Some(id)  {
        pair = HOT_PAIR;
        if imtui.active.is_none() {
            if imtui.key == Some(10) {
                imtui.active = Some(id);
                pair = ACTIVE_PAIR;
            }
        }
    }

    let pos = imtui.layouts.last().unwrap().free_pos();

    attron(COLOR_PAIR(pair));
    mv(pos.1, pos.0);

    let text = format!("[ {} ]", label);
    addstr(&text);

    imtui.layouts.last_mut().unwrap().add_size(Point(text.len() as i32, 1));

    attroff(COLOR_PAIR(pair));

    return clicked;
}

const EDIT_FIELD_SIZE: Point = Point(20, 1);

fn edit_field(imtui: &mut ImTui, buffer: &mut String, _cursor: &mut usize, id: Id) {
    let mut pair = INACTIVE_PAIR;

    if imtui.active == Some(id) {
        if let Some(key) = imtui.key {
            match key {
                27 | 10 => imtui.active = None,
                32..=127 => buffer.push(key as u8 as char),
                _ => {}
            }
        }
    } else if imtui.hot == Some(id) {
        pair = HOT_PAIR;
        if imtui.active.is_none() {
            if imtui.key == Some(10) {
                imtui.active = Some(id);
                pair = INACTIVE_PAIR;
            }
        }
    }

    let pos = imtui.layouts.last().unwrap().free_pos();

    attron(COLOR_PAIR(pair));
    mv(pos.1, pos.0);

    let text = buffer.get(0..EDIT_FIELD_SIZE.0 as usize).unwrap_or(buffer);
    addstr(&text);

    if text.len() < EDIT_FIELD_SIZE.0 as usize {
        let n = EDIT_FIELD_SIZE.0 as usize - text.len();
        for _i in 0..n {
            addstr(" ");
        }
    }

    attroff(COLOR_PAIR(pair));

    imtui.layouts.last_mut().unwrap().add_size(EDIT_FIELD_SIZE);
}

const INACTIVE_PAIR: i16 = 1;
const HOT_PAIR: i16 = 2;
const ACTIVE_PAIR: i16 = 3;

struct GenId {
    count: usize
}

impl GenId {
    fn new() -> Self {
        Self {count: 0}
    }

    fn next(&mut self) -> Id {
        let id = self.count;
        self.count += 1;
        Id(id)
    }
}

fn main() {
    initscr();
    noecho();
    timeout(16);

    start_color();
    init_pair(INACTIVE_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HOT_PAIR, COLOR_BLACK, COLOR_WHITE);
    init_pair(ACTIVE_PAIR, COLOR_BLACK, COLOR_RED);

    let mut imtui = ImTui::new();
    let mut quit = false;

    imtui.hot = Some(Id(0));

    let mut gen_id = GenId::new();

    let mut first_name = String::new();
    let mut first_name_cursor: usize = 0;
    let first_name_id = gen_id.next();
    let mut last_name = String::new();
    let mut last_name_cursor: usize = 0;
    let last_name_id = gen_id.next();
    let submit_id = gen_id.next();
    let clear_id = gen_id.next();
    let quit_id = gen_id.next();
    let hide_db_id = gen_id.next();
    let mut hide_db_state = false;

    let mut database = Vec::<(String, String)>::new();

    while !quit {
        erase();

        imtui.begin(Point(0, 0));
        {
            if imtui.active.is_none() {
                match imtui.key.map(|x| x as u8 as char) {
                    Some('s') => {
                        imtui.hot = imtui.hot.map(|Id(x)| Id((x + 1) % gen_id.count));
                    }
                    Some('w') => {
                        imtui.hot = imtui
                            .hot
                            .map(|Id(x)| if x == 0 { Id(gen_id.count - 1) } else { Id(x - 1) });
                    }
                    Some('q') => {
                        quit = true
                    },
                    _ => {}
                }
            }

            checkbox(&mut imtui, "Hide Database", &mut hide_db_state, hide_db_id);

            if !hide_db_state {
                label(&mut imtui, "------------------------------");
                for (first, last) in database.iter() {
                    label(&mut imtui, &format!("{} | {}", first, last));
                }
            }

            label(&mut imtui, "------------------------------");

            imtui.begin_layout(LayoutType::Horz, 1);
            {
                label(&mut imtui, "First Name:");
                edit_field(&mut imtui, &mut first_name, &mut first_name_cursor, first_name_id);
            }
            imtui.end_layout();

            imtui.begin_layout(LayoutType::Horz, 1);
            {
                label(&mut imtui, "Last Name: ");
                edit_field(&mut imtui, &mut last_name, &mut last_name_cursor, last_name_id);
            }
            imtui.end_layout();

            label(&mut imtui, "------------------------------");

            imtui.begin_layout(LayoutType::Horz, 1);
            {
                if button(&mut imtui, "Submit", submit_id) {
                    database.push((first_name.clone(), last_name.clone()));
                    first_name.clear();
                    last_name.clear();
                }

                if button(&mut imtui, "Clear", clear_id) {
                    first_name.clear();
                    last_name.clear();
                }

                if button(&mut imtui, "Quit", quit_id) {
                    quit = true;
                }
            }
            imtui.end_layout();

        }
        imtui.end();

        refresh();

        imtui.feed_key(getch());
    }

    endwin();
}
