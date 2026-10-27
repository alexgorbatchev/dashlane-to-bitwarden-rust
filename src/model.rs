use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Export {
    pub encrypted: bool,
    pub folders: Vec<Folder>,
    pub items: Vec<Item>,
}

#[derive(Serialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Uuid,
    organization_id: Option<Uuid>,
    folder_id: Uuid,
    r#type: u8,
    name: String,
    notes: Option<String>,
    favorite: bool,
    reprompt: u8,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<Field>,
    #[serde(flatten)]
    data: Data,
}

#[derive(Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Data {
    Login {
        login: Login,
    },
    Card {
        card: Card,
    },
    Identity {
        identity: Box<Identity>,
    },
    Note {
        #[serde(rename = "secureNote")]
        secure_note: Note,
    },
}

impl Item {
    pub fn kind_index(&self) -> usize {
        usize::from(self.r#type - 1)
    }
    pub fn new(
        folder_id: Uuid,
        name: String,
        notes: Option<String>,
        fields: Vec<Field>,
        data: Data,
    ) -> Self {
        let r#type = match &data {
            Data::Login { .. } => 1,
            Data::Note { .. } => 2,
            Data::Card { .. } => 3,
            Data::Identity { .. } => 4,
        };
        Self {
            id: Uuid::new_v4(),
            organization_id: None,
            folder_id,
            r#type,
            name,
            notes,
            favorite: false,
            reprompt: 0,
            fields,
            data,
        }
    }
}

#[derive(Serialize)]
pub struct Login {
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub uris: Vec<LoginUri>,
}

#[derive(Serialize)]
pub struct LoginUri {
    pub r#match: Option<u8>,
    pub uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub cardholder_name: Option<String>,
    pub brand: Option<String>,
    pub number: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub code: Option<String>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub title: Option<String>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ssn: Option<String>,
    pub username: Option<String>,
    pub passport_number: Option<String>,
    pub license_number: Option<String>,
}

#[derive(Serialize)]
pub struct Note {
    pub r#type: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub value: String,
    pub r#type: u8,
    pub linked_id: Option<u32>,
}

impl Field {
    pub fn new(name: &str, value: &str, hidden: bool) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            r#type: u8::from(hidden),
            linked_id: None,
        }
    }
}
