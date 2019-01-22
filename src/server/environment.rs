
use dotenv::dotenv;
use std::env;

fn get_var(var: &'static str) -> String {
    dotenv().expect("could not parse dotenv file");

    env::var(var).expect(&format!("`{}` must be set", var))
}

pub struct Env;

impl Env {

    pub fn www_path() -> String {
        get_var("WWW_PATH")
    }

    pub fn is_secure() -> bool {
        let is_secure = get_var("SECURE");
        is_secure == "true"
    }

    pub fn domain() -> String {
        get_var("SERVER_DOMAIN")
    }

    pub fn server_addr() -> String {
        let server_addr = get_var("SERVER_ADDR");
        let server_port = get_var("SERVER_PORT");
        format!("{}:{}", server_addr, server_port)
    }

    pub fn secret_key() -> String {
        get_var("SECRET_KEY")
    }

    pub fn ssl_cert_privkey_path() -> String {
        get_var("SSL_CERT_PRIVKEY_PATH")
    }
    pub fn ssl_cert_fullchain_path() -> String {
        get_var("SSL_CERT_FULLCHAIN_PATH")
    }
}
