#[derive(Debug, Clone)]
pub enum Gender {
    Neutral,
    Male,
    Female
} 

impl Gender {
    pub fn from_string(string: impl Into<String>) -> Self {
        match string.into() {
            v if v == "male".to_string() => { Gender::Male },
            v if v == "female".to_string() => { Gender::Female },
            _ => { Gender::Neutral }
        }
    }

    pub fn to_string(&self) -> &str {
        match &self {
            Gender::Neutral => "neutral",
            Gender::Male => "male",
            Gender::Female => "female"
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Hidden,
    Unlisted,
    Public
}

impl Visibility {
    pub fn from_string(string: impl Into<String>) -> Self {
        match string.into() {
            v if v == "PRIVATE".to_string() => { Visibility::Hidden },
            v if v == "UNLISTED".to_string() => { Visibility::Unlisted },
            _ => { Visibility::Public }
        }
    }

    pub fn to_string(&self) -> &str {
        match &self {
            Visibility::Hidden => "PRIVATE",
            Visibility::Unlisted => "UNLISTED",
            Visibility::Public => "PUBLIC"
        }
    }
}