#[derive(Clone, Copy, PartialEq)]
pub enum CellState {
    Written,
    NotWritten,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub codepoint: u32,
    pub state: CellState,
}

#[repr(C)]
#[derive(bytemuck::Pod, Copy, Clone, bytemuck::Zeroable, Default)]
pub struct CellInstance {
    pub x: f32, 
    pub y: f32,

    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}
