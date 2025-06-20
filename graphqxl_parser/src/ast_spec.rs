use crate::ast_import::parse_import;
use crate::parser::{GraphqxlParser, Rule};
use crate::utils::{already_defined_error, custom_error, unknown_rule_error};
use crate::{
    parse_block_def, parse_directive_def, parse_generic_block_def, parse_scalar, parse_schema,
    parse_union, BlockDef, DirectiveDef, GenericBlockDef, Identifier, OwnedSpan, Scalar, Schema,
    Union,
};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum DefType {
    Type(Identifier),
    GenericType(Identifier),
    Input(Identifier),
    GenericInput(Identifier),
    Enum(Identifier),
    Interface(Identifier),
    Scalar(Identifier),
    Union(Identifier),
    Directive(Identifier),
    Schema(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spec {
    pub types: HashMap<String, BlockDef>,
    pub generic_types: HashMap<String, GenericBlockDef>,
    pub inputs: HashMap<String, BlockDef>,
    pub generic_inputs: HashMap<String, GenericBlockDef>,
    pub enums: HashMap<String, BlockDef>,
    pub interfaces: HashMap<String, BlockDef>,
    pub scalars: HashMap<String, Scalar>,
    pub unions: HashMap<String, Union>,
    pub directives: HashMap<String, DirectiveDef>,
    pub schemas: HashMap<String, Schema>,
    pub order: Vec<DefType>,
}

impl Spec {
    fn new() -> Self {
        Self::default()
    }

    fn extend_identifier(id: &Identifier) -> Identifier {
        let mut clone = id.clone();
        clone.id = format!("{}__extend__{}", clone.id, uuid::Uuid::new_v4());
        clone
    }

    fn merge(&mut self, other: Spec) -> Result<(), Box<dyn Error>> {
        for el in other.order.into_iter() {
            match &el {
                DefType::Type(name) => {
                    if self.types.contains_key(&name.id)
                        || self.generic_types.contains_key(&name.id)
                    {
                        return Err(name.span.make_error("Duplicated type"));
                    }
                    self.types.insert(
                        name.id.to_string(),
                        other.types.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::GenericType(name) => {
                    if self.generic_types.contains_key(&name.id)
                        || self.types.contains_key(&name.id)
                    {
                        return Err(name.span.make_error("Duplicated type"));
                    }
                    self.generic_types.insert(
                        name.id.to_string(),
                        other.generic_types.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Input(name) => {
                    if self.inputs.contains_key(&name.id)
                        && self.generic_inputs.contains_key(&name.id)
                    {
                        return Err(name.span.make_error("Duplicated input"));
                    }
                    self.inputs.insert(
                        name.id.to_string(),
                        other.inputs.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::GenericInput(name) => {
                    if self.generic_inputs.contains_key(&name.id)
                        || self.inputs.contains_key(&name.id)
                    {
                        return Err(name.span.make_error("Duplicated input"));
                    }
                    self.generic_inputs.insert(
                        name.id.to_string(),
                        other.generic_inputs.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Enum(name) => {
                    if self.enums.contains_key(&name.id) {
                        return Err(name.span.make_error("Duplicated enum"));
                    }
                    self.enums.insert(
                        name.id.to_string(),
                        other.enums.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Interface(name) => {
                    if self.interfaces.contains_key(&name.id) {
                        return Err(name.span.make_error("Duplicated interface"));
                    }
                    self.interfaces.insert(
                        name.id.to_string(),
                        other.interfaces.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Scalar(name) => {
                    if self.scalars.contains_key(&name.id) {
                        return Err(name.span.make_error("Duplicated scalar"));
                    }
                    self.scalars.insert(
                        name.id.to_string(),
                        other.scalars.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Union(name) => {
                    if self.unions.contains_key(&name.id) {
                        return Err(name.span.make_error("Duplicated union"));
                    }
                    self.unions.insert(
                        name.id.to_string(),
                        other.unions.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Directive(name) => {
                    if self.directives.contains_key(&name.id) {
                        return Err(name.span.make_error("Duplicated directive"));
                    }
                    self.directives.insert(
                        name.id.to_string(),
                        other.directives.get(&name.id).unwrap().clone(),
                    );
                }
                DefType::Schema(name) => {
                    if let Some(schema) = self.schemas.get(name) {
                        return Err(schema.span.make_error("Schema defined multiple times"));
                    }
                    self.schemas
                        .insert(name.clone(), other.schemas.get(name).unwrap().clone());
                }
            }
            self.order.push(el);
        }
        Ok(())
    }

    fn add(&mut self, pair: Pair<Rule>, file: &str) -> Result<(), Box<dyn Error>> {
        match pair.as_rule() {
            Rule::schema_def => {
                let schema = parse_schema(pair.clone(), file)?;
                let id = "schema".to_string();
                if self.schemas.contains_key(&id) {
                    Err(Box::new(custom_error(pair, "schema is already defined")))
                } else {
                    self.schemas.insert(id.clone(), schema);
                    self.order.push(DefType::Schema(id));
                    Ok(())
                }
            }
            Rule::schema_ext => {
                let schema = parse_schema(pair, file)?;
                let id = Self::extend_identifier(&Identifier::from("schema")).id;
                self.schemas.insert(id.clone(), schema);
                self.order.push(DefType::Schema(id));
                Ok(())
            }
            Rule::type_def => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = block_def.name.clone();
                if self.types.contains_key(&id.id) || self.generic_types.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "type", &id.id)))
                } else {
                    self.types.insert(id.id.clone(), block_def);
                    self.order.push(DefType::Type(id));
                    Ok(())
                }
            }
            Rule::type_ext => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = Self::extend_identifier(&block_def.name);
                self.types.insert(id.id.clone(), block_def);
                self.order.push(DefType::Type(id));
                Ok(())
            }
            Rule::generic_type_def => {
                let generic_block_def = parse_generic_block_def(pair.clone(), file)?;
                let id = generic_block_def.name.clone();
                if self.generic_types.contains_key(&id.id) || self.types.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "type", &id.id)))
                } else {
                    self.generic_types.insert(id.id.clone(), generic_block_def);
                    self.order.push(DefType::GenericType(id));
                    Ok(())
                }
            }
            Rule::input_def => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = block_def.name.clone();
                if self.inputs.contains_key(&id.id) || self.generic_inputs.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "input", &id.id)))
                } else {
                    self.inputs.insert(id.id.clone(), block_def);
                    self.order.push(DefType::Input(id));
                    Ok(())
                }
            }
            Rule::input_ext => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = Self::extend_identifier(&block_def.name);
                self.inputs.insert(id.id.clone(), block_def);
                self.order.push(DefType::Input(id));
                Ok(())
            }
            Rule::generic_input_def => {
                let generic_block_def = parse_generic_block_def(pair.clone(), file)?;
                let id = generic_block_def.name.clone();
                if self.generic_inputs.contains_key(&id.id) || self.inputs.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "input", &id.id)))
                } else {
                    self.generic_inputs.insert(id.id.clone(), generic_block_def);
                    self.order.push(DefType::GenericInput(id));
                    Ok(())
                }
            }
            Rule::enum_def => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = block_def.name.clone();
                if self.enums.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "enum", &id.id)))
                } else {
                    self.enums.insert(id.id.clone(), block_def);
                    self.order.push(DefType::Enum(id));
                    Ok(())
                }
            }
            Rule::enum_ext => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = Self::extend_identifier(&block_def.name);
                self.enums.insert(id.id.clone(), block_def);
                self.order.push(DefType::Enum(id));
                Ok(())
            }
            Rule::interface_def => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = block_def.name.clone();
                if self.interfaces.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "interface", &id.id)))
                } else {
                    self.interfaces.insert(id.id.clone(), block_def);
                    self.order.push(DefType::Interface(id));
                    Ok(())
                }
            }
            Rule::interface_ext => {
                let block_def = parse_block_def(pair.clone(), file)?;
                let id = Self::extend_identifier(&block_def.name);
                self.interfaces.insert(id.id.clone(), block_def);
                self.order.push(DefType::Interface(id));
                Ok(())
            }
            Rule::scalar_def => {
                let scalar = parse_scalar(pair.clone(), file)?;
                let id = scalar.name.clone();
                if self.scalars.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "scalar", &id.id)))
                } else {
                    self.scalars.insert(id.id.clone(), scalar);
                    self.order.push(DefType::Scalar(id));
                    Ok(())
                }
            }
            Rule::scalar_ext => {
                let block_def = parse_scalar(pair.clone(), file)?;
                let id = Self::extend_identifier(&block_def.name);
                self.scalars.insert(id.id.clone(), block_def);
                self.order.push(DefType::Scalar(id));
                Ok(())
            }
            Rule::union_def => {
                let union = parse_union(pair.clone(), file)?;
                let id = union.name.clone();
                if self.unions.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "union", &id.id)))
                } else {
                    self.unions.insert(id.id.clone(), union);
                    self.order.push(DefType::Union(id));
                    Ok(())
                }
            }
            Rule::union_ext => {
                let block_def = parse_union(pair.clone(), file)?;
                let id = Self::extend_identifier(&block_def.name);
                self.unions.insert(id.id.clone(), block_def);
                self.order.push(DefType::Union(id));
                Ok(())
            }
            Rule::directive_def => {
                let directive = parse_directive_def(pair.clone(), file)?;
                let id = directive.name.clone();
                if self.directives.contains_key(&id.id) {
                    Err(Box::new(already_defined_error(pair, "directive", &id.id)))
                } else {
                    self.directives.insert(id.id.clone(), directive);
                    self.order.push(DefType::Directive(id));
                    Ok(())
                }
            }
            _unknown => Err(Box::new(unknown_rule_error(
                pair,
                "type, input, enum, interface, scalar, union, directive",
            ))),
        }
    }
}

