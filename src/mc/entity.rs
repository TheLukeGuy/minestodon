use num_enum::IntoPrimitive;

#[derive(Copy, Clone, IntoPrimitive)]
#[repr(i8)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}
