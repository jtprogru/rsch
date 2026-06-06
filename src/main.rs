//! # rsch — RuSt CLI Helper
//!
//! Маленький консольный генератор паролей. Образовательный проект на Rust:
//! идиоматичный CLI на [`clap`], криптостойкий RNG из крейта [`rand`],
//! гарантированное покрытие всех выбранных категорий символов и Fisher–Yates
//! shuffle, чтобы итоговый пароль не имел предсказуемой структуры.
//!
//! ## Алфавит
//!
//! По умолчанию пароль состоит из латинских букв в обоих регистрах
//! (`A–Z`, `a–z`). Дополнительно можно подмешать цифры (`-d` / `--digits`) и
//! спецсимволы (`-s` / `--specials`). Подробный набор спецсимволов — в
//! константе [`SPECIALS`].
//!
//! ## Гарантии
//!
//! * Если длина пароля не меньше числа выбранных категорий, итоговая строка
//!   гарантированно содержит хотя бы один символ из **каждой** включённой
//!   категории. Это важно для систем, валидирующих сложность пароля
//!   («должна быть минимум одна цифра», «минимум один спецсимвол» и т.п.).
//! * Если длина меньше числа категорий, гарантии нет — алгоритм просто
//!   заполняет строку случайными символами из общего алфавита.
//! * После генерации позиции символов перемешиваются Fisher–Yates,
//!   чтобы «обязательные» символы не оказались всегда в начале.
//!
//! ## Источник случайности
//!
//! Используется [`rand::rng()`] — это `ThreadRng`, который в `rand 0.9`
//! построен на ChaCha12 и засеян из ОС-генератора. Этого достаточно для
//! генерации паролей; явный `OsRng` не нужен.
//!
//! ## Поведение CLI
//!
//! Сам пароль печатается в **stdout**, подпись `Password: ` — в **stderr**.
//! Это позволяет безопасно использовать `rsch` в пайпах:
//!
//! ```text
//! $ rsch -l 32 -d -s | pbcopy   # в буфер попадёт только пароль
//! ```
//!
//! ## Примеры запуска
//!
//! ```text
//! $ rsch                  # 24 символа, только буквы
//! $ rsch -l 16 -d         # 16 символов с цифрами
//! $ rsch -l 32 -d -s      # 32 символа со всем доступным алфавитом
//! $ rsch --version
//! $ rsch --help
//! ```

#![warn(missing_docs)]

use clap::Parser;
use rand::RngExt;

/// Заглавные латинские буквы. ASCII-only — благодаря этому корректна
/// побайтовая индексация в [`pass_gen`].
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Строчные латинские буквы. ASCII-only.
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

/// Арабские цифры. Подмешиваются при `-d` / `--digits`.
const DIGITS: &[u8] = b"0123456789";

/// Набор спецсимволов. Подмешиваются при `-s` / `--specials`. Выбран
/// подмножество ASCII-пунктуации, чтобы пароль был безопасен в большинстве
/// систем (без управляющих символов и без пробела).
const SPECIALS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:',.<>?/";

/// Parsed command-line arguments.
//
// Длинное пояснение для читателей кода — обычным `//`-комментарием, чтобы
// `clap` derive не подхватил его как `long_about` и не подменил строку из
// `#[command(about = ...)]` в выводе `rsch --help`. По той же причине ниже
// явно проставлен `long_about = None` — это страховка на случай, если кто-то
// в будущем добавит многострочный `///` сюда.
//
// Doc-комментарии на ПОЛЯХ структуры, напротив, безопасны: `clap` использует
// их как текст справки конкретных опций, и они написаны коротко по-английски.
#[derive(Parser)]
#[command(
    name = "rsch",
    version,
    about = "RuSt CLI Helper — password generator",
    long_about = None
)]
pub struct Cli {
    /// Password length in characters (default: 24)
    #[arg(short, long, default_value_t = 24)]
    length: usize,

