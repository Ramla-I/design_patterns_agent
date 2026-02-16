use crate::parser::{CodeItem, ImplBlock, StructDef};

/// An "interesting" code item that warrants invariant analysis
#[derive(Debug, Clone)]
pub enum InterestingItem {
    /// A struct with its associated impl blocks - analyze for potential invariants
    StructWithImpls {
        struct_def: StructDef,
        impl_blocks: Vec<ImplBlock>,
    },
    /// A standalone impl block (for types we don't have struct definitions for)
    StandaloneImpl {
        impl_block: ImplBlock,
    },
    /// A struct with PhantomData (potential typestate) - legacy
    TypeStateCandidate {
        struct_def: StructDef,
        impl_blocks: Vec<ImplBlock>,
    },
    /// A struct with Drop implementation (potential linear type) - legacy
    LinearTypeCandidate {
        struct_def: StructDef,
        impl_blocks: Vec<ImplBlock>,
    },
    /// Methods that consume self (potential state transition) - legacy
    StateTransition {
        impl_block: ImplBlock,
    },
    /// Generic interesting construct
    Generic {
        item: CodeItem,
        reason: String,
    },
}

/// Code context extracted for LLM analysis
#[derive(Debug, Clone)]
pub struct CodeContext {
    pub item: InterestingItem,
    pub surrounding_code: String,
    pub module_path: String,
}

impl InterestingItem {
    /// Check if a code item is "interesting" for invariant analysis
    /// Now analyzes ALL structs and impl blocks to infer invariants from non-idiomatic code
    pub fn from_code_item(item: &CodeItem, related_impls: &[ImplBlock]) -> Option<Self> {
        match item {
            // Analyze ALL structs - we want to infer invariants from any code
            CodeItem::Struct(s) => {
                // Skip trivial structs with no fields and no methods
                let has_fields = !s.fields.is_empty();
                let has_methods = !related_impls.is_empty() &&
                    related_impls.iter().any(|i| !i.methods.is_empty());

                if has_fields || has_methods {
                    Some(InterestingItem::StructWithImpls {
                        struct_def: s.clone(),
                        impl_blocks: related_impls.to_vec(),
                    })
                } else {
                    None
                }
            }
            // Analyze impl blocks for types we may not have struct definitions for
            CodeItem::Impl(impl_block) if !impl_block.methods.is_empty() => {
                Some(InterestingItem::StandaloneImpl {
                    impl_block: impl_block.clone(),
                })
            }
            _ => None,
        }
    }

    /// Get a description of why this item is interesting
    pub fn reason(&self) -> String {
        match self {
            InterestingItem::StructWithImpls { struct_def, impl_blocks } => {
                let method_count: usize = impl_blocks.iter().map(|i| i.methods.len()).sum();
                format!(
                    "Struct '{}' with {} fields and {} methods - analyzing for implicit invariants",
                    struct_def.name,
                    struct_def.fields.len(),
                    method_count
                )
            }
            InterestingItem::StandaloneImpl { impl_block } => {
                format!(
                    "Impl block for '{}' with {} methods - analyzing for ordering/state invariants",
                    impl_block.type_name,
                    impl_block.methods.len()
                )
            }
            InterestingItem::TypeStateCandidate { .. } => {
                "Contains PhantomData, potential typestate pattern".to_string()
            }
            InterestingItem::LinearTypeCandidate { .. } => {
                "Has Drop implementation, potential linear type".to_string()
            }
            InterestingItem::StateTransition { .. } => {
                "Contains methods that consume self, potential state transition".to_string()
            }
            InterestingItem::Generic { reason, .. } => reason.clone(),
        }
    }
}

fn has_drop_impl(struct_def: &StructDef, impl_blocks: &[ImplBlock]) -> bool {
    impl_blocks.iter().any(|impl_block| {
        impl_block.type_name.contains(&struct_def.name)
            && impl_block
                .trait_name
                .as_ref()
                .map_or(false, |t| t.contains("Drop"))
    })
}

fn has_consuming_methods(impl_block: &ImplBlock) -> bool {
    impl_block.methods.iter().any(|m| {
        m.self_param
            .as_ref()
            .map_or(false, |sp| *sp == crate::parser::SelfParam::Owned)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{FunctionDef, SelfParam, SourceLocation, Visibility};

    #[test]
    fn test_typestate_candidate_detection() {
        let struct_def = StructDef {
            name: "State".to_string(),
            generics: "<S>".to_string(),
            fields: vec![],
            visibility: Visibility::Public,
            doc_comment: None,
            has_phantom_data: true,
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 1,
            },
        };

        let code_item = CodeItem::Struct(struct_def);
        let interesting = InterestingItem::from_code_item(&code_item, &[]);

        assert!(matches!(
            interesting,
            Some(InterestingItem::TypeStateCandidate { .. })
        ));
    }

    #[test]
    fn test_linear_type_candidate_detection() {
        let struct_def = StructDef {
            name: "Resource".to_string(),
            generics: "".to_string(),
            fields: vec![],
            visibility: Visibility::Public,
            doc_comment: None,
            has_phantom_data: false,
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 1,
            },
        };

        let impl_block = ImplBlock {
            trait_name: Some("Drop".to_string()),
            type_name: "Resource".to_string(),
            methods: vec![],
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 10,
            },
        };

        let code_item = CodeItem::Struct(struct_def);
        let interesting = InterestingItem::from_code_item(&code_item, &[impl_block]);

        assert!(matches!(
            interesting,
            Some(InterestingItem::LinearTypeCandidate { .. })
        ));
    }

    #[test]
    fn test_state_transition_detection() {
        let consuming_method = FunctionDef {
            name: "consume".to_string(),
            signature: "fn consume(self)".to_string(),
            is_method: true,
            self_param: Some(SelfParam::Owned),
            visibility: Visibility::Public,
            doc_comment: None,
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 20,
            },
        };

        let impl_block = ImplBlock {
            trait_name: None,
            type_name: "Foo".to_string(),
            methods: vec![consuming_method],
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 18,
            },
        };

        let code_item = CodeItem::Impl(impl_block);
        let interesting = InterestingItem::from_code_item(&code_item, &[]);

        assert!(matches!(
            interesting,
            Some(InterestingItem::StateTransition { .. })
        ));
    }

    #[test]
    fn test_reason_description() {
        let struct_def = StructDef {
            name: "State".to_string(),
            generics: "".to_string(),
            fields: vec![],
            visibility: Visibility::Public,
            doc_comment: None,
            has_phantom_data: true,
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 1,
            },
        };

        let interesting = InterestingItem::TypeStateCandidate {
            struct_def,
            impl_blocks: vec![],
        };

        let reason = interesting.reason();
        assert!(reason.contains("PhantomData"));
        assert!(reason.contains("typestate"));
    }
}