fn check_import_loop(import_stack: &Vec<PathBuf>, span: &OwnedSpan) -> Result<(), Box<dyn Error>> {
    let mut seen = HashSet::new();
    for import in import_stack {
        let import_string = import.to_string_lossy();
        if seen.contains(&import_string) {
            let mut msg = "".to_string();
            let mut start_flag = false;
            for import in import_stack {
                if import.to_string_lossy() == import_string {
                    start_flag = true;
                    msg += &import_string;
                } else if start_flag {
                    msg += " -> ";
                    msg += &import_string;
                }
            }
            return Err(span.make_error(&format!("cyclical import {}", msg)));
        } else {
            seen.insert(import_string);
        }
    }
    Ok(())
}


/// Trait for a virtual file system.
pub trait VirtualFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, Box<dyn Error>>;
    fn canonicalize(&self, path: &Path) -> Result<PathBuf, Box<dyn Error>>;
    fn exists(&self, path: &Path) -> bool;
}

/// VFS-based version of private_parse_spec
fn private_parse_spec_vfs<P: AsRef<Path>>(
    path: P,
    import_stack: Vec<PathBuf>,
    already_imported: &mut HashSet<PathBuf>,
    vfs: &dyn VirtualFileSystem,
) -> Result<Spec, Box<dyn Error>> {
    let abs_path = vfs.canonicalize(path.as_ref())?;
    let file = abs_path.to_str().unwrap();

    let mut spec = Spec::new();
    if already_imported.contains(&abs_path) {
        return Ok(spec);
    }
    let content = vfs.read_to_string(&abs_path)?;
    let mut pairs = GraphqxlParser::parse(Rule::spec, &content)?;
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::spec => {
            for child in pair.into_inner() {
                if let Rule::EOI = &child.as_rule() {
                    // nothing to do here
                } else if let Rule::import = &child.as_rule() {
                    let import = parse_import(child.clone(), file)?;
                    let file_name = if import.file_name.ends_with(".graphqxl") {
                        import.file_name
                    } else {
                        import.file_name + ".graphqxl"
                    };
                    let file_dir = abs_path.parent().unwrap();
                    let import_path = Path::new(file_dir).join(&file_name);
                    if !vfs.exists(&import_path) {
                        return Err(import.span.make_error(
                            format!("file {:?} does not exist", import_path).as_str(),
                        ));
                    }
                    let mut stack = import_stack.clone();
                    stack.push(import_path.clone());
                    check_import_loop(&stack, &import.span)?;
                    let imported_spec =
                        private_parse_spec_vfs(import_path, stack, already_imported, vfs)?;
                    spec.merge(imported_spec)?;
                } else {
                    spec.add(child, file)?;
                }
            }
            already_imported.insert(abs_path.clone());
            Ok(spec)
        }
        _unknown => Err(Box::new(unknown_rule_error(pair, "spec"))),
    }
}
pub fn parse_spec_vfs<P: AsRef<Path>>(path: P, vfs: &dyn VirtualFileSystem) -> Result<Spec, Box<dyn Error>> {
    private_parse_spec_vfs(path, Vec::new(), &mut HashSet::new(), vfs)
}

