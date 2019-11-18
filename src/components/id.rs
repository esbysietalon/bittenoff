use amethyst::ecs::prelude::{Component, VecStorage};
use uuid::Uuid;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Id{
    id: Uuid,
}

impl Id{
    pub fn new() -> Id {
        Id {
            id: Uuid::new_v4(),
        }
    }
    pub fn nil() -> Id {
        Id {
            id: Uuid::nil(),
        }
    }
    pub fn get_uuid(&self) -> Uuid {
        return self.id;
    }    
}

impl Component for Id{
    type Storage = VecStorage<Self>;
}