#[derive(Debug, Clone)]
pub struct Prompt {
    pub input:String,
    pub character_index: usize,
    pub message: String,

}

impl Prompt {
    pub fn new() -> Self {
        Self {
            input: "".to_string(),
            character_index: 0,
            message: String::new(),
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.character_index = 0;
    }


    pub fn submit_message(&mut self) {
        self.message = self.input.clone();
        self.input.clear();
        self.reset_cursor();
    }


    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        if index < 40 {
            self.input.insert(index, new_char);
            self.move_cursor_right();
        }
    }

    pub fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)|i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index= current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);

            let after_char_to_delete = self.input.chars().skip(current_index);


            self.input = before_char_to_delete.chain(after_char_to_delete).collect();

            self.move_cursor_left();
        }
    }

}