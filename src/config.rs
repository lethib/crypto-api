use std::{
    env::VarError,
    net::{IpAddr, Ipv4Addr},
};

const MIN_HMAC_SECRET_LENGTH: usize = 32;

pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    #[allow(dead_code)]
    pub hmac_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let host: IpAddr = match std::env::var("HOST") {
            Ok(host_value) => host_value.parse()?,
            Err(VarError::NotPresent) => Ipv4Addr::LOCALHOST.into(),
            Err(error) => return Err(error.into()),
        };

        let port: u16 = match std::env::var("PORT") {
            Ok(port_value) => port_value.parse()?,
            Err(VarError::NotPresent) => 3_000,
            Err(error) => return Err(error.into()),
        };

        let hmac_secret: String = {
            let secret = std::env::var("HMAC_SECRET")?;
            Self::verify_hmac_secret(secret)?
        };

        Ok(Self {
            host,
            port,
            hmac_secret,
        })
    }

    fn verify_hmac_secret(secret: String) -> Result<String, Box<dyn std::error::Error>> {
        if secret.len() < MIN_HMAC_SECRET_LENGTH {
            return Err(format!(
                "HMAC_SECRET must be at least {MIN_HMAC_SECRET_LENGTH} bytes long"
            )
            .into());
        }

        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, MIN_HMAC_SECRET_LENGTH};

    mod when_verifying_the_hmac_secret {
        use super::*;

        #[test]
        fn it_should_accept_a_secret_of_the_minimum_length() {
            let secret = "a".repeat(MIN_HMAC_SECRET_LENGTH);

            assert!(Config::verify_hmac_secret(secret).is_ok());
        }

        #[test]
        fn it_should_accept_a_secret_longer_than_the_minimum() {
            let secret = "a".repeat(MIN_HMAC_SECRET_LENGTH + 10);

            assert!(Config::verify_hmac_secret(secret).is_ok());
        }

        #[test]
        fn it_should_reject_a_secret_one_byte_too_short() {
            let secret = "a".repeat(MIN_HMAC_SECRET_LENGTH - 1);

            assert!(Config::verify_hmac_secret(secret).is_err());
        }
    }
}
