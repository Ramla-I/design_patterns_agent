use crate::navigation::{CodeContext, InterestingItem};
use crate::parser::{ImplBlock, StructDef};

/// Extract code evidence for invariant detection
pub struct EvidenceExtractor;

impl EvidenceExtractor {
    pub fn extract_code_snippet(context: &CodeContext) -> String {
        match &context.item {
            InterestingItem::TypeStateCandidate { struct_def, impl_blocks } => {
                Self::format_typestate(struct_def, impl_blocks)
            }
            InterestingItem::LinearTypeCandidate { struct_def, impl_blocks } => {
                Self::format_linear_type(struct_def, impl_blocks)
            }
            InterestingItem::StateTransition { impl_block } => {
                Self::format_impl_block(impl_block)
            }
            InterestingItem::Generic { .. } => {
                "// Generic interesting item".to_string()
            }
        }
    }

    fn format_typestate(struct_def: &StructDef, impl_blocks: &[ImplBlock]) -> String {
        let mut output = String::new();

        // Format the struct
        output.push_str(&format!("pub struct {}{} {{\n", struct_def.name, struct_def.generics));
        for field in &struct_def.fields {
            output.push_str(&format!("    {}: {},\n", field.name, field.ty));
        }
        output.push_str("}\n\n");

        // Format relevant impl blocks
        for impl_block in impl_blocks {
            output.push_str(&Self::format_impl_block(impl_block));
            output.push_str("\n");
        }

        output
    }

    fn format_linear_type(struct_def: &StructDef, impl_blocks: &[ImplBlock]) -> String {
        let mut output = String::new();

        // Format the struct
        output.push_str(&format!("pub struct {} {{\n", struct_def.name));
        for field in &struct_def.fields {
            output.push_str(&format!("    {}: {},\n", field.name, field.ty));
        }
        output.push_str("}\n\n");

        // Format Drop impl if present
        for impl_block in impl_blocks {
            if impl_block.trait_name.as_ref().map_or(false, |t| t.contains("Drop")) {
                output.push_str(&Self::format_impl_block(impl_block));
                output.push_str("\n");
            }
        }

        output
    }

    fn format_impl_block(impl_block: &ImplBlock) -> String {
        let mut output = String::new();

        if let Some(trait_name) = &impl_block.trait_name {
            output.push_str(&format!("impl {} for {} {{\n", trait_name, impl_block.type_name));
        } else {
            output.push_str(&format!("impl {} {{\n", impl_block.type_name));
        }

        for method in &impl_block.methods {
            output.push_str(&format!("    {} {{ ... }}\n", method.signature));
        }

        output.push_str("}");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{FunctionDef, SelfParam, SourceLocation, Visibility};

    #[test]
    fn test_extract_typestate_snippet() {
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

        let impl_block = ImplBlock {
            trait_name: None,
            type_name: "State<Open>".to_string(),
            methods: vec![FunctionDef {
                name: "read".to_string(),
                signature: "fn read(&self)".to_string(),
                is_method: true,
                self_param: Some(SelfParam::Reference),
                visibility: Visibility::Public,
                doc_comment: None,
                source_location: SourceLocation {
                    file_path: "test.rs".to_string(),
                    line: 10,
                },
            }],
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 8,
            },
        };

        let context = CodeContext {
            item: InterestingItem::TypeStateCandidate {
                struct_def,
                impl_blocks: vec![impl_block],
            },
            surrounding_code: String::new(),
            module_path: "test".to_string(),
        };

        let snippet = EvidenceExtractor::extract_code_snippet(&context);

        assert!(snippet.contains("pub struct State<S>"));
        assert!(snippet.contains("impl State<Open>"));
        assert!(snippet.contains("fn read(&self)"));
    }

    #[test]
    fn test_format_impl_block() {
        let impl_block = ImplBlock {
            trait_name: Some("Drop".to_string()),
            type_name: "Resource".to_string(),
            methods: vec![FunctionDef {
                name: "drop".to_string(),
                signature: "fn drop(&mut self)".to_string(),
                is_method: true,
                self_param: Some(SelfParam::MutReference),
                visibility: Visibility::Public,
                doc_comment: None,
                source_location: SourceLocation {
                    file_path: "test.rs".to_string(),
                    line: 20,
                },
            }],
            source_location: SourceLocation {
                file_path: "test.rs".to_string(),
                line: 18,
            },
        };

        let formatted = EvidenceExtractor::format_impl_block(&impl_block);

        assert!(formatted.contains("impl Drop for Resource"));
        assert!(formatted.contains("fn drop(&mut self)"));
    }
}