    /// Include ASCII digits (0–9) in the alphabet
    #[arg(short = 'd', long = "digits")]
    include_digits: bool,

    /// Include ASCII special characters in the alphabet
    #[arg(short = 's', long = "specials")]
    include_specials: bool,
}

/// Сгенерировать пароль заданной длины.
///
/// # Аргументы
///
/// * `length` — длина пароля в символах. При `0` возвращается пустая строка.
/// * `include_digits` — добавить в алфавит цифры `0–9`.
/// * `include_specials` — добавить в алфавит спецсимволы (см. [`SPECIALS`]).
///
/// # Возвращает
///
/// `String` длины ровно `length` символов. Так как алфавит целиком ASCII,
/// число байт и число `char` совпадают.
///
/// # Гарантии
///
/// * Если `length` ≥ числа включённых категорий, в пароле есть хотя бы один
///   символ из **каждой** включённой категории (минимум по одной заглавной
///   и одной строчной букве; при `include_digits` — минимум одна цифра;
///   при `include_specials` — минимум один спецсимвол).
/// * Если `length` меньше числа категорий — гарантии нет, пароль просто
///   заполняется из общего алфавита.
/// * Позиции символов рандомизированы Fisher–Yates: «обязательные» символы
///   не залипают в начале строки.
///
/// # Источник случайности
///
/// Используется потоковый RNG из [`rand::rng()`] (ChaCha12, засеянный из ОС
/// в `rand 0.9`). Для генерации паролей этого достаточно.
///
/// # Паники
///
/// Не паникует на корректных входных данных. Внутренний `String::from_utf8`
/// формально может вернуть ошибку, но это невозможно: алфавит целиком ASCII,
/// что является инвариантом, проверяемым тестами.
///
/// # Пример
///
/// ```ignore
/// let pwd = pass_gen(16, true, true);
/// assert_eq!(pwd.len(), 16);
/// assert!(pwd.chars().any(|c| c.is_ascii_digit()));
/// ```
pub fn pass_gen(length: usize, include_digits: bool, include_specials: bool) -> String {
    // Собираем список включённых категорий. UPPER + LOWER подключены всегда,
    // цифры и спецсимволы — по флагам.
    let mut categories: Vec<&[u8]> = vec![UPPER, LOWER];
    if include_digits {
        categories.push(DIGITS);
    }
    if include_specials {
        categories.push(SPECIALS);
    }

    // Общий алфавит — конкатенация всех включённых категорий. Используется
    // для добора символов после того, как мы взяли по одному из каждой
    // категории.
    let alphabet: Vec<u8> = categories.iter().flat_map(|c| c.iter()).copied().collect();

    // ChaCha12 CSPRNG из rand 0.9, засеянный из ОС.
    let mut rng = rand::rng();

    // Шаг 1: если длины хватает, берём по одному обязательному символу из
    // каждой категории. Это и есть гарантия покрытия.
    let mut password: Vec<u8> = if length >= categories.len() {
        categories
            .iter()
            .map(|c| c[rng.random_range(0..c.len())])
            .collect()
    } else {
        Vec::with_capacity(length)
    };

    // Шаг 2: добиваем оставшуюся длину случайными символами из общего
    // алфавита.
    while password.len() < length {
        password.push(alphabet[rng.random_range(0..alphabet.len())]);
    }

    // Шаг 3: Fisher–Yates shuffle. Без него «обязательные» символы всегда
    // оказывались бы в первых N позициях, что снижает энтропию для атакующего,
    // знающего алгоритм.
    for i in (1..password.len()).rev() {
        let j = rng.random_range(0..=i);
        password.swap(i, j);
    }

    // Безопасно: весь алфавит — ASCII (инвариант констант), значит валидный
    // UTF-8 гарантирован.
    String::from_utf8(password).expect("alphabet is ASCII")
}

