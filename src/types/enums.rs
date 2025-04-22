#[derive(Debug)]
pub enum Gender {
    Neutral,
    Male,
    Female
} 

impl Gender {
    pub fn from_str(string: impl Into<String>) -> Self {
        match string.into() {
            v if v == "male".to_string() => { Gender::Male },
            v if v == "female".to_string() => { Gender::Female },
            _ => { Gender::Neutral }
        }
    }
}

#[derive(Debug)]
pub enum Visibility {
    Hidden,
    Unlisted,
    Public
}

impl Visibility {
    pub fn from_str(string: impl Into<String>) -> Self {
        match string.into() {
            v if v == "PRIVATE".to_string() => { Visibility::Hidden },
            v if v == "UNLISTED".to_string() => { Visibility::Unlisted },
            _ => { Visibility::Public }
        }
    }
}