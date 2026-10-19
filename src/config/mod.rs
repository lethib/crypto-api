use std::{
    env::VarError,
    net::{IpAddr, Ipv4Addr},
};

mod error;
pub use error::ConfigError;

const MIN_HMAC_SECRET_LENGTH: usize = 32;

pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub hmac_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let host: IpAddr = match std::env::var("HOST") {
            Ok(host_value) => host_value
                .parse::<IpAddr>()
                .map_err(|err| ConfigError::Invalid {
                    var: "HOST",
                    reason: err.to_string(),
                })?,
            Err(VarError::NotPresent) => Ipv4Addr::LOCALHOST.into(),
            Err(error) => {
                return Err(ConfigError::Invalid {
                    var: "HOST",
                    reason: error.to_string(),
                });
            }
        };

        let port: u16 = match std::env::var("PORT") {
            Ok(port_value) => port_value
                .parse::<u16>()
                .map_err(|err| ConfigError::Invalid {
                    var: "PORT",
                    reason: err.to_string(),
                })?,
            Err(VarError::NotPresent) => 3_000,
            Err(error) => {
                return Err(ConfigError::Invalid {
                    var: "PORT",
                    reason: error.to_string(),
                });
            }
        };

        let hmac_secret = match std::env::var("HMAC_SECRET") {
            Ok(val) => Self::verify_hmac_secret(val)?,
            Err(VarError::NotPresent) => return Err(ConfigError::Missing("HMAC_SECRET")),
            Err(err) => {
                return Err(ConfigError::Invalid {
                    var: "HMAC_SECRET",
                    reason: err.to_string(),
                });
            }
        };

        Ok(Self {
            host,
            port,
            hmac_secret,
        })
    }

    fn verify_hmac_secret(secret: String) -> Result<String, ConfigError> {
        if secret.len() < MIN_HMAC_SECRET_LENGTH {
            return Err(ConfigError::SecretTooShort {
                min: MIN_HMAC_SECRET_LENGTH,
            });
        }

        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, ConfigError, MIN_HMAC_SECRET_LENGTH};

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

            assert!(matches!(
                Config::verify_hmac_secret(secret),
                Err(ConfigError::SecretTooShort {
                    min: MIN_HMAC_SECRET_LENGTH
                })
            ));
        }
    }
}
