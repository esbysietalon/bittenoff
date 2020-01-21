use amethyst::ecs::prelude::{Component, VecStorage};
use uuid::Uuid;
use crate::game_state::{EntityType};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Id{
    id: Uuid,
    etype: EntityType,
}

impl Id{
    pub fn new(etype: EntityType) -> Id {
        Id {
            id: Uuid::new_v4(),
            etype,
        }
    }
    pub fn nil() -> Id {
        Id {
            id: Uuid::nil(),
            etype: EntityType::Size,
        }
    }
    pub fn get_uuid(&self) -> Uuid {
        self.id
    }    
    pub fn get_type(&self) -> EntityType {
        self.etype
    }
}

impl Component for Id{
    type Storage = VecStorage<Self>;
}