/// Точка входа: парсим аргументы, генерируем пароль, печатаем его в stdout.
///
/// Подпись `Password: ` уходит в stderr, чтобы stdout оставался пригодным
/// для пайпов (`rsch | pbcopy`, `rsch | xclip` и т.п.).
fn main() {
    let args = Cli::parse();
    let password = pass_gen(args.length, args.include_digits, args.include_specials);
    eprint!("Password: ");
    println!("{password}");
}

#[cfg(test)]
mod tests {
    //! Юнит-тесты [`pass_gen`].
    //!
    //! Группы инвариантов:
    //!
    //! * **Длина** — итоговая строка всегда содержит ровно столько символов,
    //!   сколько запросили.
    //! * **Покрытие категорий** — если флаги включены, соответствующие
    //!   символы реально встречаются в результате. Проверяется в цикле
    //!   (50 итераций), чтобы поймать редкие непокрытия, если они появятся
    //!   при изменении алгоритма.
    //! * **Граничные случаи** — нулевая длина, минимальная длина при всех
    //!   категориях.

    use super::*;

    /// Базовый случай: длина = 16, только буквы. Проверяем, что
    /// результат содержит ровно 16 символов.
    #[test]
    fn length_matches_default_charset() {
        let password = pass_gen(16, false, false);
        assert_eq!(password.len(), 16);
    }

    /// Длина должна совпадать с запрошенной независимо от числа включённых
    /// категорий. Регрессионный тест: ранее наивная реализация могла
    /// «съесть» лишние слоты на обязательные символы.
    #[test]
    fn length_matches_with_all_categories() {
        let password = pass_gen(32, true, true);
        assert_eq!(password.len(), 32);
    }

    /// UPPER и LOWER включены всегда — в пароле длины 32 практически
    /// невозможно случайно не получить обе категории. Тест защищает от
    /// случайного удаления одной из категорий из дефолтного набора.
    #[test]
    fn contains_upper_and_lower() {
        let password = pass_gen(32, false, false);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
    }

    /// Главная гарантия флага `-d`: при включённых цифрах хотя бы одна
    /// цифра обязана быть в пароле. Прогоняем 50 раз, чтобы поймать
    /// статистические нарушения инварианта.
    #[test]
    fn always_contains_digit_when_requested() {
        for _ in 0..50 {
            let password = pass_gen(8, true, false);
            assert!(
                password.chars().any(|c| c.is_ascii_digit()),
                "no digit in {password}"
            );
        }
    }

    /// Симметричная проверка для `-s`: при включённых спецсимволах хотя бы
    /// один спецсимвол обязан быть в пароле.
    #[test]
    fn always_contains_special_when_requested() {
        for _ in 0..50 {
            let password = pass_gen(8, false, true);
            assert!(
                password.chars().any(|c| SPECIALS.contains(&(c as u8))),
                "no special in {password}"
            );
        }
    }

    /// Совместное покрытие при всех включённых категориях: в пароле длины
    /// 8 (минимум для четырёх категорий) одновременно присутствуют upper,
    /// lower, digit и special. Это самый строгий вариант, на котором обычно
    /// проседают наивные реализации.
    #[test]
    fn contains_all_categories_when_all_enabled() {
        for _ in 0..50 {
            let password = pass_gen(8, true, true);
            assert!(password.chars().any(|c| c.is_ascii_uppercase()));
            assert!(password.chars().any(|c| c.is_ascii_lowercase()));
            assert!(password.chars().any(|c| c.is_ascii_digit()));
            assert!(password.chars().any(|c| SPECIALS.contains(&(c as u8))));
        }
    }

    /// Граничный случай: `length = 0`. Функция должна возвращать пустую
    /// строку, а не паниковать и не зацикливаться.
    #[test]
    fn zero_length_returns_empty_string() {
        let password = pass_gen(0, true, true);
        assert_eq!(password, "");
    }
}
