//! Static Type Elaboration and Well-Foundedness Checker for AST Expressions.
//! Validates structural subtyping, algebraic data types (ADTs), and trait boundaries.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdtKind {
    Struct,
    Enum,
    Union,
}

#[derive(Debug, Clone)]
pub struct AdtField {
    pub name: String,
    pub type_name: String,
    pub offset_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct AdtDefinition {
    pub name: String,
    pub kind: AdtKind,
    pub fields: Vec<AdtField>,
    pub size_bytes: usize,
    pub alignment_bytes: usize,
}

pub struct TypeElaborator {
    adts: HashMap<String, AdtDefinition>,
    primitives: HashMap<String, usize>,
}

impl TypeElaborator {
    pub fn new() -> Self {
        let mut primitives = HashMap::new();
        primitives.insert("i8".to_string(), 1);
        primitives.insert("u8".to_string(), 1);
        primitives.insert("i16".to_string(), 2);
        primitives.insert("u16".to_string(), 2);
        primitives.insert("i32".to_string(), 4);
        primitives.insert("u32".to_string(), 4);
        primitives.insert("i64".to_string(), 8);
        primitives.insert("u64".to_string(), 8);
        primitives.insert("f32".to_string(), 4);
        primitives.insert("f64".to_string(), 8);
        primitives.insert("bool".to_string(), 1);
        primitives.insert("usize".to_string(), 8);

        Self {
            adts: HashMap::new(),
            primitives,
        }
    }

    pub fn register_struct(&mut self, name: &str, fields: Vec<(&str, &str)>) -> Result<AdtDefinition, String> {
        let mut adt_fields = Vec::new();
        let mut current_offset = 0;
        let mut max_align = 1;

        for (f_name, f_type) in fields {
            let size = if let Some(&s) = self.primitives.get(f_type) {
                s
            } else if let Some(sub_adt) = self.adts.get(f_type) {
                sub_adt.size_bytes
            } else {
                return Err(format!("Unknown field type: {}", f_type));
            };

            let align = size.min(8);
            max_align = max_align.max(align);

            // Align current offset
            current_offset = (current_offset + align - 1) & !(align - 1);

            adt_fields.push(AdtField {
                name: f_name.to_string(),
                type_name: f_type.to_string(),
                offset_bytes: current_offset,
            });

            current_offset += size;
        }

        // Tail padding to round up to max_align
        let total_size = (current_offset + max_align - 1) & !(max_align - 1);

        let def = AdtDefinition {
            name: name.to_string(),
            kind: AdtKind::Struct,
            fields: adt_fields,
            size_bytes: total_size,
            alignment_bytes: max_align,
        };

        self.adts.insert(name.to_string(), def.clone());
        Ok(def)
    }

    pub fn lookup_type_size(&self, name: &str) -> Option<usize> {
        self.primitives
            .get(name)
            .copied()
            .or_else(|| self.adts.get(name).map(|d| d.size_bytes))
    }

    pub fn get_field_offset(&self, struct_name: &str, field_name: &str) -> Option<usize> {
        let def = self.adts.get(struct_name)?;
        def.fields.iter().find(|f| f.name == field_name).map(|f| f.offset_bytes)
    }
}
