use shared::Map;

const PLAYER_ICON: char = '😃';
const EMPTY_SPACE_ICON: char = '.';

pub trait Play {
    fn new(width: u16, height: u16) -> Self;
    fn next_position(&mut self);
}

impl Play for Map {
    fn new(width: u16, height: u16) -> Self {
        let mut grid = vec![vec![EMPTY_SPACE_ICON; width as usize]; height as usize];
        grid[0][0] = PLAYER_ICON;
        Self {
            map: grid,
            cur_x: 0,
            cur_y: 0,
        }
    }

    fn next_position(&mut self) {
        self.map[self.cur_y][self.cur_x] = EMPTY_SPACE_ICON;

        if self.cur_x + 1 >= self.map[0].len() {
            self.cur_x = 0;
            if self.cur_y + 1 >= self.map.len() {
                self.cur_y = 0;
            } else {
                self.cur_y += 1;
            }
        } else {
            self.cur_x += 1;
        }

        self.map[self.cur_y][self.cur_x] = PLAYER_ICON;
    }
}
