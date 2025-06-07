#![allow(unused_variables)]
#![allow(dead_code)]

macro_rules! typed_enum {
     ($enum:ident {
        $(
            $item:ident $( ( $($sub_item:ty),+ ) )?
        ),* $(,)?
    }) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            struct $item $( ( $($sub_item),+ ) )?;
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

macro_rules! property_enum {
    ($property:ident {
        $(
            $value:ident $( ( $($sub_item:ty),+ ) )?
        ),* $(,)?
    }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct $property;
        impl std::fmt::Display for $property {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!($property))
            }
        }
        paste::paste! {
            typed_enum!([<$property Value>] {
                $(
                    $value $( ( $($sub_item),+ ) )?
                ),*
            });
            impl Property for $property {
                type Value = [<$property Value>];
            }
        }

    };
}

pub trait Property {
    type Value: Copy + std::fmt::Display;
}

struct InfectionData {
    t: f64,
}

property_enum!(InfectionStatus {
    Susceptible,
    Infected(InfectionData),
    Recovered,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PersonId(usize);

struct Context {}
impl Context {
    pub fn new() -> Self {
        Context {}
    }
    // Use impl Into<P::Value> so we can use the individual variant types
    fn set_property<P: Property, V: Into<P::Value>>(
        &mut self,
        person: PersonId,
        property: P,
        v: V,
    ) {
        let value: P::Value = v.into();
        println!("Setting property to value: {}", value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Age;
impl Property for Age {
    type Value = u32;
}

fn main() {
    let mut context = Context::new();
    let person = PersonId(1);
    // Annoyingly, Rust assumes 30 is an i32 because it can no longer infer the type
    context.set_property(person, Age, 30_u32);
    context.set_property(person, InfectionStatus, Susceptible);
    context.set_property(person, InfectionStatus, Infected(InfectionData { t: 1.0 }));
    context.set_property(
        person,
        InfectionStatus,
        InfectionStatusValue::from(Recovered),
    );
}
