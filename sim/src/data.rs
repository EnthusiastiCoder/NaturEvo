#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Behavior {
    Aggressive,
    Pack,
    Defensive,
    Berserker,
    Venomous,
    Trickster,
    Healer,
    Charger,
}

impl Behavior {
    pub fn name(self) -> &'static str {
        match self {
            Behavior::Aggressive => "Aggressive",
            Behavior::Pack => "Pack",
            Behavior::Defensive => "Defensive",
            Behavior::Berserker => "Berserker",
            Behavior::Venomous => "Venomous",
            Behavior::Trickster => "Trickster",
            Behavior::Healer => "Healer",
            Behavior::Charger => "Charger",
        }
    }
}
