use shared::Map;

pub trait Play {
    fn next_position(&mut self);
}

impl Play for Map {
    fn next_position(&mut self) {
        todo!()
    }
    /*
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
    }*/
}
