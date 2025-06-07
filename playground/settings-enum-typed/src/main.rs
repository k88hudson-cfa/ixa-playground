#![allow(unused_variables)]
#![allow(dead_code)]

macro_rules! typed_enum {
    (
        enum $enum:ident { $($item:ident),+ $(,)? }
    ) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            struct $item;
        )+
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum $enum {
            $(
                $item($item),
            )+
        }
        impl std::fmt::Display for $enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $enum::$item(inner) => write!(f, "{}", inner),
                    )+
                }
            }
        }
        $(
            impl std::fmt::Display for $item {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", stringify!($item))
                }
            }
            impl From<$item> for $enum {
                fn from(value: $item) -> Self {
                    $enum::$item(value)
                }
            }

            impl From<$enum> for $item {
                fn from(value: $enum) -> Self {
                    match value {
                        $enum::$item(inner) => inner,
                        _ => panic!("Cannot convert {}::{} to {}", stringify!($enum), value, stringify!($item)),
                    }
                }
            }

        )+

    }
}

pub trait Setting {
    fn get_contact(&self) -> usize;
}
typed_enum!(
    enum SettingEnum {
        Home,
        Work,
    }
);
impl Work {
    fn get_floors(&self) -> usize {
        2
    }
}

impl Home {
    fn get_rooms(&self) -> usize {
        3
    }
}
impl Setting for Home {
    fn get_contact(&self) -> usize {
        1
    }
}
impl Setting for Work {
    fn get_contact(&self) -> usize {
        1
    }
}
// I guess we have to do this with a derive macro
impl Setting for SettingEnum {
    fn get_contact(&self) -> usize {
        match self {
            SettingEnum::Home(home) => home.get_contact(),
            SettingEnum::Work(work) => work.get_contact(),
        }
    }
}

fn main() {
    let home = SettingEnum::from(Home);
    let work = SettingEnum::from(Work);

    fn print_setting(setting: SettingEnum) {
        println!("Contact: {}", setting.get_contact());
    }

    print_setting(home);
    print_setting(work);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_enum_get_contact() {
        let home = SettingEnum::from(Home);
        let work = SettingEnum::from(Work);

        assert_eq!(home.get_contact(), 1);
        assert_eq!(work.get_contact(), 1);
    }

    #[test]
    fn test_home_get_rooms() {
        let home = SettingEnum::from(Home);
        let home_value = Home::from(home);
        assert_eq!(home_value.get_rooms(), 2);
    }

    #[test]
    fn test_work_get_floors() {
        let work = SettingEnum::from(Work);
        let work_value = Work::from(work);
        assert_eq!(work_value.get_floors(), 3);
    }

    #[test]
    #[should_panic(expected = "Cannot convert SettingEnum::Home to Work")]
    fn test_wrong_enum_variant_panics_on_from() {
        let home = SettingEnum::from(Home);
        let _ = Work::from(home);
    }
}
