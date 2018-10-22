const PRIMARY_LANGUAGE_MASK: u16 = 0x03FF;
const SUB_LANGUAGE_MASK: u16 = 0xFC00;

/// A language used to read string descriptors from USB devices.
///
/// A language consists of a primary language and a sub language. Primary languages are language
/// families, such as English or Spanish. Sub languages identify a dialect of the primary language.
/// The dialect may be based on regional differences (United States English compared to United
/// Kindgdom English), writing systems (Cyrillic compared to Latin), or age (Modern compared to
/// Traditional). Each primary language has its own set of sub languages.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Language {
    raw: u16,
}

impl Language {
    /// Returns the language's 16-bit `LANGID`.
    ///
    /// Each language's `LANGID` is defined by the USB forum
    /// (http://www.usb.org/developers/docs/USB_LANGIDs.pdf).
    pub fn lang_id(self) -> u16 {
        self.raw
    }

    /// Returns the primary language.
    pub fn primary_language(self) -> PrimaryLanguage {
        PrimaryLanguage::from_raw(self.raw)
    }

    /// Returns the sub language.
    pub fn sub_language(self) -> SubLanguage {
        SubLanguage::from_raw(self.primary_language(), self.raw)
    }
}

#[doc(hidden)]
pub fn from_lang_id(raw: u16) -> Language {
    Language { raw }
}

/// Primary language families.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PrimaryLanguage {
    Afrikaans,
    Albanian,
    Arabic,
    Armenian,
    Assamese,
    Azeri,
    Basque,
    Belarussian,
    Bengali,
    Bulgarian,
    Burmese,
    Catalan,
    Chinese,
    Croatian,
    Czech,
    Danish,
    Dutch,
    English,
    Estonian,
    Faeroese,
    Farsi,
    Finnish,
    French,
    Georgian,
    German,
    Greek,
    Gujarati,
    Hebrew,
    Hindi,
    Hungarian,
    Icelandic,
    Indonesian,
    Italian,
    Japanese,
    Kannada,
    Kashmiri,
    Kazakh,
    Konkani,
    Korean,
    Latvian,
    Lithuanian,
    Macedonian,
    Malay,
    Malayalam,
    Manipuri,
    Marathi,
    Nepali,
    Norwegian,
    Oriya,
    Polish,
    Portuguese,
    Punjabi,
    Romanian,
    Russian,
    Sanskrit,
    Serbian,
    Sindhi,
    Slovak,
    Slovenian,
    Spanish,
    Sutu,
    Swahili,
    Swedish,
    Tamil,
    Tatar,
    Telugu,
    Thai,
    Turkish,
    Ukrainian,
    Urdu,
    Uzbek,
    Vietnamese,

    HID,
    Other(u16),
}

impl PrimaryLanguage {
    fn from_raw(raw: u16) -> PrimaryLanguage {
        match raw & PRIMARY_LANGUAGE_MASK {
            0x0036 => PrimaryLanguage::Afrikaans,
            0x001C => PrimaryLanguage::Albanian,
            0x0001 => PrimaryLanguage::Arabic,
            0x002B => PrimaryLanguage::Armenian,
            0x004D => PrimaryLanguage::Assamese,
            0x002C => PrimaryLanguage::Azeri,
            0x002D => PrimaryLanguage::Basque,
            0x0023 => PrimaryLanguage::Belarussian,
            0x0045 => PrimaryLanguage::Bengali,
            0x0002 => PrimaryLanguage::Bulgarian,
            0x0055 => PrimaryLanguage::Burmese,
            0x0003 => PrimaryLanguage::Catalan,
            0x0004 => PrimaryLanguage::Chinese,
            0x001A => match raw & SUB_LANGUAGE_MASK {
                0x0400 => PrimaryLanguage::Croatian,
                _ => PrimaryLanguage::Serbian,
            },
            0x0005 => PrimaryLanguage::Czech,
            0x0006 => PrimaryLanguage::Danish,
            0x0013 => PrimaryLanguage::Dutch,
            0x0009 => PrimaryLanguage::English,
            0x0025 => PrimaryLanguage::Estonian,
            0x0038 => PrimaryLanguage::Faeroese,
            0x0029 => PrimaryLanguage::Farsi,
            0x000B => PrimaryLanguage::Finnish,
            0x000C => PrimaryLanguage::French,
            0x0037 => PrimaryLanguage::Georgian,
            0x0007 => PrimaryLanguage::German,
            0x0008 => PrimaryLanguage::Greek,
            0x0047 => PrimaryLanguage::Gujarati,
            0x000D => PrimaryLanguage::Hebrew,
            0x0039 => PrimaryLanguage::Hindi,
            0x000E => PrimaryLanguage::Hungarian,
            0x000F => PrimaryLanguage::Icelandic,
            0x0021 => PrimaryLanguage::Indonesian,
            0x0010 => PrimaryLanguage::Italian,
            0x0011 => PrimaryLanguage::Japanese,
            0x004B => PrimaryLanguage::Kannada,
            0x0060 => PrimaryLanguage::Kashmiri,
            0x003F => PrimaryLanguage::Kazakh,
            0x0057 => PrimaryLanguage::Konkani,
            0x0012 => PrimaryLanguage::Korean,
            0x0026 => PrimaryLanguage::Latvian,
            0x0027 => PrimaryLanguage::Lithuanian,
            0x002F => PrimaryLanguage::Macedonian,
            0x003E => PrimaryLanguage::Malay,
            0x004C => PrimaryLanguage::Malayalam,
            0x0058 => PrimaryLanguage::Manipuri,
            0x004E => PrimaryLanguage::Marathi,
            0x0061 => PrimaryLanguage::Nepali,
            0x0014 => PrimaryLanguage::Norwegian,
            0x0048 => PrimaryLanguage::Oriya,
            0x0015 => PrimaryLanguage::Polish,
            0x0016 => PrimaryLanguage::Portuguese,
            0x0046 => PrimaryLanguage::Punjabi,
            0x0018 => PrimaryLanguage::Romanian,
            0x0019 => PrimaryLanguage::Russian,
            0x004F => PrimaryLanguage::Sanskrit,
            0x0059 => PrimaryLanguage::Sindhi,
            0x001B => PrimaryLanguage::Slovak,
            0x0024 => PrimaryLanguage::Slovenian,
            0x000A => PrimaryLanguage::Spanish,
            0x0030 => PrimaryLanguage::Sutu,
            0x0041 => PrimaryLanguage::Swahili,
            0x001D => PrimaryLanguage::Swedish,
            0x0049 => PrimaryLanguage::Tamil,
            0x0044 => PrimaryLanguage::Tatar,
            0x004A => PrimaryLanguage::Telugu,
            0x001E => PrimaryLanguage::Thai,
            0x001F => PrimaryLanguage::Turkish,
            0x0022 => PrimaryLanguage::Ukrainian,
            0x0020 => PrimaryLanguage::Urdu,
            0x0043 => PrimaryLanguage::Uzbek,
            0x002A => PrimaryLanguage::Vietnamese,
            0x00FF => PrimaryLanguage::HID,
            n => PrimaryLanguage::Other(n),
        }
    }
}

