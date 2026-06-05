//! Интеграционные (end-to-end) тесты бинаря `rsch`.
//!
//! В отличие от юнит-тестов в `src/main.rs`, эти тесты собирают реальный
//! исполняемый файл через `assert_cmd::Command::cargo_bin` и запускают его
//! как чёрный ящик: проверяют код возврата, содержимое stdout/stderr и
//! поведение флагов. Это нужно, чтобы зафиксировать публичный контракт CLI
//! отдельно от внутренней логики генерации — например, инвариант «stdout
//! содержит только сам пароль» невозможно проверить юнит-тестом.

use assert_cmd::Command;
use predicates::prelude::*;

/// Вернуть готовую команду для запуска бинаря `rsch`.
///
/// Использует `cargo_bin`, который автоматически указывает на собранный
/// в `target/debug/` бинарь текущего крейта. Это правильный способ
/// тестировать CLI: он работает и локально, и в CI, без захардкоженных путей.
fn rsch() -> Command {
    Command::cargo_bin("rsch").expect("binary built")
}

/// Без аргументов длина пароля должна быть 24 (значение по умолчанию,
/// прописанное в [`crate::Cli`] через `default_value_t = 24`).
///
/// `trim_end()` срезает завершающий `\n`, который добавляет `println!`.
#[test]
fn default_length_is_24() {
    rsch()
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| s.trim_end().len() == 24));
}

/// `-l 40` должно давать пароль ровно длины 40. Регрессия: если в будущем
/// `length` начнёт интерпретироваться как «минимум», тест это поймает.
#[test]
fn custom_length_is_respected() {
    rsch()
        .args(["-l", "40"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| s.trim_end().len() == 40));
}

/// Контракт пайпа: в stdout не должно быть подписи `"Password"`, а в stderr —
/// должна. Это позволяет писать `rsch | pbcopy` и получать в буфере только
/// сам пароль без лейбла.
#[test]
fn stdout_contains_only_password() {
    rsch()
        .assert()
        .success()
        .stdout(predicate::str::contains("Password").not())
        .stderr(predicate::str::contains("Password"));
}

/// Флаг `-d` (digits) должен реально приводить к появлению цифры в
/// сгенерированном пароле. Это end-to-end проверка гарантии покрытия из
/// `pass_gen`, поднятая до уровня CLI.
#[test]
fn digits_flag_produces_digit() {
    rsch()
        .args(["-l", "12", "-d"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            s.chars().any(|c| c.is_ascii_digit())
        }));
}

/// Симметричная проверка для `-s` (specials). Список спецсимволов
/// продублирован здесь намеренно — тест должен ловить расхождение между
/// объявленным CLI-контрактом и фактической константой `SPECIALS` в коде.
#[test]
fn specials_flag_produces_special() {
    let specials = "!@#$%^&*()-_=+[]{}|;:',.<>?/";
    rsch()
        .args(["-l", "12", "-s"])
        .assert()
        .success()
        .stdout(predicate::function(move |s: &str| {
            s.chars().any(|c| specials.contains(c))
        }));
}

/// `--help` должен успешно завершаться и упоминать «password generator»
/// (текст из `#[command(about = ...)]`). Тест защищает от случайной потери
/// описания при рефакторинге `Cli`.
#[test]
fn help_flag_works() {
    rsch()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("password generator"));
}

/// `--version` должен успешно завершаться и выводить имя крейта. Версия
/// автоматически берётся `clap`-ом из `Cargo.toml`, так что отдельной
/// проверки конкретной строки версии не делаем — это сделало бы тест
/// хрупким к каждому bump'у.
#[test]
fn version_flag_works() {
    rsch()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rsch"));
}
