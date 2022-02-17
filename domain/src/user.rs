use crate::{Id, Metadata, WithJsonProcessor, WithMetadata};
use serde::{Deserialize, Serialize};

#[derive(PartialOrd, PartialEq, Debug, Serialize, Deserialize, WithMetadata, WithJsonProcessor)]
pub struct User {
    domain_metadata: Metadata,
    nickname: String,
    password: String,
    profile: Profile,
    roles: Vec<String>,
}

impl User {
    pub fn new(nickname: &str, roles: Vec<&'static str>, password: &str, profile: Profile) -> Self {
        User {
            domain_metadata: Default::default(),
            nickname: String::from(nickname),
            password: String::from(password),
            roles: roles.into_iter().map(String::from).collect(),
            profile,
        }
    }
    pub fn set_password(self, password: &str) -> Self {
        User {
            password: String::from(password),
            ..self
        }
    }
    pub fn id(&self) -> &Id {
        self.domain_metadata.id()
    }
    pub fn nickname(&self) -> &str {
        &self.nickname
    }
    pub fn password(&self) -> &str {
        &self.password
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn roles(&self) -> Vec<&String> {
        let roles: Vec<&String> = self.roles.iter().collect();
        roles
    }
    pub fn add_role(&mut self, role: &str) -> &mut Self {
        self.roles.push(String::from(role));
        self
    }
    pub fn remove_role(&mut self, role: &str) -> &mut Self {
        let idx = self.roles.binary_search(&String::from(role));
        if let Ok(idx) = idx {
            self.roles.remove(idx);
        }
        self
    }
}

#[derive(PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Profile {
    picture: Option<Metadata>,
    firstname: String,
    lastname: String,
    phone_number: String,
    email_address: String,
    address: Address,
}

impl Profile {
    pub fn picture(&self) -> &Option<Metadata> {
        &self.picture
    }
    pub fn firstname(&self) -> &str {
        &self.firstname
    }
    pub fn lastname(&self) -> &str {
        &self.lastname
    }
    pub fn phone_number(&self) -> &str {
        &self.phone_number
    }
    pub fn email_address(&self) -> &str {
        &self.email_address
    }
    pub fn address(&self) -> &Address {
        &self.address
    }
}

impl Profile {
    pub fn new_with_default(email_address: &str) -> Profile {
        Profile {
            email_address: String::from(email_address),
            ..Default::default()
        }
    }
    pub fn new(
        picture: Option<Metadata>,
        firstname: &str,
        lastname: &str,
        phone_number: &str,
        email_address: &str,
        address: Address,
    ) -> Self {
        Profile {
            picture,
            firstname: String::from(firstname),
            lastname: String::from(lastname),
            phone_number: String::from(phone_number),
            email_address: String::from(email_address),
            address,
        }
    }
    pub fn set_picture(self, picture: Metadata) -> Self {
        Profile {
            picture: Some(picture),
            ..self
        }
    }
    pub fn set_firstname(self, firstname: String) -> Self {
        Profile { firstname, ..self }
    }
    pub fn set_lastname(self, lastname: String) -> Self {
        Profile { lastname, ..self }
    }
    pub fn set_phone_number(self, phone_number: String) -> Self {
        Profile {
            phone_number,
            ..self
        }
    }
    pub fn set_email_address(self, email_address: String) -> Self {
        Profile {
            email_address,
            ..self
        }
    }
    pub fn set_address(self, address: Address) -> Self {
        Profile { address, ..self }
    }
}

#[derive(PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Address {
    street: String,
    number: String,
    po_box: String,
    municipality: String,
    province: String,
    country: String,
}

impl Address {
    pub fn set_street(self, street: String) -> Self {
        Address { street, ..self }
    }
    pub fn set_number(self, number: String) -> Self {
        Address { number, ..self }
    }
    pub fn set_po_box(self, po_box: String) -> Self {
        Address { po_box, ..self }
    }
    pub fn set_municipality(self, municipality: String) -> Self {
        Address {
            municipality,
            ..self
        }
    }
    pub fn set_province(self, province: String) -> Self {
        Address { province, ..self }
    }
    pub fn set_country(self, country: String) -> Self {
        Address { country, ..self }
    }
}

impl Address {
    pub fn street(&self) -> &str {
        &self.street
    }
    pub fn number(&self) -> &str {
        &self.number
    }
    pub fn po_box(&self) -> &str {
        &self.po_box
    }
    pub fn municipality(&self) -> &str {
        &self.municipality
    }
    pub fn province(&self) -> &str {
        &self.province
    }
    pub fn country(&self) -> &str {
        &self.country
    }
}

impl Address {
    pub fn new(
        street: &str,
        number: &str,
        po_box: &str,
        municipality: &str,
        province: &str,
        country: &str,
    ) -> Self {
        Address {
            street: String::from(street),
            number: String::from(number),
            po_box: String::from(po_box),
            municipality: String::from(municipality),
            province: String::from(province),
            country: String::from(country),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::user::{Address, Profile, User};
    use crate::{WithJsonProcessor, WithMetadata};

    #[test]
    fn test_user_creation() {
        let profile = Profile::new(
            Some(Default::default()),
            "nordine",
            "bittich",
            "(0032)0444/999.99.33",
            "nordine@keke.com",
            Address::new("pangaert", "20", "19", "Ganshoren", "Bxl", "Belgium"),
        );
        let user = User::new("nickk", vec!["user", "admin"], "xxxx", profile);
        assert_eq!("nickk", user.nickname());
        assert_eq!("xxxx", user.password());
        assert!(!user.id().is_empty());
        assert_eq!("nordine", user.profile().firstname());
        assert_eq!("bittich", user.profile().lastname());
        assert_eq!("(0032)0444/999.99.33", user.profile().phone_number());

        let roles = &user.roles();
        assert_eq!(&vec!["user", "admin"], roles);
        let mut user = user;
        user.add_role("super_admin");
        assert_eq!(vec!["user", "admin", "super_admin"], user.roles());
        user.remove_role("super_admin");
        assert_eq!(vec!["user", "admin"], user.roles());
        let json = user.to_json().unwrap();
        println!("{}", json);
        let user = User::from_json(json.as_str()).unwrap();

        let mut user: Box<dyn WithMetadata> = Box::new(user);
        user.domain_metadata_mut().update_metadata();
        assert_eq!(&Some(1), user.domain_metadata_mut().version());
        let json = user.domain_metadata_mut().to_json();
        println!("{}", json.unwrap());
    }
}
