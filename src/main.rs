use rand::Rng;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Cli {
    /// Default password length 24
    #[structopt(short = "l", long, default_value = "24")]
    length: usize,

    /// Add digits in password
    #[structopt(short = "d", long = "digits")]
    include_digits: bool,

    /// Add special characters
    #[structopt(short = "s", long = "specials")]
    include_specials: bool,
}

pub fn pass_gen(length: usize, include_digits: bool, include_specials: bool) -> String {

    let charset_upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let charset_lower = "abcdefghijklmnopqrstuvwxyz";
    let charset_digits = "0123456789";
    let charset_specials = "!@#$%^&*()-_=+[]{}|;:',.<>?/";

    let mut alphabet = String::from(charset_upper) + charset_lower;

    if include_digits {
        alphabet.push_str(charset_digits);
    }

    if include_specials {
        alphabet.push_str(charset_specials);
    }

    if alphabet.is_empty() {
        eprintln!("Error: alphabet is empty");
        "";
    }

    let mut rng = rand::rng();
    return (0..length)
        .map(|_| {
            let idx = rng.random_range(0..alphabet.len());
            alphabet.chars().nth(idx).unwrap()
        })
        .collect();
}

fn main() {
    let args = Cli::from_args();
    let password = pass_gen(args.length, args.include_digits, args.include_specials);
    println!("Password:\n{}", password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_password_length() {
        let length = 16;
        let include_digits = false;
        let include_specials = false;
        let password = pass_gen(length, include_digits, include_specials);
        assert_eq!(password.len(), length, "Length password not equal got {} want {}", password.len(), length );
    }

    #[test]
    async fn test_password_include_digits() {
        let length = 32;
        let include_digits = true;
        let include_specials = false;
        let password = pass_gen(length, include_digits, include_specials);
        assert!(password.chars().any(|c| c.is_numeric()), "Password is not contains a digits");
    }

    #[test]
    async fn test_password_includes_specials() {
        let length = 16;
        let include_digits = false;
        let include_specials = true;
        let password = pass_gen(length, include_digits, include_specials);
        let specials = "!@#$%^&*()-_=+[]{}|;:',.<>?/";
        assert!(
            password.chars().any(|c| specials.contains(c)),
            "Password is not contains a specials",
        );
    }

    #[test]
    async fn test_password_includes_upper_and_lower_case() {
        let length = 32;
        let include_digits = false;
        let include_specials = false;
        let password = pass_gen(length, include_digits, include_specials);
        assert!(
            password.chars().any(|c| c.is_uppercase()),
            "Password is not contains a uppercase",
        );
        assert!(
            password.chars().any(|c| c.is_lowercase()),
            "Password is not contains a lowercase",
        );
    }

    #[test]
    async fn test_empty_charset_panic() {
        let length = 16;
        let include_digits = false;
        let include_specials = false;
        let result = std::panic::catch_unwind(|| pass_gen(length, include_digits, include_specials));
        assert!(result.is_ok(), "Function don't panic");
    }
}
