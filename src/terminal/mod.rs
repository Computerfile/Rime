use std::cmp;

use crate::terminal::cell::{Cell, CellInstance, CellState};

pub mod cell;

pub const CELL_WIDTH_PX: f32 = 16.0;
pub const CELL_HEIGHT_PX: f32 = 32.0;

pub struct Terminal {
    pub grid: Vec<Cell>,
    pub cols: u32,
    pub rows: u32,
    pub px_width: u32,
    pub px_height: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,


    pub cell_height_px: u16,
    pub cell_width_px: u16,
}


impl Terminal {
    pub fn new(px_width: u32, px_height: u32, cell_width_px: u16, cell_height_px: u16) -> Self {
        let width: f32 = px_width as f32 / cell_width_px as f32;
        let height: f32 = px_height as f32 / cell_height_px as f32;

        let grid = vec![Cell { codepoint: ' ' as u32, state: CellState::NotWritten } ; width as usize * height as usize];
        Self { 
            grid, 
            px_width, 
            px_height, 
            cursor_x: 0, 
            cursor_y: 0, 
            rows: height as u32, 
            cols: width as u32,
            cell_height_px, 
            cell_width_px
        }
    }

    
    pub fn resize(&mut self, new_px_width: u32, new_px_height: u32) {
        let new_cols = (new_px_width as f32 / self.cell_width_px as f32) as u32;
        let new_rows = (new_px_height as f32 / self.cell_height_px as f32) as u32;

        let mut new_grid = vec![Cell { codepoint: ' ' as u32, state: CellState::NotWritten }; (new_cols * new_rows) as usize];

        for old_index in 0..self.grid.len() {
            let old_row = old_index as u32 / self.cols;
            let old_col = old_index as u32 % self.cols;

            if old_row < new_rows && old_col < new_cols {
            let new_index = old_row * new_cols + old_col;
                new_grid[new_index as usize] = self.grid[old_index];
            }
        }


        self.grid = new_grid;
        self.cols = new_cols;
        self.rows = new_rows;
        self.px_width = new_px_width;
        self.px_height = new_px_height;
        self.cursor_x = self.cursor_x.min(new_cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(new_rows.saturating_sub(1));

    }

    pub fn write_char(&mut self, codepoint: u32) {
          
        let index = self.cursor_y * self.cols + self.cursor_x;
        self.grid[index as usize] = Cell { codepoint, state: CellState::Written };

        if self.cursor_x + 1 > self.cols {
            self.cursor_y += 1;
            if self.cursor_y + 1 > self.rows {
                // NO SPACE ??????
                tracing::warn!("Clearing the terminal dybamically has not been implemented blame the user not the dev");
                return;
            }
            self.cursor_x = 0;
        }else {
            self.cursor_x += 1;
        }



    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_x as i32 - 1 < 0 {
            if self.cursor_y as i32 - 1 < 0 {
                return;
            }
            self.cursor_y -= 1;
            self.cursor_x = self.cols - 1;
        }else {
            self.cursor_x -= 1;
        }

        let index = self.cursor_y * self.cols + self.cursor_x;
        self.grid[index as usize] = Cell { codepoint: ' ' as u32, state: CellState::NotWritten };
    }

}

