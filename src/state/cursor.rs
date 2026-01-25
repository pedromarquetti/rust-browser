#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pos_x: usize,
    pos_y: usize,
}

pub enum Movement {
    Up,
    Down,
    Left,
    Right,
}

impl Cursor {
    pub fn new(x: usize, y: usize) -> Self {
        Self { pos_x: x, pos_y: y }
    }

    pub fn set_posx(&mut self, pos_x: usize) {
        self.pos_x = pos_x;
    }

    pub fn set_posy(&mut self, pos_y: usize) {
        self.pos_y = pos_y;
    }

    /// get cursor position (x,y)
    pub fn get_pos(&self) -> (usize, usize) {
        (self.pos_x, self.pos_y)
    }

    pub fn move_right(&mut self, max_size: usize) {
        if self.pos_x != max_size {
            self.pos_x += 1
        }
    }

    pub fn move_left(&mut self) {
        if self.pos_x as i32 != 0 {
            self.pos_x -= 1
        }
    }

    pub fn move_down(&mut self, max_size: usize) {
        if self.pos_y != max_size {
            self.pos_y += 1
        }
    }

    pub fn move_up(&mut self) {
        if self.pos_y as i32 != 0 {
            self.pos_y -= 1
        }
    }

    pub fn move_home(&mut self) {
        self.set_posx(0);
    }

    pub fn move_end(&mut self,val:usize) {
        self.set_posx(val);
    }
}
