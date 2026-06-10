use crate::types::{BxNativeFunction, BxVM, BxValue};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

static CURRENT_LOCALE: OnceLock<Mutex<String>> = OnceLock::new();

fn locale_store() -> &'static Mutex<String> {
    CURRENT_LOCALE.get_or_init(|| Mutex::new("en_US".to_string()))
}

pub fn register_i18n_bifs(bifs: &mut HashMap<String, BxNativeFunction>) {
    bifs.insert("getLocale".to_string(), get_locale as BxNativeFunction);
    bifs.insert("setLocale".to_string(), set_locale as BxNativeFunction);
    bifs.insert("clearLocale".to_string(), clear_locale as BxNativeFunction);
    bifs.insert("currencyFormat".to_string(), currency_format as BxNativeFunction);
    bifs.insert("getLocaleDisplayName".to_string(), get_locale_display_name as BxNativeFunction);
    bifs.insert("getLocaleInfo".to_string(), get_locale_info as BxNativeFunction);
    bifs.insert("isCurrency".to_string(), is_currency as BxNativeFunction);
    bifs.insert("parseCurrency".to_string(), parse_currency as BxNativeFunction);
}

fn get_locale(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let locale = locale_store().lock().unwrap().clone();
    Ok(BxValue::new_ptr(vm.string_new(locale)))
}

fn set_locale(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("setLocale() expects 1 argument".to_string());
    }
    let locale = vm.to_string(args[0]);
    *locale_store().lock().unwrap() = locale.clone();
    Ok(BxValue::new_ptr(vm.string_new(locale)))
}

fn clear_locale(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    *locale_store().lock().unwrap() = "en_US".to_string();
    Ok(BxValue::new_null())
}

fn currency_format(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("currencyFormat() expects 1 argument".to_string());
    }
    let num = if args[0].is_number() {
        args[0].as_number()
    } else {
        let s = vm.to_string(args[0]);
        s.trim().parse::<f64>().map_err(|_| format!("currencyFormat() expected number, got '{}'", s))?
    };
    let negative = num < 0.0;
    let abs = num.abs();
    let whole = abs as u64;
    let cents = ((abs - whole as f64) * 100.0).round() as u64;
    let whole_str = format!("{}", whole);
    let mut formatted = String::new();
    for (i, ch) in whole_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(ch);
    }
    let formatted: String = formatted.chars().rev().collect();
    let result = if negative {
        format!("-${}.{:02}", formatted, cents)
    } else {
        format!("${}.{:02}", formatted, cents)
    };
    Ok(BxValue::new_ptr(vm.string_new(result)))
}

fn get_locale_display_name(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("getLocaleDisplayName() expects 1 argument".to_string());
    }
    let locale = vm.to_string(args[0]);
    Ok(BxValue::new_ptr(vm.string_new(locale)))
}

fn get_locale_info(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("getLocaleInfo() expects 1 argument".to_string());
    }
    let locale = vm.to_string(args[0]);
    let parts: Vec<&str> = locale.split('_').collect();
    let language = parts.first().copied().unwrap_or(&locale);
    let country = parts.get(1).copied().unwrap_or("");
    let id = vm.struct_new();
    let lang_ptr = vm.string_new(language.to_string());
    let country_ptr = vm.string_new(country.to_string());
    let display_lang_ptr = vm.string_new(language.to_string());
    let display_country_ptr = vm.string_new(country.to_string());
    vm.struct_set(id, "language", BxValue::new_ptr(lang_ptr));
    vm.struct_set(id, "country", BxValue::new_ptr(country_ptr));
    vm.struct_set(id, "displayLanguage", BxValue::new_ptr(display_lang_ptr));
    vm.struct_set(id, "displayCountry", BxValue::new_ptr(display_country_ptr));
    Ok(BxValue::new_ptr(id))
}

fn is_currency(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("isCurrency() expects 1 argument".to_string());
    }
    let s = vm.to_string(args[0]).trim().to_string();
    let looks_like = s.starts_with('$')
        || s.starts_with('€')
        || s.starts_with('£')
        || s.starts_with('¥')
        || s.ends_with("USD")
        || s.ends_with("EUR")
        || s.ends_with("GBP")
        || s.ends_with("JPY");
    Ok(BxValue::new_bool(looks_like))
}

fn parse_currency(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("parseCurrency() expects 1 argument".to_string());
    }
    let s = vm.to_string(args[0]);
    let cleaned: String = s.chars().filter(|c| *c == '-' || *c == '.' || c.is_ascii_digit()).collect();
    let num = cleaned.trim().parse::<f64>().map_err(|_| format!("parseCurrency() cannot parse '{}'", s))?;
    Ok(BxValue::new_number(num))
}
