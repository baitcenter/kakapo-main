
#[derive(Debug, Fail, Serialize, PartialEq, Eq)]
pub enum Error {
    #[fail(display = "Expiry date for the refresh token is too short")]
    ExpiryDateTooShort,
    #[fail(display = "Could not encode jwt")]
    EncodeError,
}