/// Language dialects and writing systems.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubLanguage {
    Standard,
    Classic,
    Traditional,
    Modern,

    Algeria,            // arabic
    Argentina,          // spanish
    Australia,          // english
    Austria,            // german
    Bahrain,            // arabic
    Belgium,            // dutch, french
    Belize,             // english
    Bokmal,             // norwegian
    Bolivia,            // spanish
    Brazil,             // portuguese
    BruneiDarussalam,   // malay
    Canada,             // english, french
    Caribbean,          // english
    Chile,              // spanish
    China,              // chinese
    Colombia,           // spanish
    CostaRica,          // spanish
    Cyrillic,           // azeri, serbian, uzbek
    DominicanRepublic,  // spanish
    Ecuador,            // spanish
    Egypt,              // arabic
    ElSalvador,         // spanish
    Finland,            // swedish
    Guatemala,          // spanish
    Honduras,           // spanish
    HongKong,           // chinese
    India,              // kashmiri, nepali, urdu
    Iraq,               // arabic
    Ireland,            // english
    Jamaica,            // english
    Johab,              // korean
    Jordan,             // arabic
    Kuwait,             // arabic
    Latin,              // azeri, serbian, uzbek
    Lebanon,            // arabic
    Libya,              // arabic
    Liechtenstein,      // german
    Luxembourg,         // french, german
    Macau,              // chinese
    Malaysia,           // malay
    Mexico,             // spanish
    Monaco,             // french
    Morocco,            // arabic
    Netherlands,        // dutch
    NewZealand,         // english
    Nicaragua,          // spanish
    Nynorsk,            // norwegian
    Oman,               // arabic
    Pakistan,           // urdu
    Panama,             // spanish
    Paraguay,           // spanish
    Peru,               // spanish
    Philippines,        // english
    PuertoRico,         // spanish
    Qatar,              // arabic
    SaudiArabia,        // arabic
    Singapore,          // chinese
    SouthAfrica,        // english
    Switzerland,        // french, german, italian
    Syria,              // arabic
    Taiwan,             // chinese
    Trinidad,           // english
    Tunisia,            // arabic
    UnitedArabEmirates, // arabic
    UnitedKingdom,      // english
    UnitedStates,       // english
    Uruguay,            // spanish
    Venezuela,          // spanish
    Yemen,              // arabic
    Zimbabwe,           // english

    UsageDataDescriptor, // HID
    VendorDefined1,      // HID
    VendorDefined2,      // HID
    VendorDefined3,      // HID
    VendorDefined4,      // HID

    Other(u16),
}

