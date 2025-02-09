use rand::Rng;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
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

fn main() {
    let args = Cli::from_args();
    let charset_upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let charset_lower = "abcdefghijklmnopqrstuvwxyz";
    let charset_digits = "0123456789";
    let charset_specials = "!@#$%^&*()-_=+[]{}|;:',.<>?/";

    let mut alphabet = String::from(charset_upper) + charset_lower;

    if args.include_digits {
        alphabet.push_str(charset_digits);
    }

    if args.include_specials {
        alphabet.push_str(charset_specials);
    }

    if alphabet.is_empty() {
        eprintln!("Error: alphabet is empty");
        return;
    }

    let mut rng = rand::rng();
    let password: String = (0..args.length)
        .map(|_| {
            let idx = rng.random_range(0..alphabet.len());
            alphabet.chars().nth(idx).unwrap()
        })
        .collect();

    println!("Password:\n{}", password)
}
