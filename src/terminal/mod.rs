use crate::terminal::cell::{Cell, CellInstance, CellState};

pub mod cell;

pub const CELL_WIDTH_PX: f32 = 8.0;
pub const CELL_HEIGHT_PX: f32 = 16.0;

pub struct Terminal {
    pub grid: Vec<Cell>,
    pub cols: u32,
    pub rows: u32,
    pub px_width: u32,
    pub px_height: u32,
    pub cursor_x: u32,
    pub cursor_y: u32,
}


impl Terminal {
    pub fn new(px_width: u32, px_height: u32) -> Self {
        let width: f32 = px_width as f32 / CELL_WIDTH_PX;
        let height: f32 = px_height as f32 / CELL_HEIGHT_PX;

        let grid = vec![Cell { codepoint: ' ' as u32, state: CellState::NotWritten } ; width as usize * height as usize];
        Self { grid, px_width, px_height, cursor_x: 0, cursor_y: 0, rows: height as u32, cols: width as u32 }
    }

    pub fn build_instance(&mut self, surface_width: u32, surface_height: u32) -> Vec<CellInstance> {
        let mut grid_instance: Vec<CellInstance> = Vec::new();
        for index in 0..self.grid.len() {
            let cell: &Cell = self.grid.get(index).unwrap();
            if cell.state == CellState::NotWritten {
                continue;
            }

            let row = (index / self.cols as usize) as f32;
            let col = (index % self.cols as usize) as f32;
                
            let cell_width_ndc = (CELL_WIDTH_PX * 2.0) / surface_width as f32;
            let cell_height_ndc = (CELL_HEIGHT_PX * 2.0) / surface_height as f32; 

            let x_ndc = col * cell_width_ndc;
            let y_ndc = -(row * cell_height_ndc);


            // grid_instance.push(CellInstance { x: x_ndc, y: y_ndc });

        }
        grid_instance
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


}

