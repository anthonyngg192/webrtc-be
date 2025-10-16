// pub mod hex_string_as_string {
//     use mongodb::bson::oid::ObjectId;
//     use serde::{ser::Serialize, Deserialize, Deserializer, Serializer};

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let object_id = ObjectId::deserialize(deserializer)?;
//         Ok(object_id.to_hex())
//     }

//     pub fn serialize<S: Serializer>(val: &str, serializer: S) -> Result<S::Ok, S::Error> {
//         match ObjectId::parse_str(val) {
//             Ok(object_id) => object_id.serialize(serializer),
//             Err(_) => Err(serde::ser::Error::custom(format!(
//                 "Invalid ObjectId: {}",
//                 val
//             ))),
//         }
//     }
// }

pub mod hex_string_as_string {
    use mongodb::bson::oid::ObjectId;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object_id = ObjectId::deserialize(deserializer)?;
        Ok(object_id.to_hex())
    }

    pub fn serialize<S: Serializer>(val: &str, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(val)
    }
}
