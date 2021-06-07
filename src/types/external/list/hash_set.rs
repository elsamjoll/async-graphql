use std::borrow::Cow;
use std::cmp::Eq;
use std::collections::HashSet;
use std::hash::Hash;

use crate::parser::types::Field;
use crate::resolver_utils::resolve_list;
use crate::{
    registry, ContextSelectionSet, InputType, InputValueError, InputValueResult, OutputType,
    Positioned, Result, Type, Value,
};

impl<T: Type> Type for HashSet<T> {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", T::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }
}

impl<T: InputType + Hash + Eq> InputType for HashSet<T> {
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::List(values) => values
                .into_iter()
                .map(|value| InputType::parse(Some(value)))
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate),
            value => Ok({
                let mut result = Self::default();
                result.insert(InputType::parse(Some(value)).map_err(InputValueError::propagate)?);
                result
            }),
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.iter().map(InputType::to_value).collect())
    }
}

#[async_trait::async_trait]
impl<T: OutputType + Hash + Eq> OutputType for HashSet<T> {
    async fn resolve(&self, ctx: &ContextSelectionSet<'_>, field: &Positioned<Field>) -> Value {
        resolve_list(ctx, field, self, Some(self.len())).await
    }
}
