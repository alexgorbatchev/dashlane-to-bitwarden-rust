use crate::{
    input::{Input, Row},
    model::*,
};
use anyhow::Result;
use std::path::Path;
use uuid::Uuid;

fn value<'a>(row: &'a Row, key: &str) -> &'a str {
    row.get(key).map_or("", String::as_str)
}
fn optional(row: &Row, key: &str) -> Option<String> {
    let text = value(row, key);
    (!text.is_empty()).then(|| text.into())
}
fn name(row: &Row, keys: &[&str], fallback: &str) -> String {
    keys.iter()
        .map(|key| value(row, key))
        .find(|text| !text.is_empty())
        .unwrap_or(fallback)
        .into()
}
fn extra(row: &Row, specs: &[(&str, &str, bool)], keep_empty: bool) -> Vec<Field> {
    specs
        .iter()
        .filter_map(|(key, label, hidden)| {
            let text = value(row, key);
            (keep_empty || !text.is_empty()).then(|| Field::new(label, text, *hidden))
        })
        .collect()
}

pub fn convert(path: &Path) -> Result<Export> {
    let mut input = Input::open(path)?;
    let folders = [
        "Logins",
        "Cards",
        "Bank Accounts",
        "Identities",
        "IDs",
        "Secure Notes",
    ]
    .into_iter()
    .map(|name| Folder {
        id: Uuid::new_v4(),
        name: name.into(),
    })
    .collect::<Vec<_>>();
    let mut items = Vec::new();
    for row in input.rows("credentials.csv")? {
        let uris = optional(&row, "url")
            .into_iter()
            .map(|uri| LoginUri { r#match: None, uri })
            .collect();
        items.push(Item::new(
            folders[0].id,
            name(&row, &["title", "url"], "Login"),
            optional(&row, "note"),
            extra(
                &row,
                &[
                    ("username2", "username2", false),
                    ("username3", "username3", false),
                ],
                false,
            ),
            Data::Login {
                login: Login {
                    username: optional(&row, "username"),
                    password: optional(&row, "password"),
                    totp: optional(&row, "otpUrl"),
                    uris,
                },
            },
        ));
    }
    let payments = input.rows("payments.csv")?;
    for row in payments
        .iter()
        .filter(|row| value(row, "type") == "payment_card")
    {
        items.push(Item::new(
            folders[1].id,
            name(row, &["name", "account_name"], "Card"),
            optional(row, "note"),
            vec![],
            Data::Card {
                card: Card {
                    cardholder_name: optional(row, "account_holder"),
                    brand: optional(row, "issuing_bank"),
                    number: optional(row, "cc_number"),
                    exp_month: optional(row, "expiration_month"),
                    exp_year: optional(row, "expiration_year"),
                    code: optional(row, "code"),
                },
            },
        ));
    }
    for row in payments.iter().filter(|row| value(row, "type") == "bank") {
        items.push(Item::new(
            folders[2].id,
            name(row, &["account_name"], "Bank Account"),
            optional(row, "note"),
            extra(
                row,
                &[
                    ("account_holder", "Account Holder", false),
                    ("routing_number", "Routing Number", true),
                    ("account_number", "Account Number", true),
                    ("issuing_bank", "Bank", false),
                    ("country", "Country", false),
                ],
                true,
            ),
            Data::Note {
                secure_note: Note { r#type: 0 },
            },
        ));
    }
    for row in input
        .rows("personalInfo.csv")?
        .iter()
        .filter(|row| value(row, "type") == "name")
    {
        let full_name = [value(row, "first_name"), value(row, "last_name")]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let item_name = if full_name.is_empty() {
            name(row, &["item_name"], "Identity")
        } else {
            full_name
        };
        items.push(Item::new(
            folders[3].id,
            item_name,
            None,
            vec![],
            Data::Identity {
                identity: Box::new(Identity {
                    title: optional(row, "title"),
                    first_name: optional(row, "first_name"),
                    middle_name: optional(row, "middle_name"),
                    last_name: optional(row, "last_name"),
                    address1: optional(row, "address"),
                    city: optional(row, "city"),
                    state: optional(row, "state"),
                    postal_code: optional(row, "zip"),
                    country: optional(row, "country"),
                    email: optional(row, "email"),
                    phone: optional(row, "phone_number"),
                    username: optional(row, "login"),
                    ..Identity::default()
                }),
            },
        ));
    }
    for row in input.rows("ids.csv")? {
        let mut fields = extra(
            &row,
            &[
                ("issue_date", "Issue Date", false),
                ("expiration_date", "Expiration Date", false),
            ],
            false,
        );
        let mut identity = Identity {
            state: optional(&row, "state"),
            country: optional(&row, "place_of_issue"),
            ..Identity::default()
        };
        match value(&row, "type") {
            "social_security" => identity.ssn = optional(&row, "number"),
            "passport" => identity.passport_number = optional(&row, "number"),
            "license" => identity.license_number = optional(&row, "number"),
            _ => fields.extend(extra(
                &row,
                &[
                    ("type", "Document Type", false),
                    ("number", "Document Number", true),
                ],
                false,
            )),
        }
        let fallback = format!("{} - {}", value(&row, "type"), value(&row, "number"));
        items.push(Item::new(
            folders[4].id,
            name(&row, &["name"], &fallback),
            None,
            fields,
            Data::Identity {
                identity: Box::new(identity),
            },
        ));
    }
    for row in input.rows("securenotes.csv")? {
        items.push(Item::new(
            folders[5].id,
            name(&row, &["title"], "Secure Note"),
            optional(&row, "note"),
            vec![],
            Data::Note {
                secure_note: Note { r#type: 0 },
            },
        ));
    }
    Ok(Export {
        encrypted: false,
        folders,
        items,
    })
}
