use uuid::Uuid;

pub struct Household {
    // This is unique to the specific output area - ~250 households
    pub unique_id: Uuid,
    pub residents: Vec<Uuid>,
}


#[derive(Deserialize, Debug, Enum)]
pub enum PersonType {
    #[serde(alias = "All usual residents")]
    All,
    #[serde(alias = "Males")]
    Male,
    #[serde(alias = "Females")]
    Female,
    #[serde(alias = "Lives in a household")]
    LivesInHousehold,
    #[serde(alias = "Lives in a communal establishment")]
    LivesInCommunalEstablishment,
    #[serde(alias = "Schoolchild or full-time student aged 4 and over at their non term-time address")]
    Schoolchild,
}