use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Default)]
pub enum Patch<T> {
    #[default]
    None,
    Null,
    Value(T),
}

impl<T> Patch<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, Patch::None)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Patch::Null)
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Patch::Value(_))
    }

    pub fn ok(&self) -> Option<&T> {
        if let Patch::Value(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn ok_or<E>(&self, err: E) -> Result<&T, E> {
        if let Patch::Value(v) = self {
            Ok(v)
        } else {
            Err(err)
        }
    }

    pub fn or_else(self, default: T) -> T {
        match self {
            Patch::Value(v) => v,
            _ => default,
        }
    }

    pub fn apply(&self, target: &mut Option<T>)
    where
        T: Clone,
    {
        match self {
            Patch::Value(v) => *target = Some(v.clone()),
            Patch::Null => *target = None,
            _ => {}
        }
    }
}

impl<T> From<Option<T>> for Patch<T> {
    fn from(opt: Option<T>) -> Patch<T> {
        match opt {
            Some(v) => Patch::Value(v),
            None => Patch::Null,
        }
    }
}

impl<T> From<Patch<T>> for Option<T> {
    fn from(patch: Patch<T>) -> Option<T> {
        match patch {
            Patch::Value(v) => Some(v),
            _ => None,
        }
    }
}

impl<T: Clone> From<&Patch<T>> for Option<T> {
    fn from(patch: &Patch<T>) -> Option<T> {
        match patch {
            Patch::Value(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl<T: Serialize> Serialize for Patch<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Patch::Value(v) => v.serialize(serializer),
            _ => serializer.serialize_none(),
        }
    }
}

impl<'de, T> Deserialize<'de> for Patch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}
