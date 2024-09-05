use super::{instrumentation::instrument_line, test_coverage_container::TestCoverageContainer};
use rhai::{
    Engine, EvalAltResult, Expr, FnCallExpr, Module, ModuleResolver, Position, Scope, Stmt, AST,
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub struct FileCoverageModuleResolver {
    base_path: PathBuf,
    test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
    cache: Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>,
}

impl FileCoverageModuleResolver {
    pub fn new(
        base_path: impl Into<PathBuf>,
        test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
        module_cache: Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>,
    ) -> Self {
        Self {
            base_path: base_path.into(),
            test_coverage_container,
            cache: module_cache,
        }
    }

    pub fn get_file_path(&self, path: &str, source_path: Option<&Path>) -> PathBuf {
        let path = Path::new(path);

        let mut file_path: PathBuf;

        if path.is_relative() {
            file_path = self.base_path.clone();
            file_path.push(path);
        } else {
            file_path = path.into();
        }

        file_path.set_extension("rhai"); // Force extension
        file_path
    }
}

impl ModuleResolver for FileCoverageModuleResolver {
    fn resolve(
        &self,
        engine: &Engine,
        source: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<rhai::Shared<rhai::Module>, Box<rhai::EvalAltResult>> {
        let global = &mut engine.new_global_runtime_state();
        let scope = &mut Scope::new();
        let source_path = global
            .source()
            .or(source)
            .and_then(|p| Path::new(p).parent());
        let file_path = self.get_file_path(path, source_path);

        if let Some(module) = self.cache.lock().unwrap().get(&file_path) {
            return Ok(module.clone());
        }

        let mut contents = fs::read_to_string(file_path.clone())
            .map_err(|_| Box::new(EvalAltResult::ErrorModuleNotFound(path.to_string(), pos)))?;

        contents = contents
            .lines()
            .enumerate()
            .map(|(i, line)| instrument_line(i, line, path, self.test_coverage_container.clone()))
            .collect::<Vec<_>>()
            .join("\n");

        //println!("{}", contents);

        let mut ast = engine.compile(&contents).map_err(|err| {
            Box::new(EvalAltResult::ErrorInModule(
                path.to_string(),
                err.into(),
                pos,
            ))
        })?;
        ast.set_source(path);

        let m: Arc<Module> = Module::eval_ast_as_new_raw(engine, scope, global, &ast)
            .map_err(|err| Box::new(EvalAltResult::ErrorInModule(path.to_string(), err, pos)))?
            .into();

        self.cache.lock().unwrap().insert(file_path, m.clone());

        Ok(m)
    }
}
