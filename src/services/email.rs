

#[derive(Debug, Clone, Serialize)]
pub struct Email(String);
impl Email {
    pub fn new(email: String) -> Self {
        Email(email)
    }
}


#[derive(Debug, Fail, Serialize)]
pub enum EmailError {
    #[fail(display = "An unknown error occurred")]
    Unknown,
}


pub struct Emailer;

pub trait SendEmail {
    fn send_email(to: Email, text: String) -> Result<(), EmailError>;
}

impl SendEmail for Emailer {
    fn send_email(to: Email, text: String) -> Result<(), EmailError> {
        Ok(())
    }
}