// --- Get User
// {
//   "FirstName": String,           # If Preferred First Name is set then use for
//                                  #    FirstName field otherwise use First Name from Banner
//   "MiddleName": String,          # If Preferred First Name is set then leave MiddleName
//                                  #    field blank otherwise use Middle Name from Banner
//   "LastName": String,            # Use Last Name
//   "UserName": String,            # Use federated namespaced id (This field is used in d2l
//                                  #    api to retrieve an individual's user information
//   "OrgDefinedId": String|null,   # Use A-Num/bannerid (This field is used for group searches,
//                                  #    even though we use it as a unique identifier)
//   "ExternalEmail": String|null,  # Use alias for user's email (This field is used in group
//                                  #    searches that match exactly)
//
//   "Activation": {"IsActive": true},
//
//   "UserId": usize                # this is assigned by d2l upon user creation
//
//   "UniqueIdentifier": String,    # this is assigned by d2l upon user creation
//   "OrgId": usize,                # this is assigned by d2l upon user creation
//   "DisplayName": String,         # this is assigned by d2l upon user creation
// }
//
// --- Create User
// {
//   "FirstName": String,
//   "MiddleName": String,
//   "LastName": "String,
//   "UserName": String,
//   "OrgDefinedId": String,
//   "ExternalEmail": String,
//
//   "RoleId": String, # "109"="Instructor", "110"="Student"
//   "IsActive": true,
//   "SendCreationEmail": false
// }
//
// --- Update User
// {
//   "FirstName": String,
//   "MiddleName": String,
//   "LastName": String,
//   "UserName": String,
//   "OrgDefinedId": String,
//   "ExternalEmail": String,
//
//   "Activation": {"IsActive": true}
// }

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Activation {
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UserBase {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub user_name: String,
    pub org_defined_id: Option<String>,
    pub external_email: Option<String>,
}

// Read(GET Method) or Update(POST Method) User
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UserReadOrUpdate {
    #[serde(flatten)]
    pub user_base: UserBase,
    #[serde(skip_serializing)]
    pub user_id: usize,
    pub activation: Activation,
}

// Create(POST Method) User
#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UserCreate {
    #[serde(flatten)]
    pub user_base: UserBase,
    pub role_id: String,
    pub is_active: bool,
    pub send_creation_email: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {

        let data = r#"{"FirstName":"John","MiddleName":"","LastName":"Doe","UserName":"j_d1@txstate.edu","OrgDefinedId":"A00000000","ExternalEmail":"jdoe@txstate.edu","OrgId":6606,"UserId":100,"Activation":{"IsActive":true},"DisplayName":"John Doe","UniqueIdentifier":"j_d1@txstate.edu"}"#;
        let actual = serde_json::from_str(&data).unwrap();
        let expected = UserReadOrUpdate {
            user_base: UserBase {
                first_name: "John".to_string(),
                middle_name: "".to_string(),
                last_name: "Doe".to_string(),
                user_name: "j_d1@txstate.edu".to_string(),
                org_defined_id: Some("A00000000".to_string()),
                external_email: Some("jdoe@txstate.edu".to_string()),
            },
            user_id: 100,
            activation: Activation{is_active: true},
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_update() {
        let data = UserReadOrUpdate {
            user_base: UserBase {
                first_name: "John".to_string(),
                middle_name: "".to_string(),
                last_name: "Doe".to_string(),
                user_name: "j_d1@txstate.edu".to_string(),
                org_defined_id: Some("A00000000".to_string()),
                external_email: Some("jdoe@txstate.edu".to_string()),
            },
            user_id: 100,
            activation: Activation{is_active: true},
        };
        let expected = r#"{"FirstName":"John","MiddleName":"","LastName":"Doe","UserName":"j_d1@txstate.edu","OrgDefinedId":"A00000000","ExternalEmail":"jdoe@txstate.edu","Activation":{"IsActive":true}}"#;
        let actual = serde_json::to_string(&data).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_create() {
        let data = UserCreate {
            user_base: UserBase {
                first_name: "John".to_string(),
                middle_name: "".to_string(),
                last_name: "Doe".to_string(),
                user_name: "j_d1@txstate.edu".to_string(),
                org_defined_id: Some("A00000000".to_string()),
                external_email: Some("jdoe@txstate.edu".to_string()),
            },
            role_id: "109".to_string(),
            is_active: true,
            send_creation_email: false,
        };
        let expected = r#"{"FirstName":"John","MiddleName":"","LastName":"Doe","UserName":"j_d1@txstate.edu","OrgDefinedId":"A00000000","ExternalEmail":"jdoe@txstate.edu","RoleId":"109","IsActive":true,"SendCreationEmail":false}"#;
        let actual = serde_json::to_string(&data).unwrap();
        assert_eq!(expected, actual);
    }
}
