/*
 * 
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SendCodeResponse {
    #[serde(rename = "found_account")]
    pub found_account: bool,
}

impl SendCodeResponse {
    pub fn new(found_account: bool) -> SendCodeResponse {
        SendCodeResponse {
            found_account,
        }
    }
}