impl SubLanguage {
    fn from_raw(language: PrimaryLanguage, raw: u16) -> SubLanguage {
        match language {
            PrimaryLanguage::Arabic => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::SaudiArabia,
                0x0800 => SubLanguage::Iraq,
                0x0C00 => SubLanguage::Egypt,
                0x1000 => SubLanguage::Libya,
                0x1400 => SubLanguage::Algeria,
                0x1800 => SubLanguage::Morocco,
                0x1C00 => SubLanguage::Tunisia,
                0x2000 => SubLanguage::Oman,
                0x2400 => SubLanguage::Yemen,
                0x2800 => SubLanguage::Syria,
                0x2C00 => SubLanguage::Jordan,
                0x3000 => SubLanguage::Lebanon,
                0x3400 => SubLanguage::Kuwait,
                0x3800 => SubLanguage::UnitedArabEmirates,
                0x3C00 => SubLanguage::Bahrain,
                0x4000 => SubLanguage::Qatar,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Azeri => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Latin,
                0x0800 => SubLanguage::Cyrillic,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Chinese => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Taiwan,
                0x0800 => SubLanguage::China,
                0x0C00 => SubLanguage::HongKong,
                0x1000 => SubLanguage::Singapore,
                0x1400 => SubLanguage::Macau,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Dutch => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Netherlands,
                0x0800 => SubLanguage::Belgium,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::English => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::UnitedStates,
                0x0800 => SubLanguage::UnitedKingdom,
                0x0C00 => SubLanguage::Australia,
                0x1000 => SubLanguage::Canada,
                0x1400 => SubLanguage::NewZealand,
                0x1800 => SubLanguage::Ireland,
                0x1C00 => SubLanguage::SouthAfrica,
                0x2000 => SubLanguage::Jamaica,
                0x2400 => SubLanguage::Caribbean,
                0x2800 => SubLanguage::Belize,
                0x2C00 => SubLanguage::Trinidad,
                0x3000 => SubLanguage::Zimbabwe,
                0x3400 => SubLanguage::Philippines,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::French => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Standard,
                0x0800 => SubLanguage::Belgium,
                0x0C00 => SubLanguage::Canada,
                0x1000 => SubLanguage::Switzerland,
                0x1400 => SubLanguage::Luxembourg,
                0x1800 => SubLanguage::Monaco,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::German => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Standard,
                0x0800 => SubLanguage::Switzerland,
                0x0C00 => SubLanguage::Austria,
                0x1000 => SubLanguage::Luxembourg,
                0x1400 => SubLanguage::Liechtenstein,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Italian => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Standard,
                0x0800 => SubLanguage::Switzerland,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Korean => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Standard,
                0x0800 => SubLanguage::Johab,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Lithuanian => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Standard,
                0x0800 => SubLanguage::Classic,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Malay => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Malaysia,
                0x0800 => SubLanguage::BruneiDarussalam,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Norwegian => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Bokmal,
                0x0800 => SubLanguage::Nynorsk,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Portuguese => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Brazil,
                0x0800 => SubLanguage::Standard,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Serbian => match raw & SUB_LANGUAGE_MASK {
                0x0C00 => SubLanguage::Cyrillic,
                0x0800 => SubLanguage::Latin,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Spanish => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Traditional,
                0x0800 => SubLanguage::Mexico,
                0x0C00 => SubLanguage::Modern,
                0x1000 => SubLanguage::Guatemala,
                0x1400 => SubLanguage::CostaRica,
                0x1800 => SubLanguage::Panama,
                0x1C00 => SubLanguage::DominicanRepublic,
                0x2000 => SubLanguage::Venezuela,
                0x2400 => SubLanguage::Colombia,
                0x2800 => SubLanguage::Peru,
                0x2C00 => SubLanguage::Argentina,
                0x3000 => SubLanguage::Ecuador,
                0x3400 => SubLanguage::Chile,
                0x3800 => SubLanguage::Uruguay,
                0x3C00 => SubLanguage::Paraguay,
                0x4000 => SubLanguage::Bolivia,
                0x4400 => SubLanguage::ElSalvador,
                0x4800 => SubLanguage::Honduras,
                0x4C00 => SubLanguage::Nicaragua,
                0x5000 => SubLanguage::PuertoRico,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Swedish => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Standard,
                0x0800 => SubLanguage::Finland,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Urdu => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Pakistan,
                0x0800 => SubLanguage::India,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Uzbek => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::Latin,
                0x0800 => SubLanguage::Cyrillic,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::HID => match raw & SUB_LANGUAGE_MASK {
                0x0400 => SubLanguage::UsageDataDescriptor,
                0xF000 => SubLanguage::VendorDefined1,
                0xF400 => SubLanguage::VendorDefined2,
                0xF800 => SubLanguage::VendorDefined3,
                0xFC00 => SubLanguage::VendorDefined4,
                n => SubLanguage::Other(n),
            },
            PrimaryLanguage::Other(_) => SubLanguage::Other(raw & SUB_LANGUAGE_MASK),
            _ => SubLanguage::Standard,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{PrimaryLanguage, SubLanguage};
    use super::{PRIMARY_LANGUAGE_MASK, SUB_LANGUAGE_MASK};

    // language ids defined in http://www.usb.org/developers/docs/USB_LANGIDs.pdf
    const AFRIKAANS: u16 = 0x0436;
    const ALBANIAN: u16 = 0x041C;
    const ARABIC_SAUDI_ARABIA: u16 = 0x0401;
    const ARABIC_IRAQ: u16 = 0x0801;
    const ARABIC_EGYPT: u16 = 0x0C01;
    const ARABIC_LIBYA: u16 = 0x1001;
    const ARABIC_ALGERIA: u16 = 0x1401;
    const ARABIC_MOROCCO: u16 = 0x1801;
    const ARABIC_TUNISIA: u16 = 0x1C01;
    const ARABIC_OMAN: u16 = 0x2001;
    const ARABIC_YEMEN: u16 = 0x2401;
    const ARABIC_SYRIA: u16 = 0x2801;
    const ARABIC_JORDAN: u16 = 0x2C01;
    const ARABIC_LEBANON: u16 = 0x3001;
    const ARABIC_KUWAIT: u16 = 0x3401;
    const ARABIC_UAE: u16 = 0x3801;
    const ARABIC_BAHRAIN: u16 = 0x3C01;
    const ARABIC_QATAR: u16 = 0x4001;
    const ARMENIAN: u16 = 0x042B;
    const ASSAMESE: u16 = 0x044D;
    const AZERI_LATIN: u16 = 0x042C;
    const AZERI_CYRILLIC: u16 = 0x082C;
    const BASQUE: u16 = 0x042D;
    const BELARUSSIAN: u16 = 0x0423;
    const BENGALI: u16 = 0x0445;
    const BULGARIAN: u16 = 0x0402;
    const BURMESE: u16 = 0x0455;
    const CATALAN: u16 = 0x0403;
    const CHINESE_TAIWAN: u16 = 0x0404;
    const CHINESE_CHINA: u16 = 0x0804;
    const CHINESE_HONG_KONG: u16 = 0x0C04;
    const CHINESE_SINGAPORE: u16 = 0x1004;
    const CHINESE_MACAU: u16 = 0x1404;
    const CROATIAN: u16 = 0x041A;
    const CZECH: u16 = 0x0405;
    const DANISH: u16 = 0x0406;
    const DUTCH_NETHERLANDS: u16 = 0x0413;
    const DUTCH_BELGIUM: u16 = 0x0813;
    const ENGLISH_UNITED_STATES: u16 = 0x0409;
    const ENGLISH_UNITED_KINGDOM: u16 = 0x0809;
    const ENGLISH_AUSTRALIAN: u16 = 0x0C09;
    const ENGLISH_CANADIAN: u16 = 0x1009;
    const ENGLISH_NEW_ZEALAND: u16 = 0x1409;
    const ENGLISH_IRELAND: u16 = 0x1809;
    const ENGLISH_SOUTH_AFRICA: u16 = 0x1C09;
    const ENGLISH_JAMAICA: u16 = 0x2009;
    const ENGLISH_CARIBBEAN: u16 = 0x2409;
    const ENGLISH_BELIZE: u16 = 0x2809;
    const ENGLISH_TRINIDAD: u16 = 0x2C09;
    const ENGLISH_ZIMBABWE: u16 = 0x3009;
    const ENGLISH_PHILIPPINES: u16 = 0x3409;
    const ESTONIAN: u16 = 0x0425;
    const FAEROESE: u16 = 0x0438;
    const FARSI: u16 = 0x0429;
    const FINNISH: u16 = 0x040B;
    const FRENCH_STANDARD: u16 = 0x040C;
    const FRENCH_BELGIAN: u16 = 0x080C;
    const FRENCH_CANADIAN: u16 = 0x0C0C;
    const FRENCH_SWITZERLAND: u16 = 0x100C;
    const FRENCH_LUXEMBOURG: u16 = 0x140C;
    const FRENCH_MONACO: u16 = 0x180C;
    const GEORGIAN: u16 = 0x0437;
    const GERMAN_STANDARD: u16 = 0x0407;
    const GERMAN_SWITZERLAND: u16 = 0x0807;
    const GERMAN_AUSTRIA: u16 = 0x0C07;
    const GERMAN_LUXEMBOURG: u16 = 0x1007;
    const GERMAN_LIECHTENSTEIN: u16 = 0x1407;
    const GREEK: u16 = 0x0408;
    const GUJARATI: u16 = 0x0447;
    const HEBREW: u16 = 0x040D;
    const HINDI: u16 = 0x0439;
    const HUNGARIAN: u16 = 0x040E;
    const ICELANDIC: u16 = 0x040F;
    const INDONESIAN: u16 = 0x0421;
    const ITALIAN_STANDARD: u16 = 0x0410;
    const ITALIAN_SWITZERLAND: u16 = 0x0810;
    const JAPANESE: u16 = 0x0411;
    const KANNADA: u16 = 0x044B;
    const KASHMIRI_INDIA: u16 = 0x0860;
    const KAZAKH: u16 = 0x043F;
    const KONKANI: u16 = 0x0457;
    const KOREAN: u16 = 0x0412;
    const KOREAN_JOHAB: u16 = 0x0812;
    const LATVIAN: u16 = 0x0426;
    const LITHUANIAN: u16 = 0x0427;
    const LITHUANIAN_CLASSIC: u16 = 0x0827;
    const MACEDONIAN: u16 = 0x042F;
    const MALAY_MALAYSIAN: u16 = 0x043E;
    const MALAY_BRUNEI_DARUSSALAM: u16 = 0x083E;
    const MALAYALAM: u16 = 0x044C;
    const MANIPURI: u16 = 0x0458;
    const MARATHI: u16 = 0x044E;
    const NEPALI_INDIA: u16 = 0x0861;
    const NORWEGIAN_BOKMAL: u16 = 0x0414;
    const NORWEGIAN_NYNORSK: u16 = 0x0814;
    const ORIYA: u16 = 0x0448;
    const POLISH: u16 = 0x0415;
    const PORTUGUESE_BRAZIL: u16 = 0x0416;
    const PORTUGUESE_STANDARD: u16 = 0x0816;
    const PUNJABI: u16 = 0x0446;
    const ROMANIAN: u16 = 0x0418;
    const RUSSIAN: u16 = 0x0419;
    const SANSKRIT: u16 = 0x044F;
    const SERBIAN_CYRILLIC: u16 = 0x0C1A;
    const SERBIAN_LATIN: u16 = 0x081A;
    const SINDHI: u16 = 0x0459;
    const SLOVAK: u16 = 0x041B;
    const SLOVENIAN: u16 = 0x0424;
    const SPANISH_TRADITIONAL_SORT: u16 = 0x040A;
    const SPANISH_MEXICAN: u16 = 0x080A;
    const SPANISH_MODERN_SORT: u16 = 0x0C0A;
    const SPANISH_GUATEMALA: u16 = 0x100A;
    const SPANISH_COSTA_RICA: u16 = 0x140A;
    const SPANISH_PANAMA: u16 = 0x180A;
    const SPANISH_DOMINICAN_REPUBLIC: u16 = 0x1C0A;
    const SPANISH_VENEZUELA: u16 = 0x200A;
    const SPANISH_COLOMBIA: u16 = 0x240A;
    const SPANISH_PERU: u16 = 0x280A;
    const SPANISH_ARGENTINA: u16 = 0x2C0A;
    const SPANISH_ECUADOR: u16 = 0x300A;
    const SPANISH_CHILE: u16 = 0x340A;
    const SPANISH_URUGUAY: u16 = 0x380A;
    const SPANISH_PARAGUAY: u16 = 0x3C0A;
    const SPANISH_BOLIVIA: u16 = 0x400A;
    const SPANISH_EL_SALVADOR: u16 = 0x440A;
    const SPANISH_HONDURAS: u16 = 0x480A;
    const SPANISH_NICARAGUA: u16 = 0x4C0A;
    const SPANISH_PUERTO_RICO: u16 = 0x500A;
    const SUTU: u16 = 0x0430;
    const SWAHILI_KENYA: u16 = 0x0441;
    const SWEDISH: u16 = 0x041D;
    const SWEDISH_FINLAND: u16 = 0x081D;
    const TAMIL: u16 = 0x0449;
    const TATAR_TATARSTAN: u16 = 0x0444;
    const TELUGU: u16 = 0x044A;
    const THAI: u16 = 0x041E;
    const TURKISH: u16 = 0x041F;
    const UKRAINIAN: u16 = 0x0422;
    const URDU_PAKISTAN: u16 = 0x0420;
    const URDU_INDIA: u16 = 0x0820;
    const UZBEK_LATIN: u16 = 0x0443;
    const UZBEK_CYRILLIC: u16 = 0x0843;
    const VIETNAMESE: u16 = 0x042A;
    const HID_USAGE_DATA_DESCRIPTOR: u16 = 0x04FF;
    const HID_VENDOR_DEFINED_1: u16 = 0xF0FF;
    const HID_VENDOR_DEFINED_2: u16 = 0xF4FF;
    const HID_VENDOR_DEFINED_3: u16 = 0xF8FF;
    const HID_VENDOR_DEFINED_4: u16 = 0xFCFF;

    #[test]
    fn it_recognizes_afrikaans_as_afrikaans_language() {
        assert_eq!(
            super::from_lang_id(AFRIKAANS).primary_language(),
            PrimaryLanguage::Afrikaans
        );
    }

    #[test]
    fn it_recognizes_albanian_as_albanian_language() {
        assert_eq!(
            super::from_lang_id(ALBANIAN).primary_language(),
            PrimaryLanguage::Albanian
        );
    }

    #[test]
    fn it_recognizes_arabic_from_saudi_arabia_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_SAUDI_ARABIA).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_saudi_arabia_as_saudi_arabia_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_SAUDI_ARABIA).sub_language(),
            SubLanguage::SaudiArabia
        );
    }

    #[test]
    fn it_recognizes_arabic_from_iraq_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_IRAQ).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_iraq_as_iraq_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_IRAQ).sub_language(),
            SubLanguage::Iraq
        );
    }

    #[test]
    fn it_recognizes_arabic_from_egypt_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_EGYPT).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_egypt_as_egypt_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_EGYPT).sub_language(),
            SubLanguage::Egypt
        );
    }

    #[test]
    fn it_recognizes_arabic_from_libya_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_LIBYA).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_libya_as_libya_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_LIBYA).sub_language(),
            SubLanguage::Libya
        );
    }

    #[test]
    fn it_recognizes_arabic_from_algeria_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_ALGERIA).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_algeria_as_algeria_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_ALGERIA).sub_language(),
            SubLanguage::Algeria
        );
    }

    #[test]
    fn it_recognizes_arabic_from_morocco_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_MOROCCO).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_morocco_as_morocco_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_MOROCCO).sub_language(),
            SubLanguage::Morocco
        );
    }

    #[test]
    fn it_recognizes_arabic_from_tunisia_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_TUNISIA).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_tunisia_as_tunisia_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_TUNISIA).sub_language(),
            SubLanguage::Tunisia
        );
    }

    #[test]
    fn it_recognizes_arabic_from_oman_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_OMAN).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_oman_as_oman_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_OMAN).sub_language(),
            SubLanguage::Oman
        );
    }

    #[test]
    fn it_recognizes_arabic_from_yemen_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_YEMEN).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_yemen_as_yemen_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_YEMEN).sub_language(),
            SubLanguage::Yemen
        );
    }

    #[test]
    fn it_recognizes_arabic_from_syria_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_SYRIA).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_syria_as_syria_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_SYRIA).sub_language(),
            SubLanguage::Syria
        );
    }

    #[test]
    fn it_recognizes_arabic_from_jordan_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_JORDAN).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_jordan_as_jordan_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_JORDAN).sub_language(),
            SubLanguage::Jordan
        );
    }

    #[test]
    fn it_recognizes_arabic_from_lebanon_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_LEBANON).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_lebanon_as_lebanon_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_LEBANON).sub_language(),
            SubLanguage::Lebanon
        );
    }

    #[test]
    fn it_recognizes_arabic_from_kuwait_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_KUWAIT).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_kuwait_as_kuwait_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_KUWAIT).sub_language(),
            SubLanguage::Kuwait
        );
    }

    #[test]
    fn it_recognizes_arabic_from_uae_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_UAE).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_uae_as_uae_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_UAE).sub_language(),
            SubLanguage::UnitedArabEmirates
        );
    }

    #[test]
    fn it_recognizes_arabic_from_bahrain_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_BAHRAIN).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_bahrain_as_bahrain_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_BAHRAIN).sub_language(),
            SubLanguage::Bahrain
        );
    }

    #[test]
    fn it_recognizes_arabic_from_qatar_as_arabic_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_QATAR).primary_language(),
            PrimaryLanguage::Arabic
        );
    }

    #[test]
    fn it_recognizes_arabic_from_qatar_as_qatar_sub_language() {
        assert_eq!(
            super::from_lang_id(ARABIC_QATAR).sub_language(),
            SubLanguage::Qatar
        );
    }

    #[test]
    fn it_recognizes_armenian_as_armenian_language() {
        assert_eq!(
            super::from_lang_id(ARMENIAN).primary_language(),
            PrimaryLanguage::Armenian
        );
    }

    #[test]
    fn it_recognizes_assamese_as_assamese_language() {
        assert_eq!(
            super::from_lang_id(ASSAMESE).primary_language(),
            PrimaryLanguage::Assamese
        );
    }

    #[test]
    fn it_recognizes_azeri_latin_as_azeri_language() {
        assert_eq!(
            super::from_lang_id(AZERI_LATIN).primary_language(),
            PrimaryLanguage::Azeri
        );
    }

    #[test]
    fn it_recognizes_azeri_latin_as_latin_sub_language() {
        assert_eq!(
            super::from_lang_id(AZERI_LATIN).sub_language(),
            SubLanguage::Latin
        );
    }

    #[test]
    fn it_recognizes_azeri_cyrillic_as_azeri_language() {
        assert_eq!(
            super::from_lang_id(AZERI_CYRILLIC).primary_language(),
            PrimaryLanguage::Azeri
        );
    }

    #[test]
    fn it_recognizes_azeri_cyrillic_as_cyrillic_sub_language() {
        assert_eq!(
            super::from_lang_id(AZERI_CYRILLIC).sub_language(),
            SubLanguage::Cyrillic
        );
    }

    #[test]
    fn it_recognizes_basque_as_basque_language() {
        assert_eq!(
            super::from_lang_id(BASQUE).primary_language(),
            PrimaryLanguage::Basque
        );
    }

    #[test]
    fn it_recognizes_belarussian_as_belarussian_language() {
        assert_eq!(
            super::from_lang_id(BELARUSSIAN).primary_language(),
            PrimaryLanguage::Belarussian
        );
    }

    #[test]
    fn it_recognizes_bengali_as_bengali_language() {
        assert_eq!(
            super::from_lang_id(BENGALI).primary_language(),
            PrimaryLanguage::Bengali
        );
    }

    #[test]
    fn it_recognizes_bulgarian_as_bulgarian_language() {
        assert_eq!(
            super::from_lang_id(BULGARIAN).primary_language(),
            PrimaryLanguage::Bulgarian
        );
    }

    #[test]
    fn it_recognizes_burmese_as_burmese_language() {
        assert_eq!(
            super::from_lang_id(BURMESE).primary_language(),
            PrimaryLanguage::Burmese
        );
    }

    #[test]
    fn it_recognizes_catalan_as_catalan_language() {
        assert_eq!(
            super::from_lang_id(CATALAN).primary_language(),
            PrimaryLanguage::Catalan
        );
    }

    #[test]
    fn it_recognizes_chinese_from_taiwan_as_chinese_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_TAIWAN).primary_language(),
            PrimaryLanguage::Chinese
        );
    }

    #[test]
    fn it_recognizes_chinese_from_taiwan_as_taiwan_sub_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_TAIWAN).sub_language(),
            SubLanguage::Taiwan
        );
    }

    #[test]
    fn it_recognizes_chinese_from_china_as_chinese_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_CHINA).primary_language(),
            PrimaryLanguage::Chinese
        );
    }

    #[test]
    fn it_recognizes_chinese_from_china_as_china_sub_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_CHINA).sub_language(),
            SubLanguage::China
        );
    }

    #[test]
    fn it_recognizes_chinese_from_hong_kong_as_chinese_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_HONG_KONG).primary_language(),
            PrimaryLanguage::Chinese
        );
    }

    #[test]
    fn it_recognizes_chinese_from_hong_kong_as_hong_kong_sub_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_HONG_KONG).sub_language(),
            SubLanguage::HongKong
        );
    }

    #[test]
    fn it_recognizes_chinese_from_singapore_as_chinese_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_SINGAPORE).primary_language(),
            PrimaryLanguage::Chinese
        );
    }

    #[test]
    fn it_recognizes_chinese_from_singapore_as_singapore_sub_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_SINGAPORE).sub_language(),
            SubLanguage::Singapore
        );
    }

    #[test]
    fn it_recognizes_chinese_from_macau_as_chinese_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_MACAU).primary_language(),
            PrimaryLanguage::Chinese
        );
    }

    #[test]
    fn it_recognizes_chinese_from_macau_as_macau_sub_language() {
        assert_eq!(
            super::from_lang_id(CHINESE_MACAU).sub_language(),
            SubLanguage::Macau
        );
    }

    #[test]
    fn it_recognizes_croatian_as_croatian_language() {
        assert_eq!(
            super::from_lang_id(CROATIAN).primary_language(),
            PrimaryLanguage::Croatian
        );
    }

    #[test]
    fn it_recognizes_czech_as_czech_language() {
        assert_eq!(
            super::from_lang_id(CZECH).primary_language(),
            PrimaryLanguage::Czech
        );
    }

    #[test]
    fn it_recognizes_danish_as_danish_language() {
        assert_eq!(
            super::from_lang_id(DANISH).primary_language(),
            PrimaryLanguage::Danish
        );
    }

    #[test]
    fn it_recognizes_dutch_from_netherlands_as_dutch_language() {
        assert_eq!(
            super::from_lang_id(DUTCH_NETHERLANDS).primary_language(),
            PrimaryLanguage::Dutch
        );
    }

    #[test]
    fn it_recognizes_dutch_from_netherlands_as_netherlands_sub_language() {
        assert_eq!(
            super::from_lang_id(DUTCH_NETHERLANDS).sub_language(),
            SubLanguage::Netherlands
        );
    }

    #[test]
    fn it_recognizes_dutch_from_belgium_as_dutch_language() {
        assert_eq!(
            super::from_lang_id(DUTCH_BELGIUM).primary_language(),
            PrimaryLanguage::Dutch
        );
    }

    #[test]
    fn it_recognizes_dutch_from_belgium_as_belgium_sub_language() {
        assert_eq!(
            super::from_lang_id(DUTCH_BELGIUM).sub_language(),
            SubLanguage::Belgium
        );
    }

    #[test]
    fn it_recognizes_english_from_united_states_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_UNITED_STATES).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_united_states_as_united_states_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_UNITED_STATES).sub_language(),
            SubLanguage::UnitedStates
        );
    }

    #[test]
    fn it_recognizes_english_from_united_kingdom_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_UNITED_KINGDOM).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_united_kingdom_as_united_kingdom_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_UNITED_KINGDOM).sub_language(),
            SubLanguage::UnitedKingdom
        );
    }

    #[test]
    fn it_recognizes_english_from_australia_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_AUSTRALIAN).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_australia_as_australia_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_AUSTRALIAN).sub_language(),
            SubLanguage::Australia
        );
    }

    #[test]
    fn it_recognizes_english_from_canada_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_CANADIAN).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_canada_as_canada_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_CANADIAN).sub_language(),
            SubLanguage::Canada
        );
    }

    #[test]
    fn it_recognizes_english_from_new_zealand_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_NEW_ZEALAND).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_new_zealand_as_new_zealand_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_NEW_ZEALAND).sub_language(),
            SubLanguage::NewZealand
        );
    }

    #[test]
    fn it_recognizes_english_from_ireland_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_IRELAND).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_ireland_as_ireland_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_IRELAND).sub_language(),
            SubLanguage::Ireland
        );
    }

    #[test]
    fn it_recognizes_english_from_south_africa_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_SOUTH_AFRICA).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_south_africa_as_south_africa_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_SOUTH_AFRICA).sub_language(),
            SubLanguage::SouthAfrica
        );
    }

    #[test]
    fn it_recognizes_english_from_jamaica_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_JAMAICA).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_jamaica_as_jamaica_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_JAMAICA).sub_language(),
            SubLanguage::Jamaica
        );
    }

    #[test]
    fn it_recognizes_english_from_caribbean_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_CARIBBEAN).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_caribbean_as_caribbean_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_CARIBBEAN).sub_language(),
            SubLanguage::Caribbean
        );
    }

    #[test]
    fn it_recognizes_english_from_belize_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_BELIZE).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_belize_as_belize_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_BELIZE).sub_language(),
            SubLanguage::Belize
        );
    }

    #[test]
    fn it_recognizes_english_from_trinidad_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_TRINIDAD).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_trinidad_as_trinidad_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_TRINIDAD).sub_language(),
            SubLanguage::Trinidad
        );
    }

    #[test]
    fn it_recognizes_english_from_zimbabwe_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_ZIMBABWE).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_zimbabwe_as_zimbabwe_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_ZIMBABWE).sub_language(),
            SubLanguage::Zimbabwe
        );
    }

    #[test]
    fn it_recognizes_english_from_philippines_as_english_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_PHILIPPINES).primary_language(),
            PrimaryLanguage::English
        );
    }

    #[test]
    fn it_recognizes_english_from_philippines_as_philippines_sub_language() {
        assert_eq!(
            super::from_lang_id(ENGLISH_PHILIPPINES).sub_language(),
            SubLanguage::Philippines
        );
    }

    #[test]
    fn it_recognizes_estonian_as_estonian_language() {
        assert_eq!(
            super::from_lang_id(ESTONIAN).primary_language(),
            PrimaryLanguage::Estonian
        );
    }

    #[test]
    fn it_recognizes_faeroese_as_faeroese_language() {
        assert_eq!(
            super::from_lang_id(FAEROESE).primary_language(),
            PrimaryLanguage::Faeroese
        );
    }

    #[test]
    fn it_recognizes_farsi_as_farsi_language() {
        assert_eq!(
            super::from_lang_id(FARSI).primary_language(),
            PrimaryLanguage::Farsi
        );
    }

    #[test]
    fn it_recognizes_finnish_as_finnish_language() {
        assert_eq!(
            super::from_lang_id(FINNISH).primary_language(),
            PrimaryLanguage::Finnish
        );
    }

    #[test]
    fn it_recognizes_french_standard_as_french_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_STANDARD).primary_language(),
            PrimaryLanguage::French
        );
    }

    #[test]
    fn it_recognizes_french_standard_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_STANDARD).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_french_from_belgium_as_french_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_BELGIAN).primary_language(),
            PrimaryLanguage::French
        );
    }

    #[test]
    fn it_recognizes_french_from_belgium_as_belgium_sub_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_BELGIAN).sub_language(),
            SubLanguage::Belgium
        );
    }

    #[test]
    fn it_recognizes_french_from_canada_as_french_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_CANADIAN).primary_language(),
            PrimaryLanguage::French
        );
    }

    #[test]
    fn it_recognizes_french_from_canada_as_canada_sub_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_CANADIAN).sub_language(),
            SubLanguage::Canada
        );
    }

    #[test]
    fn it_recognizes_french_from_switzerland_as_french_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_SWITZERLAND).primary_language(),
            PrimaryLanguage::French
        );
    }

    #[test]
    fn it_recognizes_french_from_switzerland_as_switzerland_sub_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_SWITZERLAND).sub_language(),
            SubLanguage::Switzerland
        );
    }

    #[test]
    fn it_recognizes_french_from_luxembourg_as_french_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_LUXEMBOURG).primary_language(),
            PrimaryLanguage::French
        );
    }

    #[test]
    fn it_recognizes_french_from_luxembourg_as_luxembourg_sub_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_LUXEMBOURG).sub_language(),
            SubLanguage::Luxembourg
        );
    }

    #[test]
    fn it_recognizes_french_from_monaco_as_french_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_MONACO).primary_language(),
            PrimaryLanguage::French
        );
    }

    #[test]
    fn it_recognizes_french_from_monaco_as_monaco_sub_language() {
        assert_eq!(
            super::from_lang_id(FRENCH_MONACO).sub_language(),
            SubLanguage::Monaco
        );
    }

    #[test]
    fn it_recognizes_georgian_as_georgian_language() {
        assert_eq!(
            super::from_lang_id(GEORGIAN).primary_language(),
            PrimaryLanguage::Georgian
        );
    }

    #[test]
    fn it_recognizes_german_standard_as_german_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_STANDARD).primary_language(),
            PrimaryLanguage::German
        );
    }

    #[test]
    fn it_recognizes_german_standard_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_STANDARD).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_german_from_switzerland_as_german_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_SWITZERLAND).primary_language(),
            PrimaryLanguage::German
        );
    }

    #[test]
    fn it_recognizes_german_from_switzerland_as_switzerland_sub_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_SWITZERLAND).sub_language(),
            SubLanguage::Switzerland
        );
    }

    #[test]
    fn it_recognizes_german_from_austria_as_german_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_AUSTRIA).primary_language(),
            PrimaryLanguage::German
        );
    }

    #[test]
    fn it_recognizes_german_from_austria_as_austria_sub_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_AUSTRIA).sub_language(),
            SubLanguage::Austria
        );
    }

    #[test]
    fn it_recognizes_german_from_luxembourg_as_german_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_LUXEMBOURG).primary_language(),
            PrimaryLanguage::German
        );
    }

    #[test]
    fn it_recognizes_german_from_luxembourg_as_luxembourg_sub_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_LUXEMBOURG).sub_language(),
            SubLanguage::Luxembourg
        );
    }

    #[test]
    fn it_recognizes_german_from_liechtenstein_as_german_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_LIECHTENSTEIN).primary_language(),
            PrimaryLanguage::German
        );
    }

    #[test]
    fn it_recognizes_german_from_liechtenstein_as_liechtenstein_sub_language() {
        assert_eq!(
            super::from_lang_id(GERMAN_LIECHTENSTEIN).sub_language(),
            SubLanguage::Liechtenstein
        );
    }

    #[test]
    fn it_recognizes_greek_as_greek_language() {
        assert_eq!(
            super::from_lang_id(GREEK).primary_language(),
            PrimaryLanguage::Greek
        );
    }

    #[test]
    fn it_recognizes_gujarati_as_gujarati_language() {
        assert_eq!(
            super::from_lang_id(GUJARATI).primary_language(),
            PrimaryLanguage::Gujarati
        );
    }

    #[test]
    fn it_recognizes_hebrew_as_hebrew_language() {
        assert_eq!(
            super::from_lang_id(HEBREW).primary_language(),
            PrimaryLanguage::Hebrew
        );
    }

    #[test]
    fn it_recognizes_hindi_as_hindi_language() {
        assert_eq!(
            super::from_lang_id(HINDI).primary_language(),
            PrimaryLanguage::Hindi
        );
    }

    #[test]
    fn it_recognizes_hungarian_as_hungarian_language() {
        assert_eq!(
            super::from_lang_id(HUNGARIAN).primary_language(),
            PrimaryLanguage::Hungarian
        );
    }

    #[test]
    fn it_recognizes_icelandic_as_icelandic_language() {
        assert_eq!(
            super::from_lang_id(ICELANDIC).primary_language(),
            PrimaryLanguage::Icelandic
        );
    }

    #[test]
    fn it_recognizes_indonesian_as_indonesian_language() {
        assert_eq!(
            super::from_lang_id(INDONESIAN).primary_language(),
            PrimaryLanguage::Indonesian
        );
    }

    #[test]
    fn it_recognizes_italian_standard_as_italian_language() {
        assert_eq!(
            super::from_lang_id(ITALIAN_STANDARD).primary_language(),
            PrimaryLanguage::Italian
        );
    }

    #[test]
    fn it_recognizes_italian_standard_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(ITALIAN_STANDARD).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_italian_from_switzerland_as_italian_language() {
        assert_eq!(
            super::from_lang_id(ITALIAN_SWITZERLAND).primary_language(),
            PrimaryLanguage::Italian
        );
    }

    #[test]
    fn it_recognizes_italian_from_switzerland_as_switzerland_sub_language() {
        assert_eq!(
            super::from_lang_id(ITALIAN_SWITZERLAND).sub_language(),
            SubLanguage::Switzerland
        );
    }

    #[test]
    fn it_recognizes_japanese_as_japanese_language() {
        assert_eq!(
            super::from_lang_id(JAPANESE).primary_language(),
            PrimaryLanguage::Japanese
        );
    }

    #[test]
    fn it_recognizes_kannada_as_kannada_language() {
        assert_eq!(
            super::from_lang_id(KANNADA).primary_language(),
            PrimaryLanguage::Kannada
        );
    }

    #[test]
    fn it_recognizes_kashmiri_as_kashmiri_language() {
        assert_eq!(
            super::from_lang_id(KASHMIRI_INDIA).primary_language(),
            PrimaryLanguage::Kashmiri
        );
    }

    #[test]
    fn it_recognizes_kazakh_as_kazakh_language() {
        assert_eq!(
            super::from_lang_id(KAZAKH).primary_language(),
            PrimaryLanguage::Kazakh
        );
    }

    #[test]
    fn it_recognizes_konkani_as_konkani_language() {
        assert_eq!(
            super::from_lang_id(KONKANI).primary_language(),
            PrimaryLanguage::Konkani
        );
    }

    #[test]
    fn it_recognizes_korean_as_korean_language() {
        assert_eq!(
            super::from_lang_id(KOREAN).primary_language(),
            PrimaryLanguage::Korean
        );
    }

    #[test]
    fn it_recognizes_korean_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(KOREAN).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_korean_johab_as_korean_language() {
        assert_eq!(
            super::from_lang_id(KOREAN_JOHAB).primary_language(),
            PrimaryLanguage::Korean
        );
    }

    #[test]
    fn it_recognizes_korean_johab_as_johab_sub_language() {
        assert_eq!(
            super::from_lang_id(KOREAN_JOHAB).sub_language(),
            SubLanguage::Johab
        );
    }

    #[test]
    fn it_recognizes_latvian_as_latvian_language() {
        assert_eq!(
            super::from_lang_id(LATVIAN).primary_language(),
            PrimaryLanguage::Latvian
        );
    }

    #[test]
    fn it_recognizes_lithuanian_as_lithuanian_language() {
        assert_eq!(
            super::from_lang_id(LITHUANIAN).primary_language(),
            PrimaryLanguage::Lithuanian
        );
    }

    #[test]
    fn it_recognizes_lithuanian_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(LITHUANIAN).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_lithuanian_classic_as_lithuanian_language() {
        assert_eq!(
            super::from_lang_id(LITHUANIAN_CLASSIC).primary_language(),
            PrimaryLanguage::Lithuanian
        );
    }

    #[test]
    fn it_recognizes_lithuanian_classic_as_classic_sub_language() {
        assert_eq!(
            super::from_lang_id(LITHUANIAN_CLASSIC).sub_language(),
            SubLanguage::Classic
        );
    }

    #[test]
    fn it_recognizes_macedonian_as_macedonian_language() {
        assert_eq!(
            super::from_lang_id(MACEDONIAN).primary_language(),
            PrimaryLanguage::Macedonian
        );
    }

    #[test]
    fn it_recognizes_malay_from_malaysia_as_malay_language() {
        assert_eq!(
            super::from_lang_id(MALAY_MALAYSIAN).primary_language(),
            PrimaryLanguage::Malay
        );
    }

    #[test]
    fn it_recognizes_malay_from_malaysia_as_malaysia_sub_language() {
        assert_eq!(
            super::from_lang_id(MALAY_MALAYSIAN).sub_language(),
            SubLanguage::Malaysia
        );
    }

    #[test]
    fn it_recognizes_malay_from_brunei_darussalam_as_malay_language() {
        assert_eq!(
            super::from_lang_id(MALAY_BRUNEI_DARUSSALAM).primary_language(),
            PrimaryLanguage::Malay
        );
    }

    #[test]
    fn it_recognizes_malay_from_brunei_darussalam_as_brunei_darussalam_sub_language() {
        assert_eq!(
            super::from_lang_id(MALAY_BRUNEI_DARUSSALAM).sub_language(),
            SubLanguage::BruneiDarussalam
        );
    }

    #[test]
    fn it_recognizes_malayalam_as_malayalam_language() {
        assert_eq!(
            super::from_lang_id(MALAYALAM).primary_language(),
            PrimaryLanguage::Malayalam
        );
    }

    #[test]
    fn it_recognizes_manipuri_as_manipuri_language() {
        assert_eq!(
            super::from_lang_id(MANIPURI).primary_language(),
            PrimaryLanguage::Manipuri
        );
    }

    #[test]
    fn it_recognizes_marathi_as_marathi_language() {
        assert_eq!(
            super::from_lang_id(MARATHI).primary_language(),
            PrimaryLanguage::Marathi
        );
    }

    #[test]
    fn it_recognizes_nepali_as_nepali_language() {
        assert_eq!(
            super::from_lang_id(NEPALI_INDIA).primary_language(),
            PrimaryLanguage::Nepali
        );
    }

    #[test]
    fn it_recognizes_norwegian_bokmal_as_norwegian_language() {
        assert_eq!(
            super::from_lang_id(NORWEGIAN_BOKMAL).primary_language(),
            PrimaryLanguage::Norwegian
        );
    }

    #[test]
    fn it_recognizes_norwegian_bokmal_as_bokmal_sub_language() {
        assert_eq!(
            super::from_lang_id(NORWEGIAN_BOKMAL).sub_language(),
            SubLanguage::Bokmal
        );
    }

    #[test]
    fn it_recognizes_norwegian_nynorsk_as_norwegian_language() {
        assert_eq!(
            super::from_lang_id(NORWEGIAN_NYNORSK).primary_language(),
            PrimaryLanguage::Norwegian
        );
    }

    #[test]
    fn it_recognizes_norwegian_nynorsk_as_nynorsk_sub_language() {
        assert_eq!(
            super::from_lang_id(NORWEGIAN_NYNORSK).sub_language(),
            SubLanguage::Nynorsk
        );
    }

    #[test]
    fn it_recognizes_oriya_as_oriya_language() {
        assert_eq!(
            super::from_lang_id(ORIYA).primary_language(),
            PrimaryLanguage::Oriya
        );
    }

    #[test]
    fn it_recognizes_polish_as_polish_language() {
        assert_eq!(
            super::from_lang_id(POLISH).primary_language(),
            PrimaryLanguage::Polish
        );
    }

    #[test]
    fn it_recognizes_portuguese_from_brazil_as_portuguese_language() {
        assert_eq!(
            super::from_lang_id(PORTUGUESE_BRAZIL).primary_language(),
            PrimaryLanguage::Portuguese
        );
    }

    #[test]
    fn it_recognizes_portuguese_from_brazil_as_brazil_sub_language() {
        assert_eq!(
            super::from_lang_id(PORTUGUESE_BRAZIL).sub_language(),
            SubLanguage::Brazil
        );
    }

    #[test]
    fn it_recognizes_portuguese_standard_as_portuguese_language() {
        assert_eq!(
            super::from_lang_id(PORTUGUESE_STANDARD).primary_language(),
            PrimaryLanguage::Portuguese
        );
    }

    #[test]
    fn it_recognizes_portuguese_standard_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(PORTUGUESE_STANDARD).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_punjabi_as_punjabi_language() {
        assert_eq!(
            super::from_lang_id(PUNJABI).primary_language(),
            PrimaryLanguage::Punjabi
        );
    }

    #[test]
    fn it_recognizes_romanian_as_romanian_language() {
        assert_eq!(
            super::from_lang_id(ROMANIAN).primary_language(),
            PrimaryLanguage::Romanian
        );
    }

    #[test]
    fn it_recognizes_russian_as_russian_language() {
        assert_eq!(
            super::from_lang_id(RUSSIAN).primary_language(),
            PrimaryLanguage::Russian
        );
    }

    #[test]
    fn it_recognizes_sanskrit_as_sanskrit_language() {
        assert_eq!(
            super::from_lang_id(SANSKRIT).primary_language(),
            PrimaryLanguage::Sanskrit
        );
    }

    #[test]
    fn it_recognizes_serbian_cyrillic_as_serbian_language() {
        assert_eq!(
            super::from_lang_id(SERBIAN_CYRILLIC).primary_language(),
            PrimaryLanguage::Serbian
        );
    }

    #[test]
    fn it_recognizes_serbian_cyrillic_as_cyrillic_sub_language() {
        assert_eq!(
            super::from_lang_id(SERBIAN_CYRILLIC).sub_language(),
            SubLanguage::Cyrillic
        );
    }

    #[test]
    fn it_recognizes_serbian_latin_as_serbian_language() {
        assert_eq!(
            super::from_lang_id(SERBIAN_LATIN).primary_language(),
            PrimaryLanguage::Serbian
        );
    }

    #[test]
    fn it_recognizes_serbian_latin_as_latin_sub_language() {
        assert_eq!(
            super::from_lang_id(SERBIAN_LATIN).sub_language(),
            SubLanguage::Latin
        );
    }

    #[test]
    fn it_recognizes_sindhi_as_sindhi_language() {
        assert_eq!(
            super::from_lang_id(SINDHI).primary_language(),
            PrimaryLanguage::Sindhi
        );
    }

    #[test]
    fn it_recognizes_slovak_as_slovak_language() {
        assert_eq!(
            super::from_lang_id(SLOVAK).primary_language(),
            PrimaryLanguage::Slovak
        );
    }

    #[test]
    fn it_recognizes_slovenian_as_slovenian_language() {
        assert_eq!(
            super::from_lang_id(SLOVENIAN).primary_language(),
            PrimaryLanguage::Slovenian
        );
    }

    #[test]
    fn it_recognizes_spanish_traditional_sort_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_TRADITIONAL_SORT).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_traditional_sort_as_traditional_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_TRADITIONAL_SORT).sub_language(),
            SubLanguage::Traditional
        );
    }

    #[test]
    fn it_recognizes_spanish_from_mexico_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_MEXICAN).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_mexico_as_mexico_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_MEXICAN).sub_language(),
            SubLanguage::Mexico
        );
    }

    #[test]
    fn it_recognizes_spanish_modern_sort_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_MODERN_SORT).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_modern_sort_as_modern_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_MODERN_SORT).sub_language(),
            SubLanguage::Modern
        );
    }

    #[test]
    fn it_recognizes_spanish_from_guatemala_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_GUATEMALA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_guatemala_as_guatemala_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_GUATEMALA).sub_language(),
            SubLanguage::Guatemala
        );
    }

    #[test]
    fn it_recognizes_spanish_from_costa_rica_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_COSTA_RICA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_costa_rica_as_costa_rica_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_COSTA_RICA).sub_language(),
            SubLanguage::CostaRica
        );
    }

    #[test]
    fn it_recognizes_spanish_from_panama_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PANAMA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_panama_as_panama_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PANAMA).sub_language(),
            SubLanguage::Panama
        );
    }

    #[test]
    fn it_recognizes_spanish_from_dominican_republic_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_DOMINICAN_REPUBLIC).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_dominican_republic_as_dominican_republic_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_DOMINICAN_REPUBLIC).sub_language(),
            SubLanguage::DominicanRepublic
        );
    }

    #[test]
    fn it_recognizes_spanish_from_venezuela_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_VENEZUELA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_venezuela_as_venezuela_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_VENEZUELA).sub_language(),
            SubLanguage::Venezuela
        );
    }

    #[test]
    fn it_recognizes_spanish_from_colombia_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_COLOMBIA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_colombia_as_colombia_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_COLOMBIA).sub_language(),
            SubLanguage::Colombia
        );
    }

    #[test]
    fn it_recognizes_spanish_from_peru_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PERU).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_peru_as_peru_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PERU).sub_language(),
            SubLanguage::Peru
        );
    }

    #[test]
    fn it_recognizes_spanish_from_argentina_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_ARGENTINA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_argentina_as_argentina_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_ARGENTINA).sub_language(),
            SubLanguage::Argentina
        );
    }

    #[test]
    fn it_recognizes_spanish_from_ecuador_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_ECUADOR).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_ecuador_as_ecuador_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_ECUADOR).sub_language(),
            SubLanguage::Ecuador
        );
    }

    #[test]
    fn it_recognizes_spanish_from_chile_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_CHILE).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_chile_as_chile_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_CHILE).sub_language(),
            SubLanguage::Chile
        );
    }

    #[test]
    fn it_recognizes_spanish_from_uruguay_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_URUGUAY).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_uruguay_as_uruguay_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_URUGUAY).sub_language(),
            SubLanguage::Uruguay
        );
    }

    #[test]
    fn it_recognizes_spanish_from_paraguay_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PARAGUAY).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_paraguay_as_paraguay_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PARAGUAY).sub_language(),
            SubLanguage::Paraguay
        );
    }

    #[test]
    fn it_recognizes_spanish_from_bolivia_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_BOLIVIA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_bolivia_as_bolivia_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_BOLIVIA).sub_language(),
            SubLanguage::Bolivia
        );
    }

    #[test]
    fn it_recognizes_spanish_from_el_salvador_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_EL_SALVADOR).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_el_salvador_as_el_salvador_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_EL_SALVADOR).sub_language(),
            SubLanguage::ElSalvador
        );
    }

    #[test]
    fn it_recognizes_spanish_from_honduras_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_HONDURAS).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_honduras_as_honduras_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_HONDURAS).sub_language(),
            SubLanguage::Honduras
        );
    }

    #[test]
    fn it_recognizes_spanish_from_nicaragua_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_NICARAGUA).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_nicaragua_as_nicaragua_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_NICARAGUA).sub_language(),
            SubLanguage::Nicaragua
        );
    }

    #[test]
    fn it_recognizes_spanish_from_puerto_rico_as_spanish_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PUERTO_RICO).primary_language(),
            PrimaryLanguage::Spanish
        );
    }

    #[test]
    fn it_recognizes_spanish_from_puerto_rico_as_puerto_rico_sub_language() {
        assert_eq!(
            super::from_lang_id(SPANISH_PUERTO_RICO).sub_language(),
            SubLanguage::PuertoRico
        );
    }

    #[test]
    fn it_recognizes_sutu_as_sutu_language() {
        assert_eq!(
            super::from_lang_id(SUTU).primary_language(),
            PrimaryLanguage::Sutu
        );
    }

    #[test]
    fn it_recognizes_swahili_as_swahili_language() {
        assert_eq!(
            super::from_lang_id(SWAHILI_KENYA).primary_language(),
            PrimaryLanguage::Swahili
        );
    }

    #[test]
    fn it_recognizes_swedish_as_swedish_language() {
        assert_eq!(
            super::from_lang_id(SWEDISH).primary_language(),
            PrimaryLanguage::Swedish
        );
    }

    #[test]
    fn it_recognizes_swedish_as_standard_sub_language() {
        assert_eq!(
            super::from_lang_id(SWEDISH).sub_language(),
            SubLanguage::Standard
        );
    }

    #[test]
    fn it_recognizes_swedish_from_finland_as_swedish_language() {
        assert_eq!(
            super::from_lang_id(SWEDISH_FINLAND).primary_language(),
            PrimaryLanguage::Swedish
        );
    }

    #[test]
    fn it_recognizes_swedish_from_finland_as_finland_sub_language() {
        assert_eq!(
            super::from_lang_id(SWEDISH_FINLAND).sub_language(),
            SubLanguage::Finland
        );
    }

    #[test]
    fn it_recognizes_tamil_as_tamil_language() {
        assert_eq!(
            super::from_lang_id(TAMIL).primary_language(),
            PrimaryLanguage::Tamil
        );
    }

    #[test]
    fn it_recognizes_tatar_as_tatar_language() {
        assert_eq!(
            super::from_lang_id(TATAR_TATARSTAN).primary_language(),
            PrimaryLanguage::Tatar
        );
    }

    #[test]
    fn it_recognizes_telugu_as_telugu_language() {
        assert_eq!(
            super::from_lang_id(TELUGU).primary_language(),
            PrimaryLanguage::Telugu
        );
    }

    #[test]
    fn it_recognizes_thai_as_thai_language() {
        assert_eq!(
            super::from_lang_id(THAI).primary_language(),
            PrimaryLanguage::Thai
        );
    }

    #[test]
    fn it_recognizes_turkish_as_turkish_language() {
        assert_eq!(
            super::from_lang_id(TURKISH).primary_language(),
            PrimaryLanguage::Turkish
        );
    }

    #[test]
    fn it_recognizes_ukrainian_as_ukrainian_language() {
        assert_eq!(
            super::from_lang_id(UKRAINIAN).primary_language(),
            PrimaryLanguage::Ukrainian
        );
    }

    #[test]
    fn it_recognizes_urdu_from_pakistan_as_urdu_language() {
        assert_eq!(
            super::from_lang_id(URDU_PAKISTAN).primary_language(),
            PrimaryLanguage::Urdu
        );
    }

    #[test]
    fn it_recognizes_urdu_from_pakistan_as_pakistan_sub_language() {
        assert_eq!(
            super::from_lang_id(URDU_PAKISTAN).sub_language(),
            SubLanguage::Pakistan
        );
    }

    #[test]
    fn it_recognizes_urdu_from_india_as_urdu_language() {
        assert_eq!(
            super::from_lang_id(URDU_INDIA).primary_language(),
            PrimaryLanguage::Urdu
        );
    }

    #[test]
    fn it_recognizes_urdu_from_india_as_india_sub_language() {
        assert_eq!(
            super::from_lang_id(URDU_INDIA).sub_language(),
            SubLanguage::India
        );
    }

    #[test]
    fn it_recognizes_uzbek_latin_as_uzbek_language() {
        assert_eq!(
            super::from_lang_id(UZBEK_LATIN).primary_language(),
            PrimaryLanguage::Uzbek
        );
    }

    #[test]
    fn it_recognizes_uzbek_latin_as_latin_sub_language() {
        assert_eq!(
            super::from_lang_id(UZBEK_LATIN).sub_language(),
            SubLanguage::Latin
        );
    }

    #[test]
    fn it_recognizes_uzbek_cyrillic_as_uzbek_language() {
        assert_eq!(
            super::from_lang_id(UZBEK_CYRILLIC).primary_language(),
            PrimaryLanguage::Uzbek
        );
    }

    #[test]
    fn it_recognizes_uzbek_cyrillic_as_cyrillic_sub_language() {
        assert_eq!(
            super::from_lang_id(UZBEK_CYRILLIC).sub_language(),
            SubLanguage::Cyrillic
        );
    }

    #[test]
    fn it_recognizes_vietnamese_as_vietnamese_language() {
        assert_eq!(
            super::from_lang_id(VIETNAMESE).primary_language(),
            PrimaryLanguage::Vietnamese
        );
    }

    #[test]
    fn it_recognizes_hid_usage_data_descriptor_as_hid_language() {
        assert_eq!(
            super::from_lang_id(HID_USAGE_DATA_DESCRIPTOR).primary_language(),
            PrimaryLanguage::HID
        );
    }

    #[test]
    fn it_recognizes_hid_usage_data_descriptor_as_usage_data_descriptor_sub_language() {
        assert_eq!(
            super::from_lang_id(HID_USAGE_DATA_DESCRIPTOR).sub_language(),
            SubLanguage::UsageDataDescriptor
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_1_as_hid_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_1).primary_language(),
            PrimaryLanguage::HID
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_1_as_vendor_defined_1_sub_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_1).sub_language(),
            SubLanguage::VendorDefined1
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_2_as_hid_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_2).primary_language(),
            PrimaryLanguage::HID
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_1_as_vendor_defined_2_sub_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_2).sub_language(),
            SubLanguage::VendorDefined2
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_3_as_hid_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_3).primary_language(),
            PrimaryLanguage::HID
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_1_as_vendor_defined_3_sub_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_3).sub_language(),
            SubLanguage::VendorDefined3
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_4_as_hid_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_4).primary_language(),
            PrimaryLanguage::HID
        );
    }

    #[test]
    fn it_recognizes_hid_vendor_defined_1_as_vendor_defined_4_sub_language() {
        assert_eq!(
            super::from_lang_id(HID_VENDOR_DEFINED_4).sub_language(),
            SubLanguage::VendorDefined4
        );
    }

    #[test]
    fn it_recognizes_other_as_other_language() {
        assert_eq!(
            super::from_lang_id(0xFFFF).primary_language(),
            PrimaryLanguage::Other(PRIMARY_LANGUAGE_MASK)
        );
    }

    #[test]
    fn it_recognizes_other_as_other_sub_language() {
        assert_eq!(
            super::from_lang_id(0xFFFF).sub_language(),
            SubLanguage::Other(SUB_LANGUAGE_MASK)
        );
    }
}
