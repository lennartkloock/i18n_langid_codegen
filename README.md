# `i18n_langid_codegen`

Function-like proc macro for internationalization

Generates structs and functions from a set of YAML files with support for
the [`unic-langid` crate](https://crates.io/crates/unic-langid).

Inspired by [`i18n_codegen` crate](https://crates.io/crates/i18n_codegen).

## How to use it?

Your YAML files:

```yaml
# locales/en.default.yml
hello: Hello World!

# locales/de.yml
hello: Hallo Welt!
```

Your Rust code:

```rust
mod i18n {
    i18n_langid_codegen::i18n!("locales");
}

fn main() {
    // Get single key
    assert_eq!("Hello World!", i18n::I18n::en().hello);
    assert_eq!("Hallo Welt!", i18n::I18n::de().hello);
    
    // Get the right struct instance by language identifier
    let id = unic_langid::langid!("de");
    let de = I18n::from_lang_id(id).unwrap_or_default();
    assert_eq!("Hallo Welt!", de.hello);
}
```

## Full Example

### Add dependencies

```shell
cargo add unic-langid
cargo add i18n_langid_codegen
```

### Add macro call to your code

```rust
mod i18n {
    i18n_langid_codegen::i18n!("locales");
}
```

### Files

Consider the following file tree.

```
├── ...
├── Cargo.toml
├── locales
│   ├── de.yml
│   └── en.default.yml
├── src
│   └── ...
└── ...
```

Content of `locales/en.default.yml`:

```yaml
hello: Hello World!
login_form:
    email: Email
    password: Password
    button: Log In
```

Content of `locales/de.yml`:

```yaml
hello: Hallo Welt!
login_form:
    password: Passwort
    button: Anmelden
```

Note that `login_form.email` is not included in the German translation. In this case the value from the file ending
in `.default.yml` is used.

### What the `i18n` macro generates

```rust
#[derive(Debug)]
pub struct I18n {
    pub lang_id: unic_langid::LanguageIdentifier,
    pub hello: &'static str,
    pub login_form: LoginForm,
}

#[derive(Debug)]
pub struct LoginForm {
    pub email: &'static str,
    pub password: &'static str,
    pub button: &'static str,
}

impl I18n {
    pub fn from_lang_id(
        lang_id: &unic_langid::LanguageIdentifier,
    ) -> Option<Self> {
        match lang_id.to_string().as_str() {
            "en" => Some(Self::en()),
            "de" => Some(Self::de()),
            _ => None,
        }
    }

    pub fn en() -> Self {
        Self {
            lang_id: unic_langid::LanguageIdentifier::from_str("en").unwrap(),
            hello: "Hello World!",
            login_form: LoginForm {
                email: "Email",
                password: "Password",
                button: "Log In",
            },
        }
    }

    pub fn de() -> Self {
        Self {
            lang_id: unic_langid::LanguageIdentifier::from_str("de").unwrap(),
            hello: "Hallo Welt!",
            login_form: LoginForm {
                email: "Email",
                password: "Passwort",
                button: "Anmelden",
            },
        }
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::en()
    }
}
```
