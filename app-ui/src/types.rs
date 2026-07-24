#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Page {
  #[default]
  Recommend,
  Install,
  /// Local library: builtin + downloaded/installed packages.
  Library,
  Restore,
  Settings,
}

impl Page {
  /// Stable analytics / routing id (snake_case).
  pub fn analytics_id(self) -> &'static str {
    match self {
      Page::Recommend => "recommend",
      Page::Install => "install",
      Page::Library => "library",
      Page::Restore => "restore",
      Page::Settings => "settings",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Locale {
  ZhHans,
  ZhHant,
  #[default]
  EnUs,
  JaJp,
  KoKr,
  DeDe,
  EsEs,
}

impl Locale {
  pub const ALL: [Locale; 7] = [
    Locale::ZhHans,
    Locale::ZhHant,
    Locale::EnUs,
    Locale::JaJp,
    Locale::KoKr,
    Locale::DeDe,
    Locale::EsEs,
  ];

  pub fn code(self) -> &'static str {
    match self {
      Locale::ZhHans => "zh-Hans",
      Locale::ZhHant => "zh-Hant",
      Locale::EnUs => "en-US",
      Locale::JaJp => "ja-JP",
      Locale::KoKr => "ko-KR",
      Locale::DeDe => "de-DE",
      Locale::EsEs => "es-ES",
    }
  }

  pub fn label(self) -> &'static str {
    match self {
      Locale::ZhHans => "简体中文",
      Locale::ZhHant => "繁體中文",
      Locale::EnUs => "English",
      Locale::JaJp => "日本語",
      Locale::KoKr => "한국어",
      Locale::DeDe => "Deutsch",
      Locale::EsEs => "Español",
    }
  }

  pub fn from_code(code: &str) -> Self {
    match code {
      "zh-Hans" | "zh-CN" | "zh-Simple" => Locale::ZhHans,
      "zh-Hant" | "zh-TW" | "zh-Tradition" => Locale::ZhHant,
      "ja-JP" | "ja" | "jp" => Locale::JaJp,
      "ko-KR" | "ko" | "kr" => Locale::KoKr,
      "de-DE" | "de" => Locale::DeDe,
      "es-ES" | "es" => Locale::EsEs,
      "en-US" | "en" | _ => Locale::EnUs,
    }
  }
}