fn private_parse_spec<P: AsRef<Path>>(
    path: P,
    import_stack: Vec<PathBuf>,
    already_imported: &mut HashSet<PathBuf>,
) -> Result<Spec, Box<dyn Error>> {
    let abs_path = fs::canonicalize(path)?;
    let file = abs_path.to_str().unwrap();

    let mut spec = Spec::new();
    if already_imported.contains(&abs_path) {
        return Ok(spec);
    }
    let content = fs::read_to_string(&abs_path)?;
    let mut pairs = GraphqxlParser::parse(Rule::spec, &content)?;
    let pair = pairs.next().unwrap();
    match pair.as_rule() {
        Rule::spec => {
            for child in pair.into_inner() {
                if let Rule::EOI = &child.as_rule() {
                    // nothing to do here
                } else if let Rule::import = &child.as_rule() {
                    let import = parse_import(child.clone(), file)?;
                    let file_name = if import.file_name.ends_with(".graphqxl") {
                        import.file_name
                    } else {
                        import.file_name + ".graphqxl"
                    };
                    let file_dir = abs_path.parent().unwrap();
                    let import_path = Path::new(file_dir).join(&file_name);
                    if !import_path.exists() {
                        return Err(import.span.make_error(
                            format!("file {:?} does not exist", import_path).as_str(),
                        ));
                    }
                    let mut stack = import_stack.clone();
                    stack.push(import_path.clone());
                    check_import_loop(&stack, &import.span)?;
                    let imported_spec = private_parse_spec(import_path, stack, already_imported)?;
                    spec.merge(imported_spec)?;
                } else {
                    spec.add(child, file)?;
                }
            }
            already_imported.insert(abs_path.clone());
            Ok(spec)
        }
        _unknown => Err(Box::new(unknown_rule_error(pair, "spec"))),
    }
}

pub fn parse_spec<P: AsRef<Path>>(path: P) -> Result<Spec, Box<dyn Error>> {
    private_parse_spec(path, Vec::new(), &mut HashSet::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parses_spec_1() {
        let spec_or_err = parse_spec("test_graphqxl_files/1.graphqxl");
        if let Err(err) = spec_or_err {
            panic!("Error parsing file: {}", err)
        }
    }

    #[test]
    fn test_handles_cyclical_imports() {
        let err = parse_spec("test_graphqxl_files/cyclical1.graphqxl").unwrap_err();
        assert!(err.to_string().contains("cyclical"))
    }

    #[test]
    fn test_does_not_duplicate_imports() {
        let spec_or_err = parse_spec("test_graphqxl_files/no_duplicated1.graphqxl");
        if let Err(err) = spec_or_err {
            panic!("Error parsing file: {}", err)
        }
    }
}
