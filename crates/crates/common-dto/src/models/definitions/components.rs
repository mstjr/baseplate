use crate::json::Patch;
use crate::models::definitions::SelectDisplay;
use common_core::DefinitionContext;
use common_core::definitions::DefinitionDisplay;
use common_core::keys::Key;
use serde::Deserialize;
use uuid::Uuid;

/* #region SelectDisplayModel */
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct SelectDisplayModel {
    pub option_id: Option<Uuid>,
    pub option_api_name: Option<String>,
    pub display_value: Option<String>,
    pub color: Patch<String>,
}

impl SelectDisplayModel {
    pub fn to_select_display(&self) -> Result<SelectDisplay, String> {
        Ok(SelectDisplay {
            option_id: self.option_id.unwrap_or_else(Uuid::now_v7),
            option_api_name: self
                .option_api_name
                .clone()
                .ok_or_else(|| "Option API name must be provided for select display".to_string())?,
            display_value: self
                .display_value
                .clone()
                .ok_or_else(|| "Display value must be provided for select display".to_string())?,
            color: match self.color.clone() {
                Patch::Value(v) => Some(v),
                _ => None,
            },
        })
    }

    pub fn update_select_display(&self, existing: &mut SelectDisplay) {
        if let Some(option_api_name) = &self.option_api_name {
            existing.option_api_name = option_api_name.clone();
        }
        if let Some(display_value) = &self.display_value {
            existing.display_value = display_value.clone();
        }
        match self.color.clone() {
            Patch::Value(v) => existing.color = Some(v),
            Patch::Null => existing.color = None,
            Patch::Missing => {}
        }
    }
}
/* #endregion */

/* #region DefinitionDisplay */
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct DefinitionDisplayModel {
    pub definition: Option<Key>,
    pub display_field: Patch<Key>,
}

impl DefinitionDisplayModel {
    pub fn to_definition_display(
        &self,
        ctx: &DefinitionContext,
    ) -> Result<DefinitionDisplay, String> {
        let definition_key = self.definition.clone().ok_or_else(|| {
            "Definition key must be provided for reference field type".to_string()
        })?;

        let display_field_key: Option<Key> = self.display_field.clone().into();

        let (definition_id, definition) = ctx
            .get_definition_by_key(&definition_key)
            .ok_or_else(|| "Referenced definition not found".to_string())?;

        let display_field_id = if let Some(display_field_key) = display_field_key {
            definition
                .get_field_by_key(&display_field_key)
                .map(|(id, _)| id)
        } else {
            None
        };

        Ok(DefinitionDisplay {
            definition_id,
            display_field_id,
        })
    }
}

/* #endregion */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::Patch;

    #[test]
    fn test_select_display_model_to_select_display() {
        let model = SelectDisplayModel {
            option_id: Some(Uuid::now_v7()),
            option_api_name: Some("option_api".to_string()),
            display_value: Some("Option Display".to_string()),
            color: Patch::Value("#FF0000".to_string()),
        };

        let display = model.to_select_display().unwrap();
        assert_eq!(display.option_api_name, "option_api");
        assert_eq!(display.display_value, "Option Display");
        assert_eq!(display.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_select_display_model_update_select_display() {
        let mut existing = SelectDisplay {
            option_id: Uuid::now_v7(),
            option_api_name: "old_api".to_string(),
            display_value: "Old Display".to_string(),
            color: Some("#00FF00".to_string()),
        };

        let model = SelectDisplayModel {
            option_id: None,
            option_api_name: Some("new_api".to_string()),
            display_value: None,
            color: Patch::Null,
        };

        model.update_select_display(&mut existing);
        assert_eq!(existing.option_api_name, "new_api");
        assert_eq!(existing.display_value, "Old Display");
        assert_eq!(existing.color, None);
    }

    #[test]
    fn test_definition_display_model_to_definition_display() {
        let ctx = DefinitionContext::default();

        let model = DefinitionDisplayModel {
            definition: Some(Key::ApiName("TestDefinition".to_string())),
            display_field: Patch::Value(Key::ApiName("TestField".to_string())),
        };

        // This will fail because the context is empty, but we want to ensure the method is callable
        let result = model.to_definition_display(&ctx);
        assert!(result.is_err());
    }
}
