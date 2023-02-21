
pub type Handle = u64;

pub trait HandleInterface {
    type HandleComponent;
    fn from_parts(handle: Self::HandleComponent, id: Self::HandleComponent) -> Self;
    fn handle_part(&self) -> Self::HandleComponent;
    fn id_part(&self) -> Self::HandleComponent;
}

impl HandleInterface for Handle {
    type HandleComponent = u32;

    #[inline]
    fn from_parts(handle: u32, id: u32) -> Handle {
        ((id as u64) << 32) | handle as u64
    }

    #[inline]
    fn handle_part(&self) -> u32 {
        (self & 0xffffffff) as u32
    }

    #[inline]
    fn id_part(&self) -> u32 {
        (self >> 32) as u32
    }
}
