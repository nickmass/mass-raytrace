#![allow(dead_code)]

use crate::material::Background;
use crate::world::{Camera, World};
use crate::InputCollection;

mod cornell;
pub use cornell::CornellBox;

mod eve;
pub use eve::Eve;

mod lucy;
pub use lucy::Lucy;

mod sphere_grid;
pub use sphere_grid::SphereGrid;

mod menger;
pub use menger::Menger;

mod mario;
pub use mario::Mario;

pub trait Scene {
    type Background: Background;
    fn generate(
        &mut self,
        animation_t: f32,
        frame: u32,
        input: &InputCollection,
    ) -> (World<Self::Background>, Camera);
}
