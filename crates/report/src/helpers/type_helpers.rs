//! Helpers for building diagnostic codes from type names.
use crate::prelude::*;

/// Build a short diagnostic code from `type_name::<T>()` and the context's `Debug` output.
///
/// - Enum contexts: `crate::EnumName::Variant`
/// - Struct contexts: `crate::StructName`
pub fn short_code<T: Debug>(context: &T) -> String {
    let full = type_name::<T>();
    let segments: Vec<&str> = full.split("::").collect();
    let crate_name = segments
        .first()
        .expect("split should yield at least one segment");
    let type_segment = segments
        .last()
        .expect("split should yield at least one segment");
    let debug = format!("{context:?}");
    let first_word = debug
        .split([' ', '(', '{'])
        .next()
        .expect("split should yield at least one segment");
    let is_enum_variant = first_word != *type_segment;
    if is_enum_variant {
        format!("{crate_name}::{type_segment}::{first_word}")
    } else {
        format!("{crate_name}::{type_segment}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_code__enum_variant() {
        // Act
        let code = short_code(&OuterError::Operation);
        // Assert
        assert_eq!(code, "studiole_report::OuterError::Operation");
    }

    #[test]
    fn short_code__unit_struct() {
        // Act
        let code = short_code(&UnitError);
        // Assert
        assert_eq!(code, "studiole_report::UnitError");
    }
}